// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, PlaceHandle};

/// The actuation of one body for one tick (wing design record §5.2 point 2
/// and §9.14; ruling 60): what the game-side motion and contact solver,
/// running over the stack's conatus, accepted for the body the participant
/// plays, crossing as the intent (ruling 233), with the timed acts the body
/// began. Never a second body (ruling 152). Input frames never cross; the
/// solver's accepted transition does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actuation {
    /// The one body the participant plays (rulings 60, 152).
    pub body: EntityHandle,
    pub motion: Motion,
    /// Timed acts begun this tick, in order. A blow's outcome does not travel
    /// here: the foreground resolves it and hands the harm back through the
    /// handoff (ruling 232).
    pub acts: Vec<TimedAct>,
}

/// The accepted transition of the body over one tick (ruling 233).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Motion {
    /// The place-graph node the body is in after the tick (ruling 205).
    pub at: PlaceHandle,
    /// Where in the site's volume the body stands after the tick, in the
    /// sim's voxel grid. A finer pose is the game's presentation and stays
    /// there.
    pub position: WorldPoint,
    /// Voxels fallen during the tick, if the accepted transition included a
    /// fall. The injury a fall does is the sim's (ruling 123); `eponym-world`'s
    /// `Fall` was a consequence of motion and never a player's intent.
    pub fell: u32,
}

/// A raw world coordinate `[x, y, z]` in the sim's voxel grid. The same shape
/// as Mesocosm's `WorldPoint`, defined again here rather than lifted, since a
/// game's overlay adds a sibling module and changes nothing in the core
/// (ruling 197); a core `WorldPoint` is a candidate change put to Mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldPoint(pub [i32; 3]);

/// A timed act the body begins, a strike, a brace, a grip, an anchor, a use,
/// named by the ruleset's opaque key, at a target if it has one. What the
/// body can do is what its anatomy affords (rulings 59, 96).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedAct {
    pub act: ActKey,
    pub target: Option<ActTarget>,
}

/// What a timed act is aimed at: a thing, or a place named as a place-graph
/// node (ruling 205).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActTarget {
    Thing(EntityHandle),
    Place(PlaceHandle),
}

/// An act named by opaque key; the vocabulary is the ruleset's, as the sim
/// names processes by string key. Mesocosm's `ActKey` is the same shape, kept
/// in its own module for the same reason as [`WorldPoint`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActKey(pub String);
