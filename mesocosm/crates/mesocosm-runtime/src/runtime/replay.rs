// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-free reproduction: a seed, an organism count and a trace reach the
//! same world and the same past every time. Split out of `runtime.rs` at the
//! 600-line ceiling.

use mesocosm_core::{History, Intent, World};

use super::{Runtime, reckon_if_ended};
use crate::readings::FlowWindows;

impl Runtime {
    /// Rebuilds a run from a seed and trace, without any host at all. Two hosts
    /// agree exactly when their traces replay to the same hash here.
    ///
    /// Returns the past as well as the world, because it has to: a driven run
    /// drains its events every tick and a replay that did not would end holding
    /// a tick of undrained ones, which is a difference in the snapshot and so a
    /// difference in the hash. Reproducing the history rather than discarding it
    /// is also the claim that keeps it out of the snapshot, made executable.
    pub fn replay(seed: u64, organisms: u32, trace: &[Intent]) -> (World, History) {
        let replayed = Self::replayed(seed, organisms, trace);
        (replayed.world, replayed.history)
    }

    /// The same replay, keeping the readings it rebuilt.
    ///
    /// **The done-condition made runnable.** The windows are not in the
    /// snapshot, so nothing forces them to agree; reducing the replay's own
    /// streams through the same reducer and comparing the encodings is what
    /// shows that a replayed run reads the same as the run it replays.
    pub fn replayed(seed: u64, organisms: u32, trace: &[Intent]) -> Replayed {
        let mut world = World::new(seed, organisms);
        let mut history = History::new();
        let mut readings = FlowWindows::new();
        let mut epoch_seen = 0;
        for intent in trace {
            world.apply(intent.clone());
            let events = world.drain_events();
            readings.absorb(&events, &world.drain_flows());
            history.record_all(events);
            // The same reckoning a driven run does, through the same function.
            // The world ends its own epochs, so a replay reaches every boundary
            // the run did; skipping the reckoning here would leave the replayed
            // world's record short and its hash different. (PE3)
            reckon_if_ended(&mut world, &history, &mut epoch_seen);
        }
        Replayed {
            world,
            history,
            readings,
        }
    }

    /// The receipt a host probe compares: what this run was, and where it
    /// ended up.
    pub fn receipt(&self) -> Receipt {
        Receipt {
            seed: self.seed,
            organisms: self.organisms,
            // The trace is one entry per step actually applied, so it is the
            // step count whether the run was clocked or stepped by hand — and
            // it stays honest across a checkpoint, where the clock may have
            // authorised steps the held world never took.
            steps: self.trace.len() as u64,
            state_hash: self.state_hash(),
        }
    }
}

/// What a replay reproduced: the world, its past, and its readings.
pub struct Replayed {
    pub world: World,
    pub history: History,
    pub readings: FlowWindows,
}

/// A run's identity, for comparing hosts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    pub state_hash: u64,
}
