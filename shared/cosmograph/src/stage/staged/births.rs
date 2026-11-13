// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Births spend the provision (rulings 447, 448, 518, 524, 530 and 552). A
//! brood develops its lineage's whole recipe from a soma its own seed
//! draws; a clutch lays eggs, each the recipe's root alone, sharing the
//! provision; and a bud is the provision poured into a part grown at the
//! reproducing part, severing into a body of its own once it holds a
//! provision's worth, a seedling that grows the rest itself. A child's tissue is its
//! parent's provision moved, never spawned (TD6), and each move is in the
//! flow record.

use super::*;
use crate::{
    development,
    meaning::births::{self, Lineal},
};

impl Staged<'_> {
    fn spatial_birth(&mut self, child: Id) {
        let mark = crate::spatial::BirthMark {
            action: self.stage.action.clone(), process: self.stage.process.clone(), parent: self.stage.actor,
            child, ordinal: self.stage.birth_places.len() as u64,
            parent_alive: self.sim.body_at_start(self.stage.actor).is_some_and(|p| p.alive),
        };
        self.stage.birth_places.push(mark);
    }
    /// The next child's identity.
    pub(super) fn next_child(&mut self) -> Result<Id> {
        let (sim, born) = (self.sim, self.stage.births.len() as u64);
        if sim.state.population.count() + born >= sim.genesis.rules.limits.entities {
            return Err("population limit".into());
        }
        let id = sim.state.population.next_id + born;
        id.checked_add(1).ok_or("identity exhausted")?;
        Ok(id)
    }

    /// The child `id`'s parentage, both ways, so a parent's act can find
    /// its own young (526).
    pub(super) fn relate(&mut self, id: Id) {
        let actor = self.stage.actor;
        let parent = Relation {
            subject: id,
            object: actor,
            kind: "sim:parent".into(),
            value: 0,
        };
        let young = Relation {
            subject: actor,
            object: id,
            kind: "sim:child".into(),
            value: 0,
        };
        self.stage.relations.extend([(parent, true), (young, true)]);
    }

    /// The generic birth keeps its parent's form, but only the provision's
    /// matter. A bodied parent pays through its parts, never a second ledger.
    pub(super) fn provisioned_birth(&mut self, provision: &Ledger, scatter: Option<&Key>) -> Result<()> {
        let id = self.next_child()?;
        let tick = self.sim.state.tick;
        let rules = &self.sim.genesis.rules;
        // 821: cloning also retains the exact remaining journey.
        let mut child = self.actor().clone();
        child.accounts.clear();
        for part in child.parts.values_mut().filter(|p| !p.matter.is_empty()) {
            std::sync::Arc::make_mut(part).matter.clear();
        }
        crate::repertoire::rebody(rules, self.sim.state.lineages.get(&child.lineage), &mut child)?;
        child.born = tick;
        child.arrived = tick;
        child.visits.clear();
        child.skills.clear();
        child.provenance = Provenance::Born(child.lineage.clone());
        child.authored = None;
        child.learned = crate::directing::inherited(&child.learned, rules, tick);
        // A seeding lineage's young lie dormant (1193, 1197).
        let seed = rules.dormancy.as_ref().filter(|d| d.seeds.contains_key(&self.stage.process));
        if let Some(d) = seed {
            child.traits.insert(d.key.clone());
        }
        for (key, value) in provision {
            self.take(Binding::Actor, key, *value)?;
            let given = if crate::anatomy::root_store::allocation(&child, rules, key).is_some() {
                // A propagule banks its matter whole, where its upkeep reaches it.
                match seed {
                    Some(_) => crate::anatomy::root_store::bank(&mut child, rules, key, *value),
                    None => crate::anatomy::root_store::initialize(&mut child, rules, key, *value),
                }
            } else { crate::anatomy::give(&mut child, rules, key, *value) };
            if let Some(given) = given {
                for (part, amount) in given? {
                    if self.sim.flowing() {
                        self.stage.legs.push(Leg {
                            from: (Holder::Entity(self.stage.actor), key.clone()),
                            to: (Holder::Part(id, part), key.clone()),
                            amount,
                        });
                    }
                }
            } else {
                credit(&mut child.accounts, key, *value)?;
                if self.sim.flowing() && meaning::matter(rules, key) {
                    self.stage.legs.push(Leg {
                        from: (Holder::Entity(self.stage.actor), key.clone()),
                        to: (Holder::Entity(id), key.clone()),
                        amount: *value,
                    });
                }
            }
        }
        if let Some(to) = scatter.and_then(|medium| self.scattered(id, medium)) {
            child.place = to;
            child.patch = None;
        }
        self.relate(id);
        self.spatial_birth(id);
        self.stage.births.push(child);
        Ok(())
    }

    /// Where the medium carries the child `id`: a neighbouring site, one of
    /// the parent's site's routes drawn, with the chance out of a hundred
    /// that site holds the medium as the pass began; None beside its parent.
    fn scattered(&self, id: Id, medium: &Key) -> Option<Id> {
        let sim = self.sim;
        let site = sim.body_at_start(self.stage.actor)?.resident_site()?;
        let s = sim.site_at_start(site)?;
        let chance = s.conditions.get(medium).map_or(0, |v| (*v).clamp(0, 100) as u64);
        let seed = sim.genesis.dynamics_seed();
        if s.routes.is_empty() || crate::draw(seed, "scatter", &[id]) % 100 >= chance {
            return None;
        }
        let k = crate::draw(seed, "scatter-route", &[id]) % s.routes.len() as u64;
        Some(s.routes[k as usize].to)
    }

    /// The soma the child `id` draws from `l`'s recipe by its own seed.
    fn soma(&self, l: &Lineal, id: Id) -> development::Soma {
        let seed = crate::draw(self.sim.genesis.dynamics_seed(), "soma", &[id]);
        development::soma(&self.sim.genesis.rules, &l.d.recipe, seed)
    }

    /// A brood, or a clutch of eggs, from the whole provision, each child
    /// carrying `young`.
    pub(super) fn bear(&mut self, clutch: bool, young: Option<&Key>) -> Result<()> {
        let rules = &self.sim.genesis.rules;
        let (l, shares) = births::bear(self, rules, clutch)?;
        let tick = self.sim.state.tick;
        for share in shares {
            let id = self.next_child()?;
            let soma = self.soma(&l, id);
            let parent = &*self.actor();
            let (child, given) =
                births::hatch((parent, &l), rules, soma, (clutch, share), (tick, young))?;
            self.relate(id);
            if self.sim.flowing() {
                for (part, n) in given {
                    self.stage.legs.push(Leg {
                        from: (Holder::Entity(self.stage.actor), l.provision.clone()),
                        to: (Holder::Part(id, part), l.tissue.clone()),
                        amount: n,
                    });
                }
            }
            self.spatial_birth(id);
            self.stage.births.push(child);
        }
        Ok(())
    }

    /// Pours the provision into the bud, growing one where there is none,
    /// and severs it once it holds a provision's worth. Returns whether it
    /// severed.
    pub(super) fn bud(&mut self, mark: &Key) -> Result<bool> {
        let rules = &self.sim.genesis.rules;
        let b = births::bud(self, rules, mark)?;
        let actor = Holder::Entity(self.stage.actor);
        if b.pour > 0 && self.sim.flowing() {
            let poured = [(b.lineal.tissue.clone(), b.pour)];
            self.stage
                .legs
                .extend(flows::poured(actor, b.taken, poured));
        }
        let Some((bud, (frame, part))) = b.severed else {
            return Ok(false);
        };
        self.guard_owned_severed(&part.matter)?;
        let id = self.next_child()?;
        if self.sim.flowing() {
            for (key, n) in part.matter.iter().filter(|(_, n)| **n > 0) {
                self.stage.legs.push(Leg {
                    from: (Holder::Part(self.stage.actor, bud), key.clone()),
                    to: (Holder::Part(id, PartId(0)), key.clone()),
                    amount: *n,
                });
            }
        }
        let soma = self.soma(&b.lineal, id);
        let tick = self.sim.state.tick;
        let child = births::seedling(
            (self.actor(), &b.lineal),
            rules,
            ((frame, part), mark),
            soma,
            tick,
        );
        self.relate(id);
        self.spatial_birth(id);
        self.stage.births.push(child);
        Ok(true)
    }
}
