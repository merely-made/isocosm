// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receipts for paged terrain, on the CPU: which bricks a frame names, and
//! what the brick map holds after each retarget, rebuild and refresh.

use std::collections::BTreeMap;

use isometer_lens::ATLAS_BUDGET_BYTES;

use super::paging::span_of;
use super::*;

/// The card the receipts size their atlas to: the historical 2,047 bricks,
/// far more than any framing here holds.
const LIMITS: AtlasLimits = AtlasLimits::DEFAULT;

mod framing;
mod paging;

/// A heightfield `side` columns each way from the origin, stepped so its
/// bricks stand one and two layers tall, with columns an edit can reset.
struct Hills {
    side: i32,
    edits: BTreeMap<(i32, i32), i32>,
}

impl Hills {
    fn new() -> Self {
        Self {
            side: 96,
            edits: BTreeMap::new(),
        }
    }

    /// The tallest surface any column stands at: the stepped field tops out
    /// at 11, and an edit can lift a column past it.
    fn tallest(&self) -> i32 {
        self.edits.values().copied().chain([11]).max().unwrap_or(11)
    }

    fn surface(&self, x: i32, z: i32) -> i32 {
        if x < -self.side || x >= self.side || z < -self.side || z >= self.side {
            return -1;
        }
        match self.edits.get(&(x, z)) {
            Some(surface) => *surface,
            None => (x.div_euclid(7) + z.div_euclid(5)).rem_euclid(11) + 1,
        }
    }

    fn brick(&self, key: [i16; 3]) -> Vec<u8> {
        let mut out = vec![0; BRICK_BYTES];
        self.fill(key, &mut out);
        out
    }
}

impl BrickSource for Hills {
    fn bounds(&self) -> Option<[[i16; 3]; 2]> {
        let key = |v: i32| v.div_euclid(BRICK) as i16;
        Some([
            [key(-self.side), 0, key(-self.side)],
            [key(self.side - 1), key(self.tallest()), key(self.side - 1)],
        ])
    }

    fn layers(&self, column: [i16; 2]) -> Range<i16> {
        let mut top = -1;
        for dz in 0..BRICK {
            for dx in 0..BRICK {
                let at = [
                    i32::from(column[0]) * BRICK + dx,
                    i32::from(column[1]) * BRICK + dz,
                ];
                top = top.max(self.surface(at[0], at[1]));
            }
        }
        if top < 0 {
            0..0
        } else {
            0..(top.div_euclid(BRICK) as i16 + 1)
        }
    }

    fn fill(&self, key: [i16; 3], out: &mut [u8]) {
        for y in 0..BRICK {
            for z in 0..BRICK {
                for x in 0..BRICK {
                    let at = key_origin(key).map(|v| v as i32);
                    let (vx, vy, vz) = (at[0] + x, at[1] + y, at[2] + z);
                    let surface = self.surface(vx, vz);
                    out[((y * BRICK + z) * BRICK + x) as usize] = match vy {
                        _ if vy < 0 || vy > surface => 0,
                        _ if vy == surface => 2,
                        _ => 1,
                    };
                }
            }
        }
    }
}

fn key_origin(key: [i16; 3]) -> [i64; 3] {
    key.map(|k| i64::from(k) * i64::from(BRICK))
}

fn camera(centre: [f32; 2]) -> SlabCamera {
    SlabCamera::dimetric_2_1([centre[0], 6.0, centre[1]], 24.0, 1.5, 600.0)
        .expect("the test camera frames")
}

/// The bricks a camera shows, by projecting every corner of every brick the
/// terrain holds through the camera's own `ndc_of`, with the frame grown by
/// `margin` world units and then by `slack` in clip space.
fn brute(camera: SlabCamera, margin: f32, slack: f32, hills: &Hills) -> BTreeSet<[i16; 3]> {
    let [low, high] = hills.bounds().expect("bounded");
    let reach = [
        1.0 + margin / (camera.half_height * camera.aspect) + slack,
        1.0 + margin / camera.half_height + slack,
    ];
    let mut shown = BTreeSet::new();
    for z in low[2]..=high[2] {
        for x in low[0]..=high[0] {
            for y in hills.layers([x, z]) {
                let origin = key_origin([x, y, z]);
                let mut box_low = [f32::INFINITY; 2];
                let mut box_high = [f32::NEG_INFINITY; 2];
                for bits in 0..8 {
                    let corner =
                        [0, 1, 2].map(|axis| (origin[axis] + (bits >> axis & 1) * 8) as f32);
                    let ndc = camera.ndc_of(corner).expect("finite");
                    for i in 0..2 {
                        box_low[i] = box_low[i].min(ndc[i]);
                        box_high[i] = box_high[i].max(ndc[i]);
                    }
                }
                if (0..2).all(|i| box_low[i] <= reach[i] && box_high[i] >= -reach[i]) {
                    shown.insert([x, y, z]);
                }
            }
        }
    }
    shown
}

fn keys(framed: &FramedBricks) -> BTreeSet<[i16; 3]> {
    framed.keys.iter().copied().collect()
}

fn frame(hills: &Hills, centre: [f32; 2]) -> FramedBricks {
    framed_bricks(camera(centre), 8.0, hills, usize::MAX)
}

/// Every resident brick reads back through the map exactly as the source
/// makes it, and a brick the map does not hold reads as air.
fn assert_reads(map: &BrickMap, residency: &Residency, hills: &Hills) {
    for key in residency.resident() {
        let expected = hills.brick(*key);
        let origin = key_origin(*key);
        for y in 0..8 {
            for z in 0..8 {
                for x in 0..8 {
                    let at = [origin[0] + x, origin[1] + y, origin[2] + z].map(|v| v as i32);
                    assert_eq!(
                        map.material_at(at),
                        expected[((y * 8 + z) * 8 + x) as usize],
                        "brick {key:?} voxel {at:?}"
                    );
                }
            }
        }
    }
}

/// Drives one frame's refresh the way the scene does.
fn step(
    residency: &RefCell<Residency>,
    map: &mut BrickMap,
    hills: &Hills,
    framed: &FramedBricks,
    dirty: &[[i16; 3]],
) -> TerrainRefresh {
    PagedTerrain::new(residency, hills, framed, 1)
        .refresh(map, dirty)
        .expect("the refresh")
}

/// The nearest pan from `from` along +x that moves the framing, shrinking it
/// or not as asked.
fn pan(hills: &Hills, from: [f32; 2], shrink: bool) -> [f32; 2] {
    let held = keys(&frame(hills, from));
    (1..400)
        .map(|step| [from[0] + step as f32 * 0.5, from[1]])
        .find(|to| {
            let framed = keys(&frame(hills, *to));
            framed != held && (framed.len() < held.len()) == shrink
        })
        .expect("some pan moves the framing that way")
}

/// The column at the frame's centre, raised to `surface`, and every brick
/// its column holds up to that height, for the edit's dirty list.
fn lift(hills: &mut Hills, surface: i32) -> Vec<[i16; 3]> {
    hills.edits.insert((3, 4), surface);
    (0..=surface.div_euclid(BRICK) as i16)
        .map(|layer| [0, layer, 0])
        .collect()
}
