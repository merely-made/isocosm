// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One act's writes, held apart from the world until the act is accepted.
//! The interpreter once staged each act on a copy of the whole simulation.
//! A stage copies only what the act binds, its actor's and target's bodies
//! and its site, as its pass began (ruling 454), and lists what the act
//! adds: children, relations, notes, polities, record marks and its event.
//! Committing lands what the act changed of them; dropping the stage leaves
//! the world as it was, identity grouping included.

use crate::{
    Result,
    flows::{Holder, Leg, MadeBy},
    meaning::mass,
    reach::Reach,
    schedule::{Demands, Shares, fits, merge, merge_site},
    schema::*,
    simulation::Simulation,
};
use std::collections::BTreeMap;

mod staged;
#[cfg(test)]
mod tests;

pub(crate) use staged::Staged;

pub(crate) struct Stage {
    actor: Id,
    target: Option<Id>,
    place: Id,
    /// The actor's part the act binds (ruling 338).
    part: Option<PartId>,
    /// The members the actor stands for: one, or a whole bulk cohort.
    count: u64,
    /// The bound bodies as the act leaves them. A target that is the actor
    /// is the same body.
    bodies: BTreeMap<Id, Entity>,
    /// The site's ledger and conditions, copied on the first write.
    site: Option<Site>,
    births: Vec<Entity>,
    /// Kinds lineages learned by what their bodies took in (ruling 468).
    lessons: Vec<(Key, Key)>,
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
    /// The act's matter moves, for the flow record (ruling 345).
    legs: Vec<Leg>,
    /// Where its takes and gives of a body's own matter went among the
    /// body's parts (ruling 504), which split those legs.
    pub(crate) routed: Vec<crate::flows::Routed>,
    /// The number the act acts as, which keys its draws.
    act: u64,
    /// While a pass plans, what the act would take of shared ground.
    pub(crate) demands: Option<Demands>,
    /// What the act may take of shared ground, as its pass shared it out.
    pub(crate) shares: Option<Shares>,
    /// The values the act kept for its later effects.
    pub(crate) kept: BTreeMap<Key, i64>,
    /// What a carriage lets each bound body's parts still take (581), and
    /// the part the act's bite landed on.
    pub(crate) caps: BTreeMap<Id, crate::anatomy::Caps>,
    pub(crate) bitten: Option<PartId>,
}

impl Simulation {
    /// A stage for one act by `actor`, standing for `count` members at
    /// `place`, binding `part` of the actor, acting as number `act`. A named
    /// target that does not exist fails here, as lifting it from a whole
    /// copy would.
    pub(crate) fn stage(
        &self,
        (actor, target): (Id, Option<Id>),
        place: Id,
        part: Option<PartId>,
        count: u64,
        act: u64,
    ) -> Result<Stage> {
        // A cohort acts only through processes without targets.
        debug_assert!(count == 1 || target.is_none());
        let mut bodies = BTreeMap::new();
        let body = self.body_at_start(actor).ok_or("unknown entity")?;
        bodies.insert(actor, body.clone());
        if let Some(target) = target {
            let body = self.body_at_start(target).ok_or("unknown entity")?;
            bodies.entry(target).or_insert_with(|| body.clone());
        }
        Ok(Stage {
            actor,
            target,
            place,
            part,
            count,
            bodies,
            site: None,
            births: vec![],
            lessons: vec![],
            relations: vec![],
            notes: vec![],
            polities: vec![],
            marks: vec![],
            highs: BTreeMap::new(),
            event: None,
            legs: vec![],
            routed: vec![],
            act,
            demands: None,
            shares: None,
            kept: BTreeMap::new(),
            caps: BTreeMap::new(),
            bitten: None,
        })
    }

