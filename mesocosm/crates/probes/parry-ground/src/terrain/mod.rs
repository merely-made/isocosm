// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The probe's terrain fixture: legacy Mesocosm's grown enclosure, its
//! relief and the brick terrain `Ground::grow` reads, moved here whole when
//! legacy Mesocosm left Isocosm (wing ruling 786's reading), so this
//! receipt's figures stay as they were. A fixture, not a world.

mod bricks;
mod grown;
mod relief;

use serde::{Deserialize, Serialize};

/// One region of the enclosure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceId(pub u16);

/// A region, and where it is centred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: PlaceId,
    /// Centre in x and z. Not a boundary: a place ends where the next one is
    /// nearer, so the regions tile the enclosure without storing any edges.
    pub centre: [i32; 2],
}

/// The enclosure, divided.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Places {
    places: Vec<Place>,
    /// Neighbours by index, ascending. Ordered, so traversal is deterministic.
    links: Vec<Vec<PlaceId>>,
}

impl Places {
    pub fn all(&self) -> impl Iterator<Item = &Place> {
        self.places.iter()
    }

    pub fn get(&self, id: PlaceId) -> Option<&Place> {
        self.places.get(id.0 as usize)
    }
}
