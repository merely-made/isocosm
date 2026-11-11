// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a member has learned within its life (case 1051; mesocosm-overlay
//! M3.6): a small signed delta on its lineage's policy, one weight per
//! need the policy scores and one per kind of act, in the policy's own
//! 64ths, and the act whose payoff it still waits on.

use super::{Key, Tick};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A member's learned delta. The weights are as they stood at `settled`;
/// what has faded since is worked out when they are read (the rule's
/// `fade`), never kept. A member that has learned nothing, whose delta has
/// faded away and that waits on nothing carries none, so it saves and
/// hashes as before.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Learned {
    /// The tick the weights stood at, always a birth or a step or a whole
    /// span of fading after one.
    pub settled: Tick,
    /// Each need's weight, by its place in the world's list of needs; never
    /// nought.
    pub weights: BTreeMap<u32, i32>,
    /// Each kind of act's weight, by process; never nought. It scores an
    /// act the needs do not, as a move.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub acts: BTreeMap<Key, i32>,
    /// The act that did not serve at once and may yet earn credit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending: Option<Pending>,
}

/// An act that ended without serving its member, waiting to see whether a
/// later act does within the rule's `horizon`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pending {
    pub act: Key,
    /// The tick the act ended.
    pub since: Tick,
}

impl Learned {
    /// Whether it holds nothing at all, so a member should carry none.
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty() && self.acts.is_empty() && self.pending.is_none()
    }
}
