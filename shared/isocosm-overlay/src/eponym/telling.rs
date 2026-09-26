// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, Pointable};

/// A claim told by the played sophont to one hearer (ruling 87), in a
/// manner (ruling 241). What is told may be false, by deceit, honest mistake
/// or a retelling that changed it (ruling 117); whether it takes is the
/// hearer's, by what it can check, who is telling, what it wants to hear and
/// how it is told (ruling 118). A telling plants a note on the hearer or, at
/// a place, leaks into the reach field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Telling {
    /// The played sophont (ruling 152).
    pub from: EntityHandle,
    pub to: EntityHandle,
    pub claim: Claim,
    pub manner: Manner,
}

/// What is claimed: the thing it is about, and the version told as opaque
/// bytes in the sim's own vocabulary, the way an event record's payload is.
/// A true telling carries an event as the teller knows it; a false one
/// carries a version that never happened, which the sim keeps as a version
/// of the event (ruling 117).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    pub about: Pointable,
    pub as_told: Vec<u8>,
}

/// How a claim is told, the fourth thing a hearer weighs (rulings 118, 241):
/// plainly, or posed to intimidate, persuade or deceive. Posing is never an
/// intent of its own, and display and bluff in a contest are sizing up
/// (ruling 116), not a telling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Manner {
    Plain,
    Intimidate,
    Persuade,
    Deceive,
}
