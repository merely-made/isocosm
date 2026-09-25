// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// An opaque reference to one entity in the sim, minted by the sim and
/// carried by value (wing design record §5.2 point 6). Never a pointer, and
/// never constructed by a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityHandle(pub u64);

/// An opaque reference to a place: a node of the sim's place graph (wing
/// design record rulings 72, 147), such as a range, a boundary or a home.
///
/// A PLACES directive names a location this way, never by site or by
/// coordinates (ruling 205).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceHandle(pub u64);

/// An opaque reference to a priced candidate on the review's table (the
/// sim-side counterpart of a discovered condition), named in a review
/// answer at the epoch boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CandidateHandle(pub u64);

/// An opaque reference to a participant: an identity holding a grant with
/// the right to petition (the terminology supersession of 2026-09-20), such
/// as a player, a servitor or a scenario runner. Each has one attention set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ParticipantHandle(pub u64);

/// An opaque reference to a lineage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LineageHandle(pub u64);

/// An opaque reference to a faction, a polity among them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FactionHandle(pub u64);

/// An opaque reference to an event in the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventHandle(pub u64);