    /// Whether what the act changed lands whole on bodies and a site that
    /// earlier acts of its pass changed too.
    pub(crate) fn fits(&self, stage: &Stage) -> bool {
        let Some(frame) = &self.frame else {
            return true;
        };
        let bodies = stage.bodies.iter().all(|(id, body)| {
            if !frame.changed(Holder::Entity(*id)) {
                return true;
            }
            let base = frame.body(*id).expect("a changed body was kept");
            let live = self.state.population.get(*id).expect("staged bodies exist");
            let part = |id: &PartId| {
                // A part another act of the pass removed stays removed, so
                // the act may not have changed it.
                if base.parts.contains_key(id) && !live.parts.contains_key(id) {
                    return body.parts.get(id) == base.parts.get(id);
                }
                let matter = |e: &Entity| {
                    e.parts
                        .get(id)
                        .map(|p| p.matter.clone())
                        .unwrap_or_default()
                };
                fits(&matter(live), &matter(base), &matter(body))
            };
            // A part the act removed, to an eater or a child, takes its
            // matter with it: it lands only as the pass found it.
            let same = |a: &Ledger, b: &Ledger| {
                let held = |l: &Ledger| l.values().filter(|v| **v > 0).count();
                held(a) == held(b) && a.iter().all(|(k, v)| b.get(k).copied().unwrap_or(0) == *v)
            };
            let removed = base
                .parts
                .iter()
                .filter(|(id, _)| !body.parts.contains_key(*id))
                .all(|(id, was)| {
                    live.parts
                        .get(id)
                        .is_some_and(|now| same(&now.matter, &was.matter))
                });
            // A document another act reshaped while this one did too does
            // not land (674).
            let doc = live.body == base.body || body.body == base.body;
            fits(&live.accounts, &base.accounts, &body.accounts)
                && body.parts.keys().all(part)
                && removed
                && doc
        });
        let site = stage.site.as_ref().is_none_or(|site| {
            let Some(base) = frame.site(stage.place) else {
                return true;
            };
            let live = &self.state.sites[&stage.place];
            fits(&live.accounts, &base.accounts, &site.accounts)
        });
        bodies && site
    }

