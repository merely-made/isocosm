// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::handle::{EntityHandle, EventHandle, FactionHandle, LineageHandle, PlaceHandle};

/// Anything a participant can point at and pin (ruling 210): whatever can
/// have an impresa. Other kinds join as the sim mints handles for them.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Pointable {
    Entity(EntityHandle),
    Place(PlaceHandle),
    Lineage(LineageHandle),
    Faction(FactionHandle),
    Event(EventHandle),
}

/// What one participant attends to (wing design record §5.2 point 4;
/// rulings 204, 210 to 213): at once the roots the collector keeps from, the
/// foreground that runs in detail, and the source of the participant's event
/// stream. The sim holds it and publishes it with each view; a game changes
/// it only through [`AttentionChange`] intents, so every change is logged
/// (ruling 113).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttentionSet {
    /// Who the participant plays (rulings 130, 152, 155), changed by
    /// checkpoint answers rather than by attention changes.
    pub played: BTreeSet<EntityHandle>,
    /// What the participant pinned (rulings 130, 210).
    pub pinned: BTreeSet<Pointable>,
    /// The region the participant's view shows up close, a place-graph node
    /// such as the terrarium section's site or a battlemap (ruling 212). Far
    /// views, an overmap or a minimap, are not examination.
    pub examined: Option<PlaceHandle>,
    /// The game's own care (ruling 71), derived by the sim from the game's
    /// profile: Mesocosm's lineage, Eponym's sophont and those it knows, the
    /// VTT's characters and what the table authored. A group here keeps its
    /// noted members exact and runs the rest as a crowd (ruling 211).
    pub care: BTreeSet<Pointable>,
}

/// A change a participant makes to their attention set, submitted as an
/// intent so a replay knows what was watched (ruling 113).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttentionChange {
    Pin(Pointable),
    Unpin(Pointable),
    /// The view's up-close region changed. Sent only when it does, so the
    /// camera stays presentation inside a region (ruling 212).
    Examine(PlaceHandle),
    StopExamining,
}
