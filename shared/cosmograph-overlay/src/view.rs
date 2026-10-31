// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::tick::Tick;

/// A read-only handle onto the sim's state at one tick (wing design record
/// §5.2 point 3): the renderer and the interface read through it while the
/// sim computes the next tick, so neither waits on the other.
///
/// The view's own content is each game's concern (its own crate, reading
/// through whatever native or component binding is live) and is not a type
/// in this crate; what crosses the contract is this handle, naming which
/// tick's snapshot and which generation of it, so a save or a replay log can
/// record which view a game acted on without carrying the view itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ViewHandle {
    pub tick: Tick,
    /// Distinguishes successive snapshots the sim publishes for the same
    /// tick, so a holder can tell a stale handle from a current one without
    /// comparing full state.
    pub generation: u64,
}
