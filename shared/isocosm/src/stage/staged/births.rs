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
    /// The next child's identity.
    fn next_child(&mut self) -> Result<Id> {
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
    fn relate(&mut self, id: Id) {
        let actor = self.stage.actor;
        let parent = Relation {
            subject: id,
            object: actor,
            kind: "sim:parent".into(),
        };
        let young = Relation {
            subject: actor,
            object: id,
            kind: "sim:child".into(),
        };
        self.stage.relations.extend([(parent, true), (young, true)]);
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
        let Some((bud, part)) = b.severed else {
            return Ok(false);
        };
        let id = self.next_child()?;
        if self.sim.flowing() {
            for (key, n) in part.matter.iter().filter(|(_, n)| **n > 0) {
                self.stage.legs.push(Leg {
                    from: (Holder::Part(self.stage.actor, bud), key.clone()),
                    to: (Holder::Part(id, 0), key.clone()),
                    amount: *n,
                });
            }
        }
        let soma = self.soma(&b.lineal, id);
        let tick = self.sim.state.tick;
        let child = births::seedling((self.actor(), &b.lineal), rules, (part, mark), soma, tick);
        self.relate(id);
        self.stage.births.push(child);
        Ok(true)
    }
}
