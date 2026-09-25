// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::handle::{EntityHandle, PlaceHandle};

/// A standing order for the entity a player plays, one critter or its kin
/// directed whole (rulings 152, 155), in force until replaced (ruling 176). Distinct from a [`Nudge`], which is a one-off the
/// critter merely weighs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Directive {
    pub entity: EntityHandle,
    pub kind: DirectiveKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DirectiveKind {
    Priorities(Priorities),
    Places(Places),
    Stances(Stances),
    /// A one-off suggestion, sent as a directive so it shares the entity
    /// and tick stamping every other directive has.
    Nudge(Nudge),
}

/// Which needs and abilities come first (ruling 176). Named by opaque key,
/// the same way the sim names processes and accounts by string
/// (`isocosm::schema::Key`, not depended on here): the vocabulary is the
/// ruleset's, not fixed by this crate.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PriorityKey(pub String);

/// A ranking, first to last preferred. Absent keys are left to the
/// critter's own methodology.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Priorities {
    pub order: Vec<PriorityKey>,
}

/// Where to range, where to avoid, where to make home (ruling 176).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Places {
    pub range: Vec<PlaceHandle>,
    pub avoid: Vec<PlaceHandle>,
    pub home: Option<PlaceHandle>,
}

/// Bold or cautious, and how to meet a competitor (ruling 176).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stances {
    pub boldness: Boldness,
    pub competitor: CompetitorStance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Boldness {
    Bold,
    Cautious,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompetitorStance {
    Contest,
    Yield,
    Share,
}

/// A one-off suggestion, weighed like any other input rather than obeyed
/// (ruling 176): go there, or eat that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nudge {
    GoTo(PlaceHandle),
    EatThat(EntityHandle),
}
