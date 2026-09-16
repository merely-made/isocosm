// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Letting go of the played body, for deep time. (D7a)
//!
//! Split out of `world.rs` at the 600-line ceiling. A crate-internal door,
//! not an intent: nobody plays a world through deep time, so there is no hand
//! to ask for it and nothing a trace should record.

use super::World;
use crate::places::Tier;

impl World {
    /// Lets go of the played body and leaves it living in its line.
    ///
    /// Deep time runs with no hand (isoscape family plan §2.4), so the body a
    /// world opened under becomes one more critter: its line takes its own
    /// adaptation turn and nothing is held.
    ///
    /// **Not a loss, so `control_lost` is left alone.** That field names the
    /// body a world *could no longer play* on its last tick, which only
    /// ineligibility sets in [`World::apply`], and the runtime's succession
    /// reads it as a death under the hand. A released body is alive and still
    /// eligible. When a hand was on it, the last tick left `control_lost` at
    /// `None`; the next tick recomputes it from `controlled` either way.
    /// `unlocked` and `frontier` stay too: letting go unearns nothing.
    pub(crate) fn release_control(&mut self) {
        self.controlled = None;
    }

    /// Sets every living organism to [`Tier::Far`].
    ///
    /// **Ruled by the project owner, 2026-09-16: everything runs the far
    /// tier during deep time.** With no hand there is no focus, so the
    /// ordinary tier update (`organism::ecology`, gated on
    /// `World::position`) never runs once control is released, and tiers
    /// would otherwise sit frozen wherever they stood at release —
    /// whoever was near the released body, and every line descended from
    /// them, running a different tier than the rest of the enclosure for
    /// the whole span. Offspring inherit a parent's tier
    /// (`organism::ecology::breeding::bear`), so this one reset is enough:
    /// nothing else can promote a body back to near while no focus exists.
    pub(crate) fn freeze_tiers_far(&mut self) {
        for organism in self.organisms.iter_mut().filter(|o| o.is_alive()) {
            organism.tier = Tier::Far;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Intent, World};

    #[test]
    fn releasing_control_is_not_a_loss() {
        let mut world = World::new(11, 24);
        world.apply(Intent::Idle);
        let body = world.controlled_id().expect("a played critter");
        let frontier = world.frontier();

        world.release_control();
        assert_eq!(world.controlled_id(), None);
        assert_eq!(world.control_lost(), None, "letting go is not a death");
        assert!(world.is_eligible(body), "the body is alive and playable");
        assert_eq!(world.frontier(), frontier, "and nothing is unearned");

        world.apply(Intent::Idle);
        assert_eq!(world.control_lost(), None, "and no later tick calls it one");
    }
}
