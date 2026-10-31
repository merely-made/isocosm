// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::handle::EntityHandle;
use crate::tick::Tick;

/// An outcome a game settled itself, handed back to the sim in the sim's own
/// terms (wing design record §5.2 point 5): when an operation touches
/// something foregrounded, the sim asks the game to resolve it, and the
/// outcome that comes back through this envelope must pass the sim's
/// invariants — the accounts conserved — or the sim resolves it by rate
/// instead.
///
/// Generic over the game's own handoff vocabulary `O`, the same way
/// [`crate::IntentEnvelope`] is generic over intents. A game with nothing to
/// resolve itself (see [`crate::mesocosm::MesocosmHandoff`]) supplies an
/// uninhabited `O`, so the fact that nothing ever reaches its handoff is
/// checked by the type rather than only documented.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffEnvelope<O> {
    pub tick: Tick,
    pub subject: EntityHandle,
    pub outcome: O,
}
