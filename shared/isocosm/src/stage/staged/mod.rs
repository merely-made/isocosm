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
    schedule::edible,
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
            let site = self.sim.site_at_start(self.stage.place);
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

    /// Computes against the stage as it stands what an amount or a guard
    /// reads: each draw keyed by the act and its slot (X3), and a sum over a
    /// body's parts reading each of its living parts (ruling 455).
    fn compute<T>(
        &mut self,
        f: impl FnOnce(&mut Read, &mut Draw, &mut PartsOf) -> Result<T>,
    ) -> Result<T> {
        let (seed, act) = (self.sim.genesis.seed, self.stage.act);
        let sim = self.sim;
        let b = sim.genesis.rules.body();
        let living = |e: &Entity| -> Vec<Part> {
            e.parts.values().filter(|p| !p.severed).cloned().collect()
        };
        let mine = living(self.body(Binding::Actor)?);
        let theirs = match self.stage.target {
            Some(_) => Some(living(self.body(Binding::Target)?)),
            None => None,
        };
        let mut parts = |who: Binding| -> Result<(Vec<Part>, BodyRules)> {
            match who {
                Binding::Actor => Ok((mine.clone(), b)),
                Binding::Target => match theirs.clone() {
                    Some(t) => Ok((t, b)),
                    None => Err("no target is bound".into()),
                },
                _ => Err(format!("{who:?} has no parts")),
            }
        };
        let mut read = |r: &Reading| -> Result<i64> {
            match r {
                Reading::Kept { name } => {
                    let kept = self.stage.kept.get(name);
                    kept.copied()
                        .ok_or_else(|| format!("no value {name} was kept"))
                },
                Reading::Account { who, key } => {
                    let held = Parties::held(self, *who, key)?;
                    i64::try_from(held).map_err(|e| e.to_string())
                },
                r if r.who() == Binding::Part => {
                    let (body, id) = self.part()?;
                    let part = body.parts.get(&id).ok_or("bound part missing")?;
                    Ok(i64::try_from(r.of_part(part, b)).unwrap_or(i64::MAX))
                },
                r => Ok(meaning::body_reading(
                    self.body(r.who())?,
                    r,
                    &sim.genesis.rules,
                    Some(&sim.state.lineages),
                )),
            }
        };
        let mut draw = |below: u64, slot: u8| -> Result<u64> {
            Ok(crate::draw(seed, "amount", &[act, u64::from(slot)]) % below)
        };
        f(&mut read, &mut draw, &mut parts)
    }

    /// The effect with its amounts resolved against the stage as it stands.
    fn resolve<'e>(&mut self, e: &'e Effect) -> Result<Cow<'e, Effect>> {
        if !e.computes() {
            return Ok(Cow::Borrowed(e));
        }
        let resolved = self.compute(|read, draw, parts| e.resolve(read, draw, parts))?;
        Ok(Cow::Owned(resolved))
    }

    /// The effect's takes of ground the act shares (ruling 454): recorded
    /// while its pass plans, held to the act's shares when it acts. A meal's
    /// share is the matter it takes, which the meal reads itself.
    fn share_out<'e>(&mut self, e: Cow<'e, Effect>) -> Result<Cow<'e, Effect>> {
        let shared = |b: &Binding| matches!(b, Binding::Place | Binding::Target);
        let holder = |b: &Binding| self.holder(*b).ok_or("shared ground missing");
        match e.as_ref() {
            Effect::Transfer {
                from,
                account,
                amount,
                ..
            } if shared(from) => {
                let (holder, asked) = (holder(from)?, amount.resolved()?);
                let key = (holder, account.clone());
                // A plan asks for the whole take and stages what is there.
                let given = if let Some(d) = &mut self.stage.demands {
                    d.accounts.push((holder, account.clone(), asked));
                    asked.min(meaning::value(self.ledger(*from)?, account))
                } else if let Some(shares) = &mut self.stage.shares {
                    let left = shares.accounts.entry(key).or_default();
                    let given = asked.min(*left);
                    *left -= given;
                    given
                } else {
                    return Ok(e);
                };
                let mut e = e.into_owned();
                if let Effect::Transfer { amount, .. } = &mut e {
                    *amount = given.into();
                }
                Ok(Cow::Owned(e))
            },
            Effect::Transform { who, take, .. } if shared(who) && !take.is_empty() => {
                let holder = holder(who)?;
                if let Some(d) = &mut self.stage.demands {
                    let takes = take.iter().map(|(k, v)| (holder, k.clone(), *v));
                    d.accounts.extend(takes);
                }
                if let Some(shares) = &mut self.stage.shares {
                    for (k, v) in take {
                        let left = shares.accounts.entry((holder, k.clone())).or_default();
                        if *left < *v {
                            return Err("a transform of shared ground cannot be shared out".into());
                        }
                        *left -= v;
                    }
                }
                Ok(e)
            },
            Effect::Eat {
                from, amount, of, ..
            } if shared(from) => {
                let (holder, asked) = (holder(from)?, amount.resolved()?);
                if let Some(d) = &mut self.stage.demands {
                    d.meal = Some((holder, asked, of.clone()));
                }
                Ok(e)
            },
            _ => Ok(e),
        }
    }

    /// Applies one effect to the stage, a guarded one by the branch its
    /// guard chooses (X5). Returns whether it set a feat.
    pub(crate) fn effect(&mut self, e: &Effect, cause: &str) -> Result<bool> {
        if let Effect::Keep { name, value } = e {
            let value = self.compute(|read, draw, parts| value.eval_in(read, draw, parts))?;
            self.stage.kept.insert(name.clone(), value);
            return Ok(false);
        }
        if matches!(e, Effect::When { .. }) {
            let branch = self.compute(|read, draw, parts| e.branch(read, draw, parts))?;
            let mut feat = false;
            for inner in branch.expect("a guard chooses a branch") {
                feat |= self.effect(inner, cause)?;
            }
            return Ok(feat);
        }
        let resolved = self.resolve(e)?;
        let resolved = self.share_out(resolved)?;
        let e = resolved.as_ref();
        let rules = &self.sim.genesis.rules;
        // Moves are worked out only while a host keeps the flow record.
        let legs = match self.sim.flowing() {
            true => flows::moves(e, rules, |who| self.holder(who)),
            false => vec![],
        };
        let sim = self.sim;
        // An ordered take and a conversion read the balances they take, so
        // their moves are worked out as they are applied.
        match e {
            Effect::Spend {
                from,
                to,
                amount,
                into,
            } => {
                let paid = meaning::spend(self, from, *to, into.as_deref(), amount.resolved()?)?;
                let ends = (self.holder(Binding::Actor), self.holder(*to));
                if let (true, (Some(start), Some(end))) = (sim.flowing(), ends) {
                    let matter = |k: &Key| meaning::matter(&sim.genesis.rules, k);
                    for (key, paid) in paid.into_iter().filter(|(k, _)| matter(k)) {
                        let arrives = into.clone().unwrap_or_else(|| key.clone());
                        self.stage.legs.push(Leg {
                            from: (start, key),
                            to: (end, arrives),
                            amount: paid,
                        });
                    }
                }
                return Ok(false);
            },
            Effect::Grow {
                from,
                into,
                conversion,
            } => {
                let rules = &sim.genesis.rules;
                let grown = meaning::grow(self, rules, from, into, *conversion)?;
                let ends = (self.holder(Binding::Actor), self.holder(Binding::Place));
                if let (true, (Some(start), Some(end))) = (sim.flowing(), ends) {
                    for (key, amount) in grown.paid {
                        let (from, to) = ((start, key), (end, into.clone()));
                        self.stage.legs.push(Leg { from, to, amount });
                    }
                    for (taken, to, total) in grown.taken {
                        self.stage
                            .legs
                            .extend(flows::poured(start, taken, [(to, total)]));
                    }
                }
                return Ok(false);
            },
            Effect::Convert {
                who,
                from,
                to,
                amount,
                conversion,
            } => {
                let rules = &sim.genesis.rules;
                let amount = amount.resolved()?;
                let taken = meaning::convert(self, rules, *who, (from, to), amount, *conversion)?;
                if let (true, Some(holder)) = (sim.flowing(), self.holder(*who)) {
                    let total: u64 = taken.values().sum();
                    let legs = flows::poured(holder, taken, [(to.clone(), total)]);
                    self.stage.legs.extend(legs);
                }
                return Ok(false);
            },
            _ => {},
        }
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
            Effect::Bear { clutch, young } => self.bear(*clutch, young.as_ref())?,
            // A semelparous budder dies as its bud severs (521).
            Effect::Bud { mark, once } => {
                if self.bud(mark)? && *once {
                    self.effect(&Effect::Death, cause)?;
                }
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
            // takes is `meaning::share`'s, of the accounts it names (ruling
            // 456), or its share of a prey its pass shared out (ruling 454).
            Effect::Eat {
                from,
                amount,
                into,
                of,
                whole,
            } => {
                let rules = &sim.genesis.rules;
                let portion = self.stage.shares.as_mut().and_then(|s| s.meal.take());
                // A bite lands on one part, drawn by what each holds (459).
                let draw = crate::draw(sim.genesis.seed, "bite", &[self.stage.act]);
                let bitten = crate::anatomy::bitten(self.body(*from)?, of, draw);
                let offered = match bitten {
                    Some(part) => {
                        let prey = self.body(*from)?;
                        edible(
                            &prey.parts.get(&part).ok_or("bitten part missing")?.matter,
                            of,
                        )
                    },
                    None => edible(&*self.ledger(*from)?, of),
                };
                // A bite that would take all of a part may take it whole.
                if let (true, Some(part)) = (*whole, bitten) {
                    let asked: u64 = match &portion {
                        Some(p) => p.values().sum(),
                        None => amount.resolved()?,
                    };
                    let all: u64 = offered.values().sum();
                    if asked >= all && self.incorporate(*from, part)? {
                        return Ok(false);
                    }
                }
                let taken: Ledger = match portion {
                    // A share is of the prey's whole; the bitten part gives
                    // what it holds of it, the rest staying put (454).
                    Some(portion) => portion
                        .into_iter()
                        .map(|(k, v)| {
                            let held = offered.get(&k).copied().unwrap_or(0);
                            (k, v.min(held))
                        })
                        .filter(|(_, v)| *v > 0)
                        .collect(),
                    None => meaning::share(&offered, rules, amount.resolved()?),
                };
                let prey = self.holder(*from);
                let mut total = 0u64;
                for (key, value) in &taken {
                    match (bitten, prey) {
                        (Some(part), Some(Holder::Entity(id))) => {
                            let body = self.body(*from)?;
                            let p = body.parts.get_mut(&part).ok_or("bitten part missing")?;
                            debit(&mut p.matter, key, *value)?;
                            let routed = flows::Routed {
                                body: id,
                                key: key.clone(),
                                give: false,
                                parts: vec![(part, *value)],
                            };
                            self.stage.routed.push(routed);
                        },
                        _ => debit(self.ledger(*from)?, key, *value)?,
                    }
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
            // Every other effect has its meaning in `meaning::effect`.
            _ => unreachable!("shared effects return above"),
        }
        Ok(false)
    }
}

mod births;
mod incorporate;
mod parties;
