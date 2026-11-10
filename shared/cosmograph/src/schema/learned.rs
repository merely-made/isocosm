// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a member has learned within its life (case 1051; mesocosm-overlay
//! M3.6): a small signed delta on its lineage's policy, one weight per
//! need the policy scores, in the policy's own 64ths.

use super::Tick;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A member's learned delta. The weights are as they stood at `settled`;
/// what has faded since is worked out when they are read (the rule's
/// `fade`), never kept. A member that has learned nothing, or whose delta
/// has faded away, carries none, so it saves and hashes as before.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Learned {
    /// The tick the weights stood at, always a birth or a step or a whole
    /// span of fading after one.
    pub settled: Tick,
    /// Each need's weight, by its place in the world's list of needs; never
    /// nought.
    pub weights: BTreeMap<u32, i32>,
}
