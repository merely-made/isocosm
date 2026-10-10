// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where the section stands a body the runtime has not placed in a patch
//! yet: presentation only, on a fixed grid around the ground's centre, in
//! roster order, each on the surface under it.

use isometer::core::ground::Ground;

/// Voxels between neighbouring presentation spots.
pub const SPACING: i32 = 8;

/// The `index`th spot of a square spiral-free grid around the origin, on the
/// surface. Deterministic in `index` alone, so a frame never reshuffles.
pub fn spot(ground: &Ground, index: usize) -> [i32; 3] {
    let reach = (ground.extent() / SPACING).max(1);
    let side = (2 * reach + 1) as usize;
    let cell = index % (side * side);
    let x = (cell % side) as i32 - reach;
    let z = (cell / side) as i32 - reach;
    let (x, z) = (x * SPACING, z * SPACING);
    let y = ground.surface(x, z).map_or(0, |top| top + 1);
    [x, y, z]
}

/// Where the camera sits for a site with nobody to follow: its centre column.
pub fn centre(ground: &Ground) -> [i32; 3] {
    [0, ground.surface(0, 0).map_or(0, |top| top + 1), 0]
}
