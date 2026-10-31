// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::handle::EntityHandle;

/// An act the player takes themselves, beside directing (ruling 202): not a
/// nudge the critter weighs, and not an answer to a checkpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAct {
    pub entity: EntityHandle,
    pub kind: PlayerActKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerActKind {
    /// Split the line `entity` is in and name the new one; the name is the
    /// doing (`mesocosm-core`'s `Speciate`).
    Speciate { name: String },
}
