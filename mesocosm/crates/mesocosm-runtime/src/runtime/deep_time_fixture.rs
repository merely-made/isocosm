// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Test-only fixture: a world that has already run several short epochs, the
//! same way `mesocosm-core/examples/deep_time_probe.rs` runs deep time. Split
//! out of `runtime.rs` at the 600-line ceiling; this whole module is
//! `#[cfg(test)]`, so nothing here reaches a non-test build.

use mesocosm_core::{History, Intent, World};

/// Direct `World::apply`/`History::record_all`, no runtime and no reckoning,
/// so what comes back is exactly "a world with a past" and nothing else has
/// touched it. Shortening the epoch keeps a debug build fast; shipped worlds
/// carry `DEFAULT_EPOCH_TICKS` (1,000). Stops the instant `epochs` closes, so
/// the world comes back standing at that boundary (`at_boundary()` true),
/// which is where deep time's own handover leaves it (isoscape family plan
/// §2.4).
pub(crate) fn deep_time_world(
    seed: u64,
    organisms: u32,
    epoch_ticks: u64,
    epochs: u64,
) -> (World, History) {
    let world = World::new(seed, organisms);
    let rules = world.rules();
    let mut world = world.with_rules(mesocosm_core::WorldRules {
        epoch: mesocosm_core::rules::EpochRule::Timed { ticks: epoch_ticks },
        ..rules
    });
    let mut history = History::new();
    while world.epoch < epochs {
        world.apply(Intent::Idle);
        history.record_all(world.drain_events());
    }
    (world, history)
}
