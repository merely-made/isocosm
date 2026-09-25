// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// A tick of the sim's clock. Intents are stamped for one; a view and a
/// handoff outcome each name the one they belong to.
///
/// **Provisional, not ruled** (see the crate README's "Open forks"): a
/// newtype over the sim's own flat monotonic counter (`isocosm::schema::Tick
/// = u64`, not depended on here). Epoch boundaries are a rule over this
/// count, not a field of it, matching how the sim already derives them.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Tick(pub u64);

impl Tick {
    pub const ZERO: Tick = Tick(0);

    /// The next tick. Saturates rather than wrapping, since a wrapped clock
    /// is a silently corrupt replay.
    pub fn next(self) -> Tick {
        Tick(self.0.saturating_add(1))
    }
}
