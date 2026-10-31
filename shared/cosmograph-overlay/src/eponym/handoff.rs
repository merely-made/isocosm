// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{ActKey, EntityHandle, Harm};

/// Eponym's handoff vocabulary, inhabited by ruling 232: the foreground
/// resolves each blow geometrically, its strike system reading a swept
/// volume against a body's parts, and hands the harm back in the sim's
/// terms. What comes back must pass the sim's invariants and agree with the
/// sim's own fight in distribution, checked on the bench (rulings 114, 123,
/// 154); the record's §3.8 has the sim never resolve a single blow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EponymHandoff {
    Blow(Blow),
    /// Harm the game settled with no blow behind it, a fall's or a
    /// scene's, to the envelope's subject (794).
    Hurt(Harm),
    /// Items the envelope's subject wore on a part it lost, fallen where it
    /// stands: where an item is worn is the game's (ruling 776).
    Dropped(Vec<EntityHandle>),
}

/// One blow, resolved: who struck, whom, with what act, and the harm done in
/// the sim's terms (ruling 123).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blow {
    pub by: EntityHandle,
    pub target: EntityHandle,
    pub act: ActKey,
    pub harm: Harm,
}
