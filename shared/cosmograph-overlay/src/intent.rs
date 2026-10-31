// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::attention::AttentionChange;
use crate::handle::ParticipantHandle;
use crate::tick::Tick;

/// A participant's only way to reach the sim: an intent, stamped for the tick
/// it applies to and naming who submitted it (wing design record §5.2 point
/// 2). A game never writes sim state directly.
///
/// The participant is named because the sim checks that a player directs
/// only who they play (ruling 152) while two may direct one entity (ruling
/// 153), and because each participant has their own attention set (ruling
/// 204). Intents are the sim's only nondeterministic input, so a sequence of
/// these is also the replay log and the network protocol.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentEnvelope<I> {
    pub tick: Tick,
    pub participant: ParticipantHandle,
    pub intent: Intent<I>,
}

/// What an envelope carries: a change to the participant's attention set,
/// which every game shares, or the game's own vocabulary `I` (see
/// [`crate::mesocosm`] for the first).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent<I> {
    Attention(AttentionChange),
    Game(I),
}
