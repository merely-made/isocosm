// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A member's learned delta as it stands now (case 1051; M3.6), for a
//! display to show. Read only: fading is worked out here, from the ticks
//! since the delta was settled, and nothing is written, so reading it at
//! any moment leaves the state and its hash as they were.

use crate::{schema::*, simulation::Simulation};

/// One member's delta at a reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delta {
    pub member: Id,
    pub at: Tick,
    /// Each need it weighs, by its place in the world's list of needs, in
    /// the policy's 64ths, faded to `at`; never nought, none where it has
    /// learned nothing.
    pub weights: Vec<(usize, i32)>,
}

/// `member`'s delta now, or none where it is absent, or the world keeps no
/// learning.
pub fn learned(sim: &Simulation, member: Id) -> Option<Delta> {
    let e = sim.state().population.get(member)?;
    super::super::learn::rule(&sim.genesis().rules)?;
    let weights = sim
        .learned_of(e)
        .map(|k| k.weights.into_iter().map(|(k, w)| (k as usize, w)).collect())
        .unwrap_or_default();
    Some(Delta { member, at: sim.state().tick, weights })
}
