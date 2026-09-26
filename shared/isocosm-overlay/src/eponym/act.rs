// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, Pointable};

/// An act the player takes themselves, beside driving: not the body's
/// actuation, not an ask a peer weighs, and not an answer to a checkpoint.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerAct {
    /// The played sophont doing it (ruling 152).
    pub subject: EntityHandle,
    pub kind: PlayerActKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerActKind {
    /// Name the creature one lives, or what it meets; the name is the doing,
    /// a sapient's own act (rulings 36, 168). `eponym-world`'s `Name`.
    Name { of: Pointable, name: String },
    /// Write a note in the world: a bearer that can be found, read, lost or
    /// stolen like any other (rulings 127, 130), its text freeform, djot
    /// preferred (ruling 82). In survival mode it is part of what the
    /// creature one lives knows (ruling 187).
    Note {
        about: Option<Pointable>,
        text: String,
    },
}
