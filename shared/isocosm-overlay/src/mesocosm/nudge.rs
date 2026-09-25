// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::handle::{EntityHandle, PlaceHandle};

/// The one directive a Mesocosm player gives: a click that draws the played
/// critter's attention to a place or a thing (rulings 176, 214). The critter
/// weighs it by its bond (ruling 177) and answers it by its own needs, senses
/// and mood. It never warns (ruling 215), and the standing orders, places to
/// range and make home, priorities and stances, are grown by the sim from the
/// history of these nudges and what came of them, never sent (ruling 216).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nudge {
    /// The entity the player plays, one critter or its kin directed whole
    /// (rulings 152, 155).
    pub entity: EntityHandle,
    pub target: NudgeTarget,
    pub meaning: NudgeMeaning,
}

/// What was clicked: a place, named as a place-graph node (ruling 205), or a
/// thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NudgeTarget {
    Place(PlaceHandle),
    Thing(EntityHandle),
}

/// What the click asks: attending by default, another meaning picked by right
/// click (ruling 214).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NudgeMeaning {
    /// Attend to this; what to do about it is the critter's.
    Attend,
    /// One of the critter's own acts toward the target, such as eating or
    /// carving. A reading, not a ruling: the alternatives are what the
    /// critter's biology can do with the target (ruling 59), and the critter
    /// still weighs the nudge.
    Act(ActKey),
}

/// An act named by opaque key. The vocabulary is the ruleset's, the same way
/// the sim names processes by string key (`isocosm::schema::Key`, not
/// depended on here).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActKey(pub String);
