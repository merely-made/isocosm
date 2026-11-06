// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-column brick queries, answered by ordered lookups rather than a pass
//! over the whole ground.

use std::ops::Range;

use super::Ground;

impl Ground {
    /// The brick layers one column holds, lowest to highest stored key, as a
    /// half-open range; empty when the column holds none. Bricks are ordered
    /// x, y, z, so the x slab's ends bound y and each layer is one lookup.
    pub fn column_layers(&self, column: [i16; 2]) -> Range<i16> {
        let [x, z] = column;
        let slab = || {
            self.bricks
                .range([x, i16::MIN, i16::MIN]..=[x, i16::MAX, i16::MAX])
        };
        let (Some((first, _)), Some((last, _))) = (slab().next(), slab().next_back()) else {
            return 0..0;
        };
        let mut held = (first[1]..=last[1]).filter(|y| self.bricks.contains_key(&[x, *y, z]));
        let Some(low) = held.next() else {
            return 0..0;
        };
        let high = held.next_back().unwrap_or(low);
        low..high + 1
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Cavity, Terrain};
    use super::*;

    struct Hills;

    impl Terrain for Hills {
        fn sea_level(&self, _extent: i32) -> i32 {
            2
        }

        fn surface(&self, _extent: i32, x: i32, z: i32) -> i32 {
            (x * 7 + z * 3).rem_euclid(30)
        }

        fn cavities(&self, _extent: i32) -> Vec<Cavity> {
            vec![Cavity {
                rooms: vec![([4, 6, 4], 3)],
                route: Vec::new(),
            }]
        }
    }

    fn scanned(ground: &Ground, column: [i16; 2]) -> Range<i16> {
        let mut ys = ground
            .keys()
            .filter(|k| [k[0], k[2]] == column)
            .map(|k| k[1]);
        let Some(first) = ys.next() else {
            return 0..0;
        };
        let (low, high) = ys.fold((first, first), |(lo, hi), y| (lo.min(y), hi.max(y)));
        low..high + 1
    }

    #[test]
    fn column_layers_agree_with_a_full_scan() {
        let mut ground = Ground::grow(&Hills, 24);
        // A lone voxel well above one column leaves a gap the range spans.
        ground.write([([3, 70, -5], 2)]).unwrap();
        for z in -5..=5 {
            for x in -5..=5 {
                assert_eq!(
                    ground.column_layers([x, z]),
                    scanned(&ground, [x, z]),
                    "column [{x}, {z}]"
                );
            }
        }
        assert_eq!(
            ground.column_layers([0, -1]).end,
            9,
            "the lone voxel's brick tops column [0, -1]"
        );
    }
}
