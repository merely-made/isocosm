// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, rules::*, schema::*, simulation::*};
use std::collections::{BTreeMap, BTreeSet};

impl Simulation {
    pub(crate) fn apply(
        &mut self,
        actor: Id,
        target: Option<Id>,
        process: &str,
        cause: Option<Key>,
        count: u64,
    ) -> Receipt {
        let before = self.matter();
        let id = format!(
            "event:{}",
            crate::digest(&(
                self.genesis.seed,
                self.state.tick,
                self.state.next_action,
                actor,
                process
            ))
        );
        let mut receipt = Receipt {
            id: id.clone(),
            tick: self.state.tick,
            revision: self.revision.clone(),
            actor,
            target,
            count,
            chosen: process.into(),
            foregone: vec![],
            cause: cause.clone(),
            facts_read: vec![],
            effects: vec![],
            outcome: Outcome::Accepted,
            matter_before: before,
            matter_after: before,
        };
        let Some(definition) = self.genesis.rules.processes.get(process).cloned() else {
            receipt.outcome = Outcome::Refused(format!("unknown process {process}"));
            return receipt;
        };
        if count == 0 || (count > 1 && !definition.bulk_safe()) {
            receipt.outcome = Outcome::Refused("process cannot execute as a cohort".into());
            return receipt;
        }
        if count > 1
            && self
                .state
                .population
                .groups
                .get(&actor)
                .is_none_or(|g| g.count != count)
        {
            receipt.outcome = Outcome::Refused("cohort identity interval changed".into());
            return receipt;
        }
        let Some(entity) = self.state.population.get(actor) else {
            receipt.outcome = Outcome::Refused("actor is absent".into());
            return receipt;
        };
        let place = entity.place;
        if definition.target.is_some() && !self.target_matches(actor, target, &definition) {
            receipt.outcome = Outcome::Blocked("no target satisfies the declared scope".into());
            return receipt;
        }
        if cause
            .as_ref()
            .is_some_and(|id| !self.state.events.contains_key(id))
        {
            receipt.outcome = Outcome::Refused("unknown causal event".into());
            return receipt;
        }
        for query in &definition.requires {
            match self.query(actor, target, place, query) {
                Ok(fact) => receipt.facts_read.push(fact),
                Err(why) => {
                    receipt.outcome = Outcome::Blocked(why);
                    return receipt;
                },
            }
        }
        receipt.foregone = self
            .genesis
            .rules
            .processes
            .values()
            .filter(|p| p.id != process && p.shape == Shape::Choice)
            .filter(|p| {
                self.target_matches(actor, target, p)
                    && p.requires
                        .iter()
                        .all(|q| self.query(actor, target, place, q).is_ok())
            })
            .map(|p| p.id.clone())
            .collect();
        let mut staged = self.clone();
        // Single writes split before mutation. Bulk writes preserve one interval.
        if count == 1 {
            staged.state.population.lift(actor).unwrap();
        }
        if let Some(target) = target
            && let Err(why) = staged.state.population.lift(target)
        {
            receipt.outcome = Outcome::Blocked(why);
            return receipt;
        }
        let risky = definition.risk.as_ref().is_some_and(|r| {
            crate::draw(self.genesis.seed, &id, &[actor]) % 1_000_000 < u64::from(r.per_million)
        });
        let outcomes = if risky {
            &definition.risk.as_ref().unwrap().effects
        } else {
            &definition.effects
        };
        let effects: Vec<_> = definition
            .commitments
            .iter()
            .chain(outcomes)
            .cloned()
            .collect();
        let mut legend = false;
        for effect in &effects {
            match staged.effect(actor, target, place, effect, &id) {
                Ok(feat) => legend |= feat,
                Err(why) => {
                    receipt.outcome = Outcome::Blocked(why);
                    return receipt;
                },
            }
        }
        let after = staged.matter();
        if before != after {
            receipt.outcome = Outcome::Refused("matter invariant would be violated".into());
            return receipt;
        }
        let Some(next) = staged.state.next_action.checked_add(count) else {
            receipt.outcome = Outcome::Refused("action sequence exhausted".into());
            return receipt;
        };
        staged.state.next_action = next;
        if definition.note {
            if staged.state.events.len() >= staged.genesis.rules.limits.history {
                receipt.outcome = Outcome::Refused("event budget exhausted".into());
                return receipt;
            }
            let event = Event {
                id: id.clone(),
                tick: self.state.tick,
                place,
                subject: actor,
                process: process.into(),
                cause,
                strength: self.genesis.rules.field.strength,
                legend,
            };
            if let Err(why) = staged.state.reach.seed(&event, &staged.state.sites) {
                receipt.outcome = Outcome::Refused(why);
                return receipt;
            }
            staged.state.events.insert(id.clone(), event);
            if let Err(why) = staged.add_note(
                actor,
                id,
                "sim:act",
                String::new(),
                None,
                receipt.id.clone(),
            ) {
                receipt.outcome = Outcome::Refused(why);
                return receipt;
            }
        }
        receipt.effects = effects;
        receipt.matter_after = after;
        if risky {
            receipt.outcome = Outcome::RiskOutcome;
        }
        self.state = staged.state;
        receipt
    }
    fn effect(
        &mut self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        e: &Effect,
        cause: &str,
    ) -> Result<bool> {
        match e {
            Effect::Transfer {
                from,
                to,
                account,
                amount,
            } => {
                debit(
                    self.ledger_mut(actor, target, place, *from)?,
                    account,
                    *amount,
                )?;
                credit(
                    self.ledger_mut(actor, target, place, *to)?,
                    account,
                    *amount,
                )?;
            },
            Effect::Transform { who, take, give } => {
                let ledger = self.ledger_mut(actor, target, place, *who)?;
                for (key, amount) in take {
                    debit(ledger, key, *amount)?;
                }
                for (key, amount) in give {
                    credit(ledger, key, *amount)?;
                }
            },
            Effect::Condition { key, delta } => {
                let slot = self
                    .state
                    .sites
                    .get_mut(&place)
                    .unwrap()
                    .conditions
                    .entry(key.clone())
                    .or_default();
                *slot = slot.checked_add(*delta).ok_or("condition overflow")?;
            },
            Effect::Relate { kind, present } => {
                let relation = Relation {
                    subject: actor,
                    kind: kind.clone(),
                    object: target.ok_or("target required")?,
                };
                if *present {
                    self.state.relations.insert(relation);
                } else {
                    self.state.relations.remove(&relation);
                }
            },
            Effect::Trait { who, key, present } => {
                let id = self.bound(actor, target, *who)?;
                let entity = &mut self
                    .state
                    .population
                    .groups
                    .get_mut(&id)
                    .ok_or("body missing")?
                    .entity;
                if *present {
                    entity.traits.insert(key.clone());
                } else {
                    entity.traits.remove(key);
                }
                entity.body_revision = entity
                    .body_revision
                    .checked_add(1)
                    .ok_or("body revision overflow")?;
            },
            Effect::Practice { key, amount } => {
                let slot = self
                    .state
                    .population
                    .groups
                    .get_mut(&actor)
                    .unwrap()
                    .entity
                    .skills
                    .entry(key.clone())
                    .or_default();
                *slot = slot.checked_add(*amount).ok_or("skill overflow")?;
            },
            Effect::Move { destination } => {
                if !self.state.sites[&place]
                    .routes
                    .iter()
                    .any(|r| r.to == *destination)
                {
                    return Err("no route to destination".into());
                }
                let entity = &mut self.state.population.groups.get_mut(&actor).unwrap().entity;
                if entity.arrived < self.state.tick {
                    entity.visits.push(Visit {
                        place,
                        from: entity.arrived,
                        until: self.state.tick,
                    });
                }
                entity.place = *destination;
                entity.arrived = self.state.tick;
            },
            Effect::Note {
                kind,
                text,
                lifetime,
            } => {
                let expires = lifetime
                    .map(|t| self.state.tick.checked_add(t).ok_or("note expiry overflow"))
                    .transpose()?;
                self.add_note(
                    actor,
                    format!("site:{place}"),
                    kind,
                    text.clone(),
                    expires,
                    cause.into(),
                )?;
            },
            Effect::Birth { provision } => {
                if self.state.population.count() >= self.genesis.rules.limits.entities {
                    return Err("population limit".into());
                }
                let mut child = self.state.population.get(actor).unwrap().clone();
                child.accounts = provision.clone();
                child.born = self.state.tick;
                child.arrived = self.state.tick;
                child.visits.clear();
                child.skills = BTreeMap::new();
                child.provenance = Provenance::Born(child.lineage.clone());
                for (key, value) in provision {
                    debit(
                        self.ledger_mut(actor, target, place, Binding::Actor)?,
                        key,
                        *value,
                    )?;
                }
                let id = self.state.population.insert(child, 1)?;
                self.state.relations.insert(Relation {
                    subject: id,
                    object: actor,
                    kind: "sim:parent".into(),
                });
            },
            Effect::Death => {
                self.state
                    .population
                    .groups
                    .get_mut(&actor)
                    .unwrap()
                    .entity
                    .alive = false;
            },
            Effect::Tell { event } => {
                if !self.knows(actor, event)? {
                    return Err("actor does not know this event".into());
                }
                let target = target.ok_or("hearer required")?;
                self.add_note(
                    target,
                    event.clone(),
                    "sim:discover",
                    String::new(),
                    None,
                    cause.into(),
                )?;
            },
            Effect::FoundPolity {
                governance,
                focus,
                support,
            } => {
                let mut members = BTreeSet::from([actor]);
                if let Some(target) = target {
                    members.insert(target);
                }
                if self.state.polities.contains_key(&actor) {
                    return Err("founder already has a polity".into());
                }
                self.state.polities.insert(
                    actor,
                    Polity {
                        constitution: Constitution {
                            members,
                            governance: governance.clone(),
                            focus: focus.clone(),
                            support_account: support.clone(),
                            host: None,
                            founded_by: cause.into(),
                        },
                        accounts: BTreeMap::new(),
                        ended_by: None,
                    },
                );
            },
            Effect::Record { axis, account } => {
                let value = self
                    .ledger(actor, target, place, Binding::Actor)?
                    .get(account)
                    .copied()
                    .unwrap_or(0);
                let value =
                    i64::try_from(value).map_err(|_| "record reading exceeds signed range")?;
                return Ok(self.state.record.reckon(&[hagiograph::Entry {
                    axis: axis.clone(),
                    value,
                    holder: actor,
                }])[0]
                    .feat);
            },
        }
        Ok(false)
    }
}

fn debit(ledger: &mut Ledger, key: &str, amount: u64) -> Result<()> {
    let slot = ledger.entry(key.into()).or_default();
    *slot = slot
        .checked_sub(amount)
        .ok_or_else(|| format!("insufficient {key}"))?;
    Ok(())
}
fn credit(ledger: &mut Ledger, key: &str, amount: u64) -> Result<()> {
    let slot = ledger.entry(key.into()).or_default();
    *slot = slot
        .checked_add(amount)
        .ok_or_else(|| format!("account overflow: {key}"))?;
    Ok(())
}
