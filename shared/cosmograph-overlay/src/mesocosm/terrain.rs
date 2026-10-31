// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Game-resolved actual geometry for a native-declared chosen operation.

use serde::{Deserialize, Serialize};
use crate::{ActKey, EntityHandle, PlaceHandle, Tick};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainShape {
    Sphere { centre: [i64; 3], radius: u64 },
    Box { min: [i64; 3], max: [i64; 3] },
    Route { points: Vec<[i64; 3]>, radius: u64 },
}

/// Cells in the site's base frame. Identity anchors and render spots are absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainGeometry {
    pub action: String,
    pub process: ActKey,
    pub tick: Tick,
    pub actor: EntityHandle,
    pub site: PlaceHandle,
    pub body_revision: u64,
    pub terrain_revision: String,
    pub eye: [i64; 3],
    pub focus: [i64; 3],
    pub min: [i64; 2],
    pub max: [i64; 2],
    pub depth: i64,
    pub shape: TerrainShape,
}
