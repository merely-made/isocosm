// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A bitten part taken whole (rulings 468, 516 and 544). Where a bite would
//! take all of a part's tissue, the part lands whole on the eater at the
//! first free box its plan finds, as its affinity allows: a native part as
//! it was, an adapter's expressing nothing, and a refused one eaten as a
//! meal. It keeps its donor's matter, and the eater's lineage learns its
//! kind. A prey left with no living part is dead.

use super::*;
use crate::growth;

impl Staged<'_> {
    /// Takes `part` of the body `from` binds whole into the actor. Returns
    /// whether it did; a part with living children, a lineage without a
    /// recipe, a refused crossing or no seat leave it to be eaten.
    pub(super) fn incorporate(&mut self, from: Binding, part: Id) -> Result<bool> {
        let sim = self.sim;
        let Some(Holder::Entity(prey)) = self.holder(from) else {
            return Ok(false);
        };
        let actor = self.stage.actor;
        if prey == actor {
            return Ok(false);
        }
        let developed = |e: &Entity| {
            let l = sim.state.lineages.get(&e.lineage);
            l.and_then(|l| l.development.clone())
        };
        let bodies = &self.stage.bodies;
        let (eater, donor) = (&bodies[&actor], bodies.get(&prey).ok_or("prey missing")?);
        let (Some(mine), Some(theirs)) = (developed(eater), developed(donor)) else {
            return Ok(false);
        };
        let affinity = sim.genesis.rules.affinity.clone().unwrap_or_default();
        let Some(w) = growth::whole((eater, donor), part, (&mine, &theirs), &affinity) else {
            return Ok(false);
        };
        let mut donor = self.stage.bodies.remove(&prey).expect("found above");
        let id = growth::take_whole(self.actor(), &mut donor, part, &w);
        self.stage.bodies.insert(prey, donor);
        let id = id.ok_or("bitten part missing")?;
        if sim.flowing() {
            let moved = self.actor().parts[&id].matter.clone();
            for (key, n) in moved.into_iter().filter(|(_, n)| *n > 0) {
                self.stage.legs.push(Leg {
                    from: (Holder::Part(prey, part), key.clone()),
                    to: (Holder::Part(actor, id), key),
                    amount: n,
                });
            }
        }
        if let Some(kind) = w.kind {
            let eater = self.actor().lineage.clone();
            self.stage.lessons.push((eater, kind));
        }
        Ok(true)
    }
}
