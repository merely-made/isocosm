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
use crate::{anatomy, development, growth, rules::expressing};

impl Staged<'_> {
    /// The actor's lineage's development and its own accounts.
    fn lineage(&mut self) -> Result<(Development, anatomy::Own)> {
        let lineage = self.actor().lineage.clone();
        let d = self.development(&lineage)?;
        Ok((d, anatomy::own(&self.sim.genesis.rules, &lineage)))
    }

    /// The next child's identity and its seed's soma.
    fn next_child(&mut self) -> Result<Id> {
        let (sim, born) = (self.sim, self.stage.births.len() as u64);
        if sim.state.population.count() + born >= sim.genesis.rules.limits.entities {
            return Err("population limit".into());
        }
        let id = sim.state.population.next_id + born;
        id.checked_add(1).ok_or("identity exhausted")?;
        Ok(id)
    }

    /// A child `id` of the actor with `parts`, the soma it drew, nothing
    /// held, and its parentage.
    fn child(&mut self, id: Id, parts: BTreeMap<Id, Part>, soma: Vec<u8>) -> Entity {
        let tick = self.sim.state.tick;
        let actor = self.stage.actor;
        let mut child = self.actor().clone();
        child.parts = parts;
        child.soma = soma;
        child.accounts = Ledger::new();
        child.born = tick;
        child.arrived = tick;
        child.visits.clear();
        child.skills = BTreeMap::new();
        child.provenance = Provenance::Born(child.lineage.clone());
        // Both ways, so a parent's act can find its own young (526).
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
        child
    }

    /// The soma the child `id` draws from `d`'s recipe by its own seed.
    fn soma(&self, d: &Development, id: Id) -> development::Soma {
        let seed = crate::draw(self.sim.genesis.dynamics_seed(), "soma", &[id]);
        development::soma(&self.sim.genesis.rules, &d.recipe, seed)
    }

    /// A brood, or a clutch of eggs, from the whole provision.
    pub(super) fn bear(&mut self, clutch: bool) -> Result<()> {
        let (d, own) = self.lineage()?;
        let (Some(provision), Some(tissue)) = (own.provision, own.tissue) else {
            return Err("a lineage without a provision bears nothing".into());
        };
        let amount = self.held(Binding::Actor, &provision)?;
        if amount == 0 {
            return Err("nothing provisioned".into());
        }
        self.take(Binding::Actor, &provision, amount)?;
        let eggs = if clutch { u64::from(d.clutch) } else { 1 };
        let (base, extra) = (amount / eggs, amount % eggs);
        for k in 0..eggs {
            let share = base + u64::from(k < extra);
            if share == 0 {
                continue;
            }
            let id = self.next_child()?;
            let soma = self.soma(&d, id);
            let rules = &self.sim.genesis.rules;
            let mut parts = development::develop(rules, &d, &soma)?;
            if clutch {
                parts.retain(|_, p| p.situs == Some([0, 0, 0]));
            }
            let mut child = self.child(id, parts, soma.segments);
            let given = anatomy::give(&mut child, rules, &tissue, share)
                .ok_or("a child with no parts")??;
            if self.sim.flowing() {
                for (part, n) in given {
                    self.stage.legs.push(Leg {
                        from: (Holder::Entity(self.stage.actor), provision.clone()),
                        to: (Holder::Part(id, part), tissue.clone()),
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
        let (d, own) = self.lineage()?;
        let (Some(provision), Some(tissue)) = (own.provision, own.tissue) else {
            return Err("a lineage without a provision buds nothing".into());
        };
        let rules = &self.sim.genesis.rules;
        let amount = self.held(Binding::Actor, &provision)?;
        if amount == 0 {
            return Err("nothing provisioned".into());
        }
        let actor = self.actor();
        let marked = actor
            .parts
            .iter()
            .find(|(_, p)| !p.severed && p.traits.contains(mark));
        let bud = match marked.map(|(id, _)| *id) {
            Some(bud) => bud,
            None => {
                let host = expressing(actor, anatomy::REPRODUCE).ok_or("nothing reproduces")?;
                let root = &d.recipe.tagmata[0].segment;
                let kind = rules.kinds.get(root).ok_or("an unknown root kind")?;
                let offset = growth::seat(actor, &d.policy, host, kind.half_extent, None)
                    .ok_or("no room to bud")?;
                let id = actor.parts.keys().next_back().map_or(0, |last| last + 1);
                let cells: BTreeMap<Key, u32> = kind
                    .cells
                    .iter()
                    .filter(|(_, n)| **n > 0)
                    .map(|(f, n)| (f.clone(), *n))
                    .collect();
                let part = Part {
                    parent: Some(host),
                    traits: BTreeSet::from([mark.clone()]),
                    shape: kind.shape.clone(),
                    functions: cells.keys().cloned().collect(),
                    half_extent: kind.half_extent,
                    offset,
                    cells,
                    ..Default::default()
                };
                actor.parts.insert(id, part);
                id
            },
        };
        // A provision's worth (552): what the parent's reproduce cells hold,
        // or the bud's adult mass where that is less.
        let parent = self.actor();
        let worth: u64 = parent
            .parts
            .iter()
            .filter(|(id, p)| **id != bud && !p.severed)
            .map(|(_, p)| anatomy::bound(p, rules, &provision))
            .sum();
        let part = &parent.parts[&bud];
        let full = anatomy::ceiling(part, rules.body()).min(worth);
        let need = full.saturating_sub(part.matter.get(&tissue).copied().unwrap_or(0));
        // Its parts full, the body's room for tissue is the bud's; what the
        // bud does not need stays provisioned.
        let pour = amount
            .min(need)
            .min(anatomy::room(self.actor(), rules, &tissue));
        if pour > 0 {
            let from = std::slice::from_ref(&provision);
            let digested = Conversion::Digestion;
            let taken =
                meaning::convert(self, rules, Binding::Actor, (from, &tissue), pour, digested)?;
            if self.sim.flowing() {
                let actor = Holder::Entity(self.stage.actor);
                self.stage
                    .legs
                    .extend(flows::poured(actor, taken, [(tissue.clone(), pour)]));
            }
        }
        let part = &self.actor().parts[&bud];
        if full == 0 || part.matter.get(&tissue).copied().unwrap_or(0) < full {
            return Ok(false);
        }
        let mut part = self.actor().parts.remove(&bud).expect("found above");
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
        part.parent = None;
        part.offset = [0; 3];
        part.traits.remove(mark);
        part.situs = Some([0, 0, 0]);
        let soma = self.soma(&d, id);
        let child = self.child(id, BTreeMap::from([(0, part)]), soma.segments);
        self.stage.births.push(child);
        Ok(true)
    }
}
