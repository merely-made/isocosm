// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{FactionHandle, PlaceHandle};

/// A party's move across the place graph (rulings 72, 205), at a pace. A
/// party is a faction defined by a person (ruling 8), so it is named by its
/// faction handle; a split party is two. `isonetry`'s `TravelResolved` and
/// the campaign's `PartyMoved` and `PartyPaceSet`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Travel {
    pub party: FactionHandle,
    pub to: PlaceHandle,
    pub pace: Pace,
}

/// Travel pace as a percent of normal time: 100 normal, 50 fast, 200 slow,
/// the substrate's own number; what a pace trades is the ruleset's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pace(pub u32);

impl Default for Pace {
    fn default() -> Self {
        Self(100)
    }
}
