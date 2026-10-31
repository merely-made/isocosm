// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// A raw world coordinate `[x, y, z]` in the sim's voxel grid, the near
/// rung's volume (wing design record §3.6, ruling 13). A game's grid is a
/// projection over that volume (ruling 18), so every game names a cell the
/// same way. Lifted to the core at Mark's word on 2026-09-26 from the three
/// game modules that had each defined it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldPoint(pub [i32; 3]);
