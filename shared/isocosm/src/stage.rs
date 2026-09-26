// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One act's writes, held apart from the world until the act is accepted.
//! The interpreter once staged each act on a copy of the whole simulation.
//! A stage copies only what the act binds, its actor's and target's bodies
//! and its site, and lists what the act adds: children, relations, notes,
//! polities, record marks and its event. Committing writes exactly what the
//! whole copy would have held; dropping the stage leaves the world as it
//! was, identity grouping included.

use crate::{
    Result,
    meaning::{self, Parties, credit, debit, mass},
    reach::Reach,
    rules::*,
    schema::*,
    simulation::Simulation,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct Stage {
    actor: Id,
    target: Option<Id>,
    place: Id,
    /// The members the actor stands for: one, or a whole bulk cohort.
    count: u64,
    /// The bound bodies as the act leaves them. A target that is the actor
    /// is the same body.
    bodies: BTreeMap<Id, Entity>,
    /// The site's ledger and conditions, copied on the first write.
    site: Option<Site>,
    births: Vec<Entity>,
    /// Inserted (`true`) and removed relations, in the order the act made
    /// them.
    relations: Vec<(Relation, bool)>,
    notes: Vec<Note>,
    polities: Vec<(Id, Polity)>,
    /// Record entries in order, and each axis's high as the act leaves it,
    /// so a later entry is judged against an earlier one.
    marks: Vec<(Key, i64)>,
    highs: BTreeMap<Key, i64>,
    event: Option<(Event, Reach)>,
}

impl Simulation {
    /// A stage for one act by `actor`, standing for `count` members at
    /// `place`. A named target that does not exist fails here, as lifting
    /// it from a whole copy would.
    pub(crate) fn stage(
        &self,
        actor: Id,
        target: Option<Id>,
        place: Id,
        count: u64,
    ) -> Result<Stage> {
        // A cohort acts only through processes without targets.
        debug_assert!(count == 1 || target.is_none());
        let population = &self.state.population;
        let mut bodies = BTreeMap::new();
        let body = population.get(actor).ok_or("unknown entity")?;
        bodies.insert(actor, body.clone());
        if let Some(target) = target {
            let body = population.get(target).ok_or("unknown entity")?;
            bodies.entry(target).or_insert_with(|| body.clone());
        }
        Ok(Stage {
            actor,
            target,
            place,
            count,
            bodies,
            site: None,
            births: vec![],
            relations: vec![],
            notes: vec![],
            polities: vec![],
            marks: vec![],
            highs: BTreeMap::new(),
            event: None,
        })
    }

    /// Whether the staged act would change the world's matter: the bodies
    /// and site it binds, weighed before and after, and any children.
    pub(crate) fn moves_matter(&self, stage: &Stage) -> bool {
        let weigh = |ledger: &Ledger| mass(ledger, &self.genesis.rules);
        let (mut before, mut after) = (0u128, 0u128);
        for (id, body) in &stage.bodies {
            let weight = u128::from(if *id == stage.actor { stage.count } else { 1 });
            let was = self.state.population.get(*id).expect("staged bodies exist");
            before += weigh(&was.accounts) * weight;
            after += weigh(&body.accounts) * weight;
        }
        if let Some(site) = &stage.site {
            before += weigh(&self.state.sites[&stage.place].accounts);
            after += weigh(&site.accounts);
        }
        after += stage
            .births
            .iter()
            .map(|c| weigh(&c.accounts))
            .sum::<u128>();
        before != after
    }

    /// Adds the act's public event, seeding its reach, and the actor's note
    /// of it. Nothing is written to the world until the commit.
    pub(crate) fn stage_event(&self, stage: &mut Stage, event: Event) -> Result<()> {
        let mut reach = Reach::default();
        reach.seed(&event, &self.state.sites)?;
        let (actor, id) = (stage.actor, event.id.clone());
        let mut staged = Staged { sim: self, stage };
        staged.add_note(actor, id.clone(), "sim:act", String::new(), None, id)?;
        stage.event = Some((event, reach));
        Ok(())
    }

    /// Writes an accepted act. Identities split as the whole copy split them
    /// before its first write: the actor alone unless it acts for its
    /// cohort, then the target.
    pub(crate) fn commit(&mut self, stage: Stage, next_action: u64) {
        let s = &mut self.state;
        if stage.count == 1 {
            s.population.lift(stage.actor).expect("the actor exists");
        }
        if let Some(target) = stage.target {
            s.population.lift(target).expect("the target exists");
        }
        for (id, body) in stage.bodies {
            let group = s.population.groups.get_mut(&id);
            group.expect("staged bodies were lifted").entity = body;
        }
        for child in stage.births {
            s.population
                .insert(child, 1)
                .expect("staging reserved the identity");
        }
        if let Some(site) = stage.site {
            s.sites.insert(stage.place, site);
        }
        for (relation, present) in stage.relations {
            if present {
                s.relations.insert(relation);
            } else {
                s.relations.remove(&relation);
            }
        }
        s.polities.extend(stage.polities);
        for (axis, value) in stage.marks {
            s.record.reckon(&[hagiograph::Entry {
                axis,
                value,
                holder: stage.actor,
            }]);
        }
        s.notes.extend(stage.notes);
        if let Some((event, reach)) = stage.event {
            s.reach.arrivals.extend(reach.arrivals);
            s.events.insert(event.id.clone(), event);
        }
        s.next_action = next_action;
    }
}

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
    fn actor(&mut self) -> &mut Entity {
        let actor = self.stage.actor;
        self.stage
            .bodies
            .get_mut(&actor)
            .expect("the actor is staged")
    }
    fn add_note(
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

    /// Applies one effect to the stage. Returns whether it set a feat.
    pub(crate) fn effect(&mut self, e: &Effect, cause: &str) -> Result<bool> {
        if let Some(done) = meaning::effect(self, e) {
            done?;
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
    fn shift(&mut self, key: &str, delta: i64) -> Result<()> {
        meaning::shift(&mut self.site()?.conditions, key, delta, 1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Execution, Founding, Simulation, rules::*, simulation::Outcome};
    use std::collections::BTreeMap;

    #[test]
    fn an_act_that_would_make_matter_is_refused_and_changes_nothing() {
        let founding = Founding {
            seed: 5,
            sites: 2,
            population: 8,
            cohort_size: 4,
            lineages: 1,
            ..Default::default()
        };
        let mut sim = Simulation::new(founding.generate().unwrap(), Execution::Grouped).unwrap();
        // Admission refuses an unbalanced transform, so this one enters the
        // rules after it. The note before it must not survive either.
        let mut conjure = sim.genesis.rules.processes["sim:remember"].clone();
        conjure.id = "test:conjure".into();
        conjure.effects.push(Effect::Transform {
            who: Binding::Actor,
            take: BTreeMap::new(),
            give: BTreeMap::from([("world:soil".into(), 1)]),
        });
        let rules = &mut std::sync::Arc::make_mut(&mut sim.genesis).rules;
        rules.processes.insert(conjure.id.clone(), conjure);
        let hash = sim.state_hash();
        let groups = sim.state.population.groups.len();
        let r = sim.execute(3, None, "test:conjure", None);
        let refusal = "matter invariant would be violated";
        assert_eq!(r.outcome, Outcome::Refused(refusal.into()));
        assert_eq!(
            (r.matter_before, r.matter_after),
            (sim.matter(), sim.matter())
        );
        assert_eq!(sim.state_hash(), hash);
        assert_eq!(sim.state.population.groups.len(), groups);
        assert!(sim.state.notes.is_empty() && sim.state.events.is_empty());
    }
}
