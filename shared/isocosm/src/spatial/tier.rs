// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tier line's physical side (the execution plan's E5; D25, D26, D32).
//! Where a world has attention windows, near is attention's roots (the
//! played, pinned and cared-for, and every member posed in a camera's
//! frame) and those still inside the collection buffer; a member that is
//! neither gives up its pose and patch at the end of an advance, and from
//! then forages at site grain. A world without windows keeps every pose.

use crate::{Simulation, schema::*};

impl Simulation {
    /// Whether a member is near: every member is, in a world without
    /// attention windows; otherwise a root or one still buffered.
    pub fn near(&self, id: Id) -> bool {
        let w = &self.state.attention;
        w.is_empty() || w.foreground(id, 1) || w.buffered(self.state.tick).contains(&id)
    }

    /// Unplaces every posed member that is neither near nor buffered.
    pub(crate) fn unplace_far(&mut self) {
        let windows = &self.state.attention;
        if windows.is_empty() {
            return;
        }
        let buffered = windows.buffered(self.state.tick);
        let far: Vec<Id> = self
            .state
            .spatial_poses
            .keys()
            .copied()
            .filter(|id| !windows.foreground(*id, 1) && !buffered.contains(id))
            .collect();
        for id in far {
            if let Some(j) = &mut self.journal {
                let population = &self.state.population;
                if let Some((&first, _)) = population.groups.range(..=id).next_back() {
                    for key in [first, id, id.saturating_add(1)] {
                        j.group(key, population.groups.get(&key));
                    }
                }
            }
            self.state.spatial_poses.remove(&id);
            if let Ok(e) = self.state.population.lift(id) {
                e.patch = None;
            }
        }
    }
}
