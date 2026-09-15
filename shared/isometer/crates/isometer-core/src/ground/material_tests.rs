// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The material sampler seam: a column's kind chosen per voxel, with the
//! shipped soil-over-rock choice as one caller among others.

use super::*;

/// A flat plain, so every column has the same surface and the only thing a
/// receipt can be reading is the sampler.
struct Flat;

impl Terrain for Flat {
    fn sea_level(&self, _extent: i32) -> i32 {
        1
    }

    fn surface(&self, _extent: i32, _x: i32, _z: i32) -> i32 {
        6
    }
}

fn material_at(ground: &Ground, at: [i32; 3]) -> u8 {
    ground
        .brick_materials(brick_of(at))
        .map_or(AIR, |(brick, _)| brick.get(local_of(at)))
}

/// The one receipt that keeps every existing brick byte where it was: the
/// sampler `grow` passes is what `grow` always wrote inline.
#[test]
fn grow_with_the_soil_and_rock_sampler_is_exactly_grow() {
    let grown = Ground::grow(&Flat, 4);
    let sampled = Ground::grow_with(&Flat, 4, soil_over_rock);
    assert_eq!(grown, sampled);
    assert_eq!(
        crate::snapshot::encode(&grown).unwrap(),
        crate::snapshot::encode(&sampled).unwrap()
    );
    assert_eq!(material_at(&grown, [0, 6, 0]), SOIL);
    assert_eq!(material_at(&grown, [0, 5, 0]), SOIL);
    assert_eq!(material_at(&grown, [0, 4, 0]), ROCK);
}

/// A tile kind per column is the board's case, so a sampler that reads the
/// column and not the depth has to reach every voxel of that column.
#[test]
fn a_column_parity_sampler_places_exactly_its_two_materials() {
    let ground = Ground::grow_with(&Flat, 4, |x, _z, _depth| if x % 2 == 0 { 7 } else { 9 });
    for z in -4..=4 {
        for x in -4..=4 {
            let expected = if x % 2 == 0 { 7 } else { 9 };
            for y in 0..=6 {
                assert_eq!(
                    material_at(&ground, [x, y, z]),
                    expected,
                    "column [{x}, {z}] at y {y}"
                );
            }
        }
    }
    assert_eq!(material_at(&ground, [0, 7, 0]), AIR);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "past the palette bound")]
fn a_material_past_the_palette_bound_trips_the_debug_assertion() {
    Ground::grow_with(&Flat, 1, |_x, _z, _depth| MAX_MATERIAL + 1);
}

#[cfg(not(debug_assertions))]
#[test]
fn a_material_past_the_palette_bound_is_clamped() {
    let ground = Ground::grow_with(&Flat, 1, |_x, _z, _depth| 200);
    assert_eq!(material_at(&ground, [0, 6, 0]), MAX_MATERIAL);
}
