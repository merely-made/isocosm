// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::harm;

impl Doing<'_> {
    pub(super) fn wound(&mut self, who: Binding, cells: u64, slot: u8) -> Result<bool> {
        let at = *self
            .draws
            .get(&slot)
            .ok_or("the crowd did not draw the wound's part")?;
        let rules = self.rules;
        let tick = self.tick;
        let e = self.body(who)?;
        let part = harm::part_at(e, at).ok_or("a wound outside the body")?;
        let lineage = e.lineage.clone();
        let regrows = self
            .lineages
            .and_then(|l| l.get(&lineage))
            .is_some_and(|l| l.traits.contains(harm::FRAGMENT));
        let cells = u32::try_from(cells).map_err(|_| "wound exceeds cell range")?;
        let hit = harm::hurt(rules, self.body(who)?, part, cells, tick, regrows)?;
        for (key, n) in hit.wounded.spilled {
            self.give(Binding::Place, &key, n)?;
        }
        if let Some(fragment) = hit.fragment {
            self.born.push(Pending::Fragment {
                child: fragment.child,
            });
        }
        Ok(true)
    }

    pub(super) fn rot(&mut self, who: Binding, amount: u64) -> Result<bool> {
        let rules = self.rules;
        let moved = harm::rot::rot(self.body(who)?, rules, amount)?;
        for (_, ledger) in moved {
            for (key, n) in ledger {
                self.give(Binding::Place, &key, n)?;
            }
        }
        Ok(true)
    }
}
