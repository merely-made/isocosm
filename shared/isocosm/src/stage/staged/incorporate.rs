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
        let body = self.body(from)?;
        let taken = body.parts.get(&part).ok_or("bitten part missing")?.clone();
        if body
            .parts
            .values()
            .any(|c| !c.severed && c.parent == Some(part))
        {
            return Ok(false);
        }
        let prey_lineage = body.lineage.clone();
        let eater_lineage = self.actor().lineage.clone();
        let developed = |l: &Key| {
            sim.state
                .lineages
                .get(l)
                .and_then(|l| l.development.clone())
        };
        let (Some(eater), Some(donor)) = (developed(&eater_lineage), developed(&prey_lineage))
        else {
            return Ok(false);
        };
        let affinity = sim.genesis.rules.affinity.clone().unwrap_or_default();
        let verdict = affinity.verdict(donor.domain, eater.domain);
        if verdict == Verdict::Refused {
            return Ok(false);
        }
        let Some((host, offset)) = growth::resolve(self.actor(), &eater.policy, taken.half_extent)
        else {
            return Ok(false);
        };
        // What it teaches is its kind in its donor's recipe.
        let kind = taken.situs.and_then(|[t, _, slot]| {
            let tagma = donor.recipe.tagmata.get(usize::from(t))?;
            if slot == 0 {
                Some(tagma.segment.clone())
            } else {
                tagma.bears.clone()
            }
        });
        let body = self.body(from)?;
        let mut moved = body.parts.remove(&part).expect("found above");
        if !body.parts.values().any(|q| !q.severed) {
            body.alive = false;
        }
        moved.parent = Some(host);
        moved.offset = offset;
        moved.situs = None;
        if verdict == Verdict::Adapter {
            moved.cells.clear();
            moved.functions.clear();
        }
        let eater_body = self.actor();
        let id = eater_body
            .parts
            .keys()
            .next_back()
            .map_or(0, |last| last + 1);
        if sim.flowing() {
            for (key, n) in moved.matter.iter().filter(|(_, n)| **n > 0) {
                self.stage.legs.push(Leg {
                    from: (Holder::Part(prey, part), key.clone()),
                    to: (Holder::Part(actor, id), key.clone()),
                    amount: *n,
                });
            }
        }
        self.actor().parts.insert(id, moved);
        if let Some(kind) = kind {
            self.stage.lessons.push((eater_lineage, kind));
        }
        Ok(true)
    }
}
