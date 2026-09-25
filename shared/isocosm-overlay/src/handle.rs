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
/// **Provisional, not ruled** (see the crate README's "Open forks"): a
/// site reference or raw coordinates are the other live options for how a
/// PLACES directive names a location.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceHandle(pub u64);

/// An opaque reference to a priced candidate on the review's table (the
/// sim-side counterpart of a discovered condition), named in a review
/// answer at the epoch boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CandidateHandle(pub u64);
