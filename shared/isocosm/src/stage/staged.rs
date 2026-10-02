// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Effects applied to a stage, read against the world it will be committed
//! to: the individual runner's side of `meaning`'s shared effects, and the
//! effects only it applies.

use super::Stage;
use crate::{
    Result,
    flows::{self, Holder, Leg},
    meaning::{self, Parties, credit, debit},
    rules::*,
    schema::*,
    simulation::Simulation,
};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
};

/// A stage read against the world it will be committed to.
pub(crate) struct Staged<'a> {
    pub sim: &'a Simulation,
    pub stage: &'a mut Stage,
}

impl Staged<'_> {
    fn ledger(&mut self, who: Binding) -> Result<&mut Ledger> {
        if who == Binding::Place {
            return Ok(&mut self.site()?.accounts);
        }
        let id = self.sim.bound(self.stage.actor, self.stage.target, who)?;
        let body = self.stage.bodies.get_mut(&id);
        Ok(&mut body.ok_or("body binding changed")?.accounts)
    }
    fn site(&mut self) -> Result<&mut Site> {
        if self.stage.site.is_none() {
            let site = self.sim.state.sites.get(&self.stage.place);
            self.stage.site = Some(site.ok_or("site missing")?.clone());
        }
        Ok(self.stage.site.as_mut().expect("copied above"))
    }
    /// Who holds a binding's ledger in this act.
    fn holder(&self, who: Binding) -> Option<Holder> {
        match who {
            Binding::Actor => Some(Holder::Entity(self.stage.actor)),
            Binding::Target => self.stage.target.map(Holder::Entity),
            Binding::Place => Some(Holder::Site(self.stage.place)),
            Binding::Part => None,
        }
    }
    fn actor(&mut self) -> &mut Entity {
        let actor = self.stage.actor;
        self.stage
            .bodies
            .get_mut(&actor)
            .expect("the actor is staged")
    }
    pub(super) fn add_note(
        &mut self,
        subject: Id,
        object: Key,
        kind: &str,
        djot: String,
        expires: Option<Tick>,
        cause: Key,
    ) -> Result<()> {
        self.sim
            .room(self.sim.state.notes.len() + self.stage.notes.len())?;
        let note = self.sim.note(subject, object, kind, djot, expires, cause)?;
        self.stage.notes.push(note);
        Ok(())
    }

    /// The effect with its amounts resolved against the stage as it stands,
    /// each draw keyed by the act and its place among the act's draws (X3),
    /// or none where a guard came to nothing (X5).
    fn resolve<'e>(&mut self, e: &'e Effect) -> Result<Option<Cow<'e, Effect>>> {
        if !e.computes() {
            return Ok(Some(Cow::Borrowed(e)));
        }
        let (seed, act) = (self.sim.genesis.seed, self.sim.state.next_action);
        let mut draws = self.stage.draws;
        let mut read = |r: &Reading| -> Result<i64> {
            match r {
                Reading::Account { who, key } => {
                    let held = meaning::value(self.ledger(*who)?, key);
                    i64::try_from(held).map_err(|e| e.to_string())
                },
                r => Ok(meaning::body_reading(self.body(r.who())?, r)),
            }
        };
        let mut draw = |below: u64| -> Result<u64> {
            draws += 1;
            Ok(crate::draw(seed, "amount", &[act, draws]) % below)
        };
        let resolved = e.resolve(&mut read, &mut draw)?;
        self.stage.draws = draws;
        Ok(resolved.map(Cow::Owned))
    }

    /// Applies one effect to the stage. Returns whether it set a feat.
    pub(crate) fn effect(&mut self, e: &Effect, cause: &str) -> Result<bool> {
        let Some(resolved) = self.resolve(e)? else {
            return Ok(false);
        };
        let e = resolved.as_ref();
        let rules = &self.sim.genesis.rules;
        // Moves are worked out only while a host keeps the flow record.
        let legs = match self.sim.flowing() {
            true => flows::moves(e, rules, |who| self.holder(who)),
            false => vec![],
        };
        if let Some(done) = meaning::effect(self, rules, e) {
            done?;
            self.stage.legs.extend(legs);
            return Ok(false);
        }
        let (actor, target, place) = (self.stage.actor, self.stage.target, self.stage.place);
        let (sim, tick) = (self.sim, self.sim.state.tick);
        match e {
            Effect::Relate { kind, present } => {
                let relation = Relation {
                    subject: actor,
                    kind: kind.clone(),
                    object: target.ok_or("target required")?,
                };
                self.stage.relations.push((relation, *present));
            },
            Effect::Move { destination } => {
                if !sim.state.sites[&place]
                    .routes
                    .iter()
                    .any(|r| r.to == *destination)
                {
                    return Err("no route to destination".into());
                }
                let entity = self.actor();
                if entity.arrived < tick {
                    entity.visits.push(Visit {
                        place,
                        from: entity.arrived,
                        until: tick,
                    });
                }
                entity.place = *destination;
                entity.arrived = tick;
            },
            Effect::Note {
                kind,
                text,
                lifetime,
            } => {
                let expires = lifetime
                    .map(|t| tick.checked_add(t).ok_or("note expiry overflow"))
                    .transpose()?;
                let object = format!("site:{place}");
                self.add_note(actor, object, kind, text.clone(), expires, cause.into())?;
            },
            Effect::Birth { provision } => {
                let born = self.stage.births.len() as u64;
                if sim.state.population.count() + born >= sim.genesis.rules.limits.entities {
                    return Err("population limit".into());
                }
                let mut child = self.actor().clone();
                child.accounts = provision.clone();
                child.born = tick;
                child.arrived = tick;
                child.visits.clear();
                child.skills = BTreeMap::new();
                child.provenance = Provenance::Born(child.lineage.clone());
                for (key, value) in provision {
                    debit(&mut self.actor().accounts, key, *value)?;
                }
                // Children take identities in order after the world's last.
                let id = sim.state.population.next_id + born;
                id.checked_add(1).ok_or("identity exhausted")?;
                self.stage.births.push(child);
                let parent = Relation {
                    subject: id,
                    object: actor,
                    kind: "sim:parent".into(),
                };
                self.stage.relations.push((parent, true));
                // The child's matter is its parent's, moved (ruling 345).
                let rules = &sim.genesis.rules;
                let matter =
                    |k: &Key| matches!(rules.accounts.get(k), Some(AccountKind::Matter { .. }));
                let moved = provision.iter().filter(|(k, v)| matter(k) && **v > 0);
                for (key, value) in moved.filter(|_| sim.flowing()) {
                    self.stage.legs.push(Leg {
                        from: (Holder::Entity(actor), key.clone()),
                        to: (Holder::Entity(id), key.clone()),
                        amount: *value,
                    });
                }
            },
            Effect::Tell { event } => {
                let teller = &self.stage.bodies[&actor];
                if !sim.known(actor, teller, &self.stage.notes, event)? {
                    return Err("actor does not know this event".into());
                }
                let target = target.ok_or("hearer required")?;
                let kind = "sim:discover";
                self.add_note(
                    target,
                    event.clone(),
                    kind,
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
                if sim.state.polities.contains_key(&actor)
                    || self.stage.polities.iter().any(|(id, _)| *id == actor)
                {
                    return Err("founder already has a polity".into());
                }
                let polity = Polity {
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
                };
                self.stage.polities.push((actor, polity));
            },
            Effect::Record { axis, account } => {
                let value = meaning::value(&self.stage.bodies[&actor].accounts, account);
                let value =
                    i64::try_from(value).map_err(|_| "record reading exceeds signed range")?;
                // Hagiograph's feat: beating a mark that stood before.
                let standing = match self.stage.highs.get(axis) {
                    Some(high) => Some(*high),
                    None => sim.state.record.standing(axis).map(|m| m.high),
                };
                let high = standing.map_or(value, |h| h.max(value));
                self.stage.highs.insert(axis.clone(), high);
                self.stage.marks.push((axis.clone(), value));
                return Ok(standing.is_some_and(|h| value > h));
            },
            // Eating binds another body, so it is applied here; what a meal
            // takes is `meaning::share`'s.
            Effect::Eat { from, amount, into } => {
                let rules = &sim.genesis.rules;
                let taken = meaning::share(&*self.ledger(*from)?, rules, amount.resolved()?);
                let source = self.ledger(*from)?;
                let mut total = 0u64;
                for (key, value) in &taken {
                    debit(source, key, *value)?;
                    total = total.checked_add(*value).ok_or("meal overflow")?;
                }
                credit(self.ledger(Binding::Actor)?, into, total)?;
                if let Some(prey) = self.holder(*from).filter(|_| sim.flowing()) {
                    self.stage
                        .legs
                        .extend(taken.into_iter().map(|(key, amount)| Leg {
                            from: (prey, key),
                            to: (Holder::Entity(actor), into.clone()),
                            amount,
                        }));
                }
            },
            // An ordered take reads the balances it drains, so it is applied
            // here: each account in turn gives what it holds, up to what is
            // still owed (ruling 446).
            Effect::Spend { from, to, amount } => {
                let mut owed = amount.resolved()?;
                for key in from {
                    let held = meaning::value(self.ledger(Binding::Actor)?, key);
                    let paid = held.min(owed);
                    if paid == 0 {
                        continue;
                    }
                    debit(self.ledger(Binding::Actor)?, key, paid)?;
                    credit(self.ledger(*to)?, key, paid)?;
                    owed -= paid;
                    let matter = matches!(
                        sim.genesis.rules.accounts.get(key),
                        Some(AccountKind::Matter { .. })
                    );
                    let ends = (self.holder(Binding::Actor), self.holder(*to));
                    if let (true, true, (Some(from), Some(to))) = (sim.flowing(), matter, ends) {
                        self.stage.legs.push(Leg {
                            from: (from, key.clone()),
                            to: (to, key.clone()),
                            amount: paid,
                        });
                    }
                }
            },
            // Every other effect has its meaning in `meaning::effect`.
            _ => unreachable!("shared effects return above"),
        }
        Ok(false)
    }
}

/// The individual runner's parties for the shared effect meanings: the
/// staged bodies and site.
impl Parties for Staged<'_> {
    fn reach(&mut self, who: Binding) -> Result<()> {
        if who == Binding::Place {
            let site = self.sim.state.sites.get(&self.stage.place);
            return site.map(|_| ()).ok_or_else(|| "site missing".into());
        }
        self.ledger(who).map(|_| ())
    }
    fn take(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        debit(self.ledger(who)?, key, amount)
    }
    fn give(&mut self, who: Binding, key: &str, amount: u64) -> Result<()> {
        credit(self.ledger(who)?, key, amount)
    }
    fn body(&mut self, who: Binding) -> Result<&mut Entity> {
        let id = self.sim.bound(self.stage.actor, self.stage.target, who)?;
        Ok(self.stage.bodies.get_mut(&id).ok_or("body missing")?)
    }
    fn part(&mut self) -> Result<(&mut Entity, Id)> {
        let part = self.stage.part.ok_or("no part is bound")?;
        Ok((self.actor(), part))
    }
    fn shift(&mut self, key: &str, delta: i64) -> Result<()> {
        meaning::shift(&mut self.site()?.conditions, key, delta, 1)
    }
}