    /// Whether the staged act would change the world's matter: the bodies
    /// and site it binds, weighed before and after, and any children.
    pub(crate) fn moves_matter(&self, stage: &Stage) -> bool {
        let weigh = |e: &Entity| mass(&crate::anatomy::books(e), &self.genesis.rules);
        let (mut before, mut after) = (0u128, 0u128);
        for (id, body) in &stage.bodies {
            let weight = u128::from(if *id == stage.actor { stage.count } else { 1 });
            let was = self.body_at_start(*id).expect("staged bodies exist");
            before += weigh(was) * weight;
            after += weigh(body) * weight;
        }
        if let Some(site) = &stage.site {
            let was = self.site_at_start(stage.place).expect("staged sites exist");
            before += mass(&was.accounts, &self.genesis.rules);
            after += mass(&site.accounts, &self.genesis.rules);
        }
        after += stage.births.iter().map(weigh).sum::<u128>();
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

    /// The stored groups an act can change, by first identity: each lifted
    /// member's group and the pieces lifting leaves, the cohort that acts as
    /// one, and the children's new groups.
    fn reaches(&self, stage: &Stage) -> Vec<Id> {
        let groups = &self.state.population.groups;
        let mut firsts = vec![stage.actor];
        // Lifting a member out of its group rewrites the group's first entry
        // and adds entries for the member and for the rest of the group.
        let mut lifted = |id: Id| {
            if let Some((&first, group)) = groups.range(..=id).next_back() {
                firsts.push(first);
                firsts.push(id);
                if id + 1 < first + group.count {
                    firsts.push(id + 1);
                }
            }
        };
        if stage.count == 1 {
            lifted(stage.actor);
        }
        if let Some(target) = stage.target {
            lifted(target);
        }
        let next = self.state.population.next_id;
        firsts.extend((0..stage.births.len() as u64).map(|k| next + k));
        firsts.sort_unstable();
        firsts.dedup();
        firsts
    }

    /// Writes an accepted act of `process`, and its matter moves to the flow
    /// record. Identities split as the whole copy split them before its
    /// first write: the actor alone unless it acts for its cohort, then the
    /// target.
    pub(crate) fn commit(&mut self, mut stage: Stage, next_action: u64, process: &str) {
        let legs = std::mem::take(&mut stage.legs);
        let legs = crate::flows::split(legs, std::mem::take(&mut stage.routed));
        let count = stage.count;
        let reached = if self.targets.is_some() || self.filed.is_some() || self.journal.is_some() {
            self.reaches(&stage)
        } else {
            vec![]
        };
        if let Some(pass) = &mut self.pass {
            let population = &self.state.population;
            if stage.count == 1 {
                pass.lifting(population, stage.actor);
            }
            if let Some(target) = stage.target {
                pass.lifting(population, target);
            }
        }
        if let Some(j) = &mut self.journal {
            let s = &self.state;
            j.groups(&s.population, &reached);
            if stage.site.is_some() {
                j.site(stage.place, &s.sites[&stage.place]);
            }
            for (relation, _) in &stage.relations {
                j.relation(relation, s.relations.contains(relation));
            }
            for (id, _) in &stage.polities {
                j.polity(*id);
            }
            if !stage.marks.is_empty() {
                j.record(&s.record);
            }
            if let Some((event, _)) = &stage.event {
                j.event(&event.id);
            }
        }
        self.write(stage, next_action);
        self.flowed(MadeBy::Process(process.into()), legs, count);
        if let Some(t) = &mut self.targets {
            t.touch(&self.state.population, reached.iter().copied());
        }
        if let Some(f) = &mut self.filed {
            f.touch(&self.state.population, reached, self.state.tick);
        }
    }

    fn write(&mut self, stage: Stage, next_action: u64) {
        // Under a frame, what each body and the site were as the pass
        // began, which the act's changes are measured from and the rest of
        // the pass reads; without one, the act's copies replace them.
        let (bases, site_base) = match &self.frame {
            Some(_) => {
                let bases: Vec<Entity> = stage
                    .bodies
                    .keys()
                    .map(|id| {
                        self.body_at_start(*id)
                            .expect("staged bodies exist")
                            .clone()
                    })
                    .collect();
                let site_base = stage.site.as_ref().map(|_| {
                    let site = self.site_at_start(stage.place);
                    site.expect("staged sites exist").clone()
                });
                (Some(bases), site_base)
            },
            None => (None, None),
        };
        if let (Some(frame), Some(bases)) = (&mut self.frame, &bases) {
            for (id, base) in stage.bodies.keys().zip(bases) {
                let span = if *id == stage.actor { stage.count } else { 1 };
                frame.keep_body(*id, span, base);
            }
            if let Some(base) = &site_base {
                frame.keep_site(stage.place, base);
            }
        }
        let s = &mut self.state;
        if stage.count == 1 {
            s.population.lift(stage.actor).expect("the actor exists");
        }
        if let Some(target) = stage.target {
            s.population.lift(target).expect("the target exists");
        }
        for (k, (id, body)) in stage.bodies.into_iter().enumerate() {
            let group = s.population.groups.get_mut(&id);
            let live = &mut group.expect("staged bodies were lifted").entity;
            match &bases {
                Some(bases) => merge(live, &bases[k], body),
                None => *live = body,
            }
        }
        for child in stage.births {
            s.population
                .insert(child, 1)
                .expect("staging reserved the identity");
        }
        if let Some(site) = stage.site {
            let live = s.sites.get_mut(&stage.place).expect("staged sites exist");
            match &site_base {
                Some(base) => merge_site(live, base, site),
                None => *live = site,
            }
        }
        for (relation, present) in stage.relations {
            if present {
                s.relations.insert(relation);
            } else {
                s.relations.remove(&relation);
            }
        }
        for (lineage, kind) in stage.lessons {
            let learned = s
                .lineages
                .get_mut(&lineage)
                .and_then(|l| l.development.as_mut());
            if let Some(d) = learned {
                d.lexicon.insert(kind);
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
