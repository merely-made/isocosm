// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::handle::EntityHandle;

/// A raw world coordinate, `[x, y, z]` in the sim's voxel grid. Dev intents
/// reach the grid directly (mesocosm-core's own `PlaceMatter` and the
/// `OffGrid` rejection already do), unlike a play-time [`super::Nudge`],
/// which names a place-graph node (ruling 205).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldPoint(pub [i32; 3]);

/// A dev tool, never play (dev tools plan §2; mesocosm-core's DT3). Applied,
/// refused and recorded like any other intent; a receipt labels a run that
/// used one as assisted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevIntent {
    EndEpoch,
    ForceBirth { organism: EntityHandle },
    Kill { organism: EntityHandle },
    PlaceMatter { at: WorldPoint, mass_mg: u64 },
}
