// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Raising the terrain's columns a brick column at a time: each brick's
//! materials are gathered in a buffer and stored once, instead of one map
//! entry per voxel.

use super::{BRICK, Brick, Ground, MAX_MATERIAL, Terrain};

impl Ground {
    /// Lays every column from y 0 to its surface across the resident bound,
    /// as [`Ground::grow_with`] describes. A brick exists exactly where a
    /// voxel was laid, as the per-voxel path left it.
    pub(super) fn fill_columns<T: Terrain + ?Sized>(
        &mut self,
        terrain: &T,
        material: &impl Fn(i32, i32, i32) -> u8,
    ) {
        let extent = self.extent;
        let side = (BRICK * BRICK * BRICK) as usize;
        let span = |b: i32| (b * BRICK).max(-extent)..=(b * BRICK + BRICK - 1).min(extent);
        let bricks = (-extent).div_euclid(BRICK)..=extent.div_euclid(BRICK);
        // Per brick column, the layers by brick y, laid or not.
        let mut layers: Vec<Option<Vec<u8>>> = Vec::new();
        for bz in bricks.clone() {
            for bx in bricks.clone() {
                layers.clear();
                for z in span(bz) {
                    for x in span(bx) {
                        let surface = terrain.surface(extent, x, z);
                        for y in 0..=surface {
                            let chosen = material(x, z, surface - y);
                            debug_assert!(
                                chosen <= MAX_MATERIAL,
                                "material {chosen} at [{x}, {y}, {z}] is past the palette bound {MAX_MATERIAL}"
                            );
                            let layer = (y / BRICK) as usize;
                            if layers.len() <= layer {
                                layers.resize(layer + 1, None);
                            }
                            let local = [x - bx * BRICK, y % BRICK, z - bz * BRICK];
                            layers[layer].get_or_insert_with(|| vec![0; side])[Brick::index(local)] =
                                chosen.min(MAX_MATERIAL);
                        }
                    }
                }
                for (by, laid) in layers.drain(..).enumerate() {
                    if let Some(materials) = laid {
                        let key = [bx as i16, by as i16, bz as i16];
                        self.bricks.insert(key, Brick { materials });
                        self.dirty.insert(key);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Cavity, soil_over_rock};
    use super::*;

    struct Relief(fn(i32, i32) -> i32, Vec<Cavity>);

    impl Terrain for Relief {
        fn sea_level(&self, _extent: i32) -> i32 {
            2
        }

        fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
            (self.0)(x, z)
        }

        fn cavities(&self, _extent: i32) -> Vec<Cavity> {
            self.1.clone()
        }
    }

    /// The per-voxel growth this replaced, kept as the reference.
    fn per_voxel(terrain: &Relief, extent: i32, material: impl Fn(i32, i32, i32) -> u8) -> Ground {
        let mut ground = Ground::empty(extent, terrain.sea_level(extent));
        for z in -extent..=extent {
            for x in -extent..=extent {
                let surface = terrain.surface(extent, x, z);
                for y in 0..=surface {
                    ground.place([x, y, z], material(x, z, surface - y).min(MAX_MATERIAL));
                }
            }
        }
        for cavity in terrain.cavities(extent) {
            for (centre, radius) in &cavity.rooms {
                ground.hollow(*centre, *radius);
            }
            ground.carve_route(&cavity.route);
        }
        ground.dirty.clear();
        ground.revision = 0;
        ground
    }

    #[test]
    fn brick_wise_growth_matches_the_per_voxel_path() {
        let reliefs = [
            Relief(|x, z| (x * 7 + z * 3).rem_euclid(30), Vec::new()),
            Relief(|_, _| 0, Vec::new()),
            Relief(|_, _| -1, Vec::new()),
            Relief(|x, z| 30 + (x / 9 + z / 13) % 7, Vec::new()),
            Relief(
                |x, z| 12 + (x - z).rem_euclid(5),
                vec![Cavity {
                    rooms: vec![([4, 6, 4], 3), ([-9, 4, 7], 2)],
                    route: vec![[4, 9, 4], [5, 8, 4], [6, 7, 4]],
                }],
            ),
        ];
        let banded = |x: i32, z: i32, depth: i32| ((x + z).rem_euclid(3) as u8 + depth as u8) % 64;
        for (i, relief) in reliefs.iter().enumerate() {
            // Extents that do and do not end on a brick edge.
            for extent in [7, 8, 20, 23] {
                let grown = Ground::grow(relief, extent);
                let reference = per_voxel(relief, extent, soil_over_rock);
                assert_eq!(grown, reference, "relief {i} at extent {extent}");
                assert_eq!(
                    crate::snapshot::encode(&grown).unwrap(),
                    crate::snapshot::encode(&reference).unwrap(),
                    "relief {i} at extent {extent} encodes alike"
                );
                assert_eq!(
                    Ground::grow_with(relief, extent, banded),
                    per_voxel(relief, extent, banded),
                    "relief {i} at extent {extent} with banded materials"
                );
            }
        }
    }
}
