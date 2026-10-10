// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::harm;

impl Staged<'_> {
    pub(super) fn wound(&mut self, who: Binding, cells: u64, slot: u8) -> Result<()> {
        let sim = self.sim;
        let total = harm::cells(self.body(who)?);
        if total == 0 {
            return Err("a body with no cells to wound".into());
        }
        let at = self.compute(|_, draw, _| draw(total, slot))?;
        let e = self.body(who)?;
        let part = harm::part_at(e, at).ok_or("a wound outside the body")?;
        let regrows = sim
            .state
            .lineages
            .get(&e.lineage)
            .is_some_and(|l| l.traits.contains(harm::FRAGMENT));
        let cells = u32::try_from(cells).map_err(|_| "wound exceeds cell range")?;
        let hit = harm::hurt(&sim.genesis.rules, e, part, cells, sim.state.tick, regrows)?;
        let parent = sim.bound(self.stage.actor, self.stage.target, who)?;
        for (key, n) in &hit.wounded.spilled {
            credit(&mut self.site()?.accounts, key, *n)?;
        }
        if sim.flowing() {
            self.stage
                .legs
                .extend(harm::spilled_legs(parent, self.stage.place, &hit.wounded));
        }
        if let Some(fragment) = hit.fragment {
            let id = self.next_child()?;
            if fragment.child.alive {
                self.stage
                    .relations
                    .extend(harm::parentage(parent, id).map(|r| (r, true)));
            }
            if sim.flowing() {
                self.stage.legs.extend(fragment.legs(parent, id));
            }
            self.stage.births.push(fragment.child);
        }
        Ok(())
    }

    pub(super) fn rot(&mut self, who: Binding, amount: u64) -> Result<()> {
        let sim = self.sim;
        let moved = harm::rot::rot(self.body(who)?, &sim.genesis.rules, amount)?;
        harm::rot::deposit(&mut self.site()?.accounts, &moved)?;
        if sim.flowing() {
            let entity = sim.bound(self.stage.actor, self.stage.target, who)?;
            self.stage
                .legs
                .extend(harm::rot::legs(entity, self.stage.place, &moved));
        }
        Ok(())
    }
}
