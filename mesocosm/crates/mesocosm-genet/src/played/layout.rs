// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Display positions for cohort representatives with no in-site pose.

use crate::section::PlacedBody;
use isometer::core::{Aabb, BodyDocument};
use isometer::space::volume::Volume;

const BINS: usize = 16;

struct Floor {
    lo: [i32; 2],
    hi: [i32; 2],
    taken: [u16; BINS],
}

impl Floor {
    // Occupied bins approximate spacing only. Placement and resting height use
    // native voxel coordinates even for bodies smaller than a bin.
    fn bins(&self, at: [i32; 2], bounds: Aabb) -> (u16, std::ops::RangeInclusive<usize>) {
        let bin = |v: i32, axis: usize| {
            ((v - self.lo[axis]) * BINS as i32 / (self.hi[axis] - self.lo[axis]))
                .clamp(0, BINS as i32 - 1) as usize
        };
        let x0 = bin(at[0] + bounds.min[0], 0);
        let x1 = bin(at[0] + bounds.max[0] - 1, 0).max(x0);
        let z0 = bin(at[1] + bounds.min[2], 1);
        let z1 = bin(at[1] + bounds.max[2] - 1, 1).max(z0);
        (u16::MAX << x0 & u16::MAX >> (BINS - 1 - x1), z0..=z1)
    }

    fn reserve(&mut self, at: [i32; 2], bounds: Aabb) {
        let (mask, rows) = self.bins(at, bounds);
        for z in rows {
            self.taken[z] |= mask;
        }
    }

    fn spot(&self, id: u64, bounds: Aabb) -> [i32; 2] {
        // Give each representative a repeatable preferred spot, biased away
        // from the rim. Resolve crowding before proximity to that spot.
        let hash = id.wrapping_mul(0x9e3779b97f4a7c15);
        let preferred = [hash >> 32, hash >> 48].map(|v| 3 + (v % 11) as i32);
        let anchor = |bin: i32, axis: usize, body_axis: usize| {
            let first = self.lo[axis] - bounds.min[body_axis];
            let last = self.hi[axis] - bounds.max[body_axis];
            if last < first {
                return (first + last) / 2;
            }
            first + (last - first) * bin / (BINS as i32 - 1)
        };
        let mut best = ((u32::MAX, i32::MAX), [0; 2]);
        for z in 0..BINS as i32 {
            for x in 0..BINS as i32 {
                let at = [anchor(x, 0, 0), anchor(z, 1, 2)];
                let (mask, rows) = self.bins(at, bounds);
                let occupied = rows.map(|r| (self.taken[r] & mask).count_ones()).sum();
                let rank = (
                    occupied,
                    (x - preferred[0]).pow(2) + (z - preferred[1]).pow(2),
                );
                if rank < best.0 {
                    best = (rank, at);
                }
            }
        }
        best.1
    }
}

// Quarter turns keep the boxes axis aligned. Use the drawn corners, including
// custom pivots, to find the occupied columns and the underside of each part.
fn boxes(document: &BodyDocument) -> impl Iterator<Item = Aabb> + '_ {
    document.living().filter_map(|part| {
        let a = document.place(part.id, [0; 3])?;
        let b = document.place(part.id, part.half_extent.map(|v| 2 * v))?;
        Some(Aabb {
            min: [0, 1, 2].map(|k| a[k].min(b[k])),
            max: [0, 1, 2].map(|k| a[k].max(b[k])),
        })
    })
}

pub fn residents(
    volume: &Volume,
    bodies: &mut [PlacedBody],
    patch: impl Fn(u64) -> Option<[i64; 3]>,
) {
    let lower = volume.to_ground([volume.min[0], volume.datum, volume.min[1]]);
    let upper = volume.to_ground([volume.max[0], volume.datum, volume.max[1]]);
    let mut floor = Floor {
        lo: [lower[0], lower[2]],
        hi: [upper[0], upper[2]],
        taken: [0; BINS],
    };
    let mut unplaced = Vec::new();
    for body in bodies {
        let saved = patch(body.id)
            .filter(|p| volume.holds(p[0], p[2]))
            .map(|p| volume.to_ground(p));
        if let Some(at) = saved {
            body.at = at;
        }
        let parts: Vec<_> = boxes(&body.document).collect();
        let Some(bounds) = parts.iter().copied().reduce(|a, b| Aabb {
            min: [0, 1, 2].map(|k| a.min[k].min(b.min[k])),
            max: [0, 1, 2].map(|k| a.max[k].max(b.max[k])),
        }) else {
            continue;
        };
        if saved.is_some() {
            floor.reserve([body.at[0], body.at[2]], bounds);
        } else {
            unplaced.push((body, parts, bounds));
        }
    }
    for (body, parts, bounds) in unplaced {
        let [x, z] = floor.spot(body.id, bounds);
        floor.reserve([x, z], bounds);
        let mut root_y = None;
        for part in parts {
            for px in (x + part.min[0]).max(floor.lo[0])..(x + part.max[0]).min(floor.hi[0]) {
                for pz in (z + part.min[2]).max(floor.lo[1])..(z + part.max[2]).min(floor.hi[1]) {
                    if let Some(top) = volume.surface(
                        i64::from(px - lower[0]) + volume.min[0],
                        i64::from(pz - lower[2]) + volume.min[1],
                    ) {
                        let y = (top - volume.datum) as i32 + 1 - part.min[1];
                        root_y = Some(root_y.map_or(y, |prev: i32| prev.max(y)));
                    }
                }
            }
        }
        body.at = [x, root_y.unwrap_or(0) + body.document.aabb().min[1], z];
    }
}

pub fn centre(volume: &Volume) -> [i32; 3] {
    let x = (volume.min[0] + volume.max[0]) / 2;
    let z = (volume.min[1] + volume.max[1]) / 2;
    let y = volume.surface(x, z).map_or(volume.datum, |y| y + 1);
    volume.to_ground([x, y, z])
}
