// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::tick::Tick;

/// A game's only way to reach the sim: an intent, stamped for the tick it
/// applies to (wing design record §5.2 point 2). A game never writes sim
/// state directly.
///
/// Generic over the game's own intent vocabulary `I` (see [`crate::mesocosm`]
/// for the first one), so this envelope stays game-neutral. Intents are the
/// sim's only nondeterministic input, so a sequence of these is also the
/// replay log and the network protocol.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEnvelope<I> {
    pub tick: Tick,
    pub intent: I,
}
