// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's terrain description, over `isometer-core`'s brick container.
//!
//! G1's container (place-graph engine plan §3) moved to
//! [`isometer_core::ground`] with the family split; the relief model and the
//! nests that carve real burrows under their hosts are Mesocosm's and stay
//! here. What crosses the seam is [`Terrain`]: a sea line, a surface per
//! column, and the voids to hollow. `Ground::grow(&grown, extent)` reads the
//! same numbers in the same order it always did, so the raised bytes are
//! unchanged.

use isometer_core::ground::{Cavity, SURFACE_BAND, Terrain};

use super::grown::{Grown, Nest};

/// The generated, embodied route into one nest. It is derived from graph
/// facts, never stored beside the brick truth: Ground owns the resulting
/// voxels, while tests and generation share the one construction rule.
#[derive(Clone, Debug)]
pub struct NestEntry {
    /// Surface column at the mouth of the entry.
    pub anchor: [i32; 2],
    /// Floor height of the roofed room at the inner end.
    pub floor: i32,
    /// Ordered standable voxels from surface mouth to inner room.
    pub route: Vec<[i32; 3]>,
}

impl Grown {
    /// The exact generated access routes embodied by `Ground::grow`.
    ///
    /// This is a read model over the same construction rule as terrain
    /// generation. Debuggers and projections can name a threshold without
    /// reconstructing one from rendered voxels or storing another authority.
    pub fn nest_entries(&self, extent: i32) -> impl Iterator<Item = (Nest, NestEntry)> + '_ {
        self.nests
            .iter()
            .copied()
            .filter_map(move |nest| nest_entry(self, extent, nest).map(|entry| (nest, entry)))
    }
}

/// Mesocosm's half of the family's terrain seam.
///
/// Burrows are anchored at the highest column near the host, so a low-lying
/// host digs into its own hillside instead of cratering. Rooms scale to the
/// depth the ground actually affords, and every chamber keeps a roof. The
/// entry descends one voxel per horizontal step, because a vertical hollow is
/// a picture of a burrow, not a route `near::step` can actually traverse.
/// Deterministic from the graph alone.
impl Terrain for Grown {
    fn sea_level(&self, _extent: i32) -> i32 {
        1 + self.relief.sea * (SURFACE_BAND - 1) / super::relief::CEILING
    }

    fn surface(&self, extent: i32, x: i32, z: i32) -> i32 {
        surface_from(self, extent, x, z)
    }

    fn cavities(&self, extent: i32) -> Vec<Cavity> {
        let mut out = Vec::new();
        for nest in &self.nests {
            let Some(entry) = nest_entry(self, extent, *nest) else {
                continue;
            };
            let [x, z] = entry.anchor;
            let anchor = entry.route[0][1] - 1;
            let floor = entry.floor;
            let radius = if anchor - floor >= 5 { 2 } else { 1 };
            let [entry_dx, entry_dz] = nest_entry_direction(nest.host.0);
            let drop = anchor - floor;
            let (entry_x, entry_z) = (x + entry_dx * drop, z + entry_dz * drop);
            let mut rooms = Vec::new();
            for room in 0..nest.rooms {
                let spin = (nest.host.0 as i32 * 7 + room as i32 * 5) % 8;
                let (dx, dz) = [
                    (radius + 1, 0),
                    (radius, radius),
                    (0, radius + 1),
                    (-radius, radius),
                    (-radius - 1, 0),
                    (-radius, -radius),
                    (0, -radius - 1),
                    (radius, -radius),
                ][spin as usize];
                let lift = (room as i32) % (anchor - 1 - radius - floor).max(1);
                let centre_y = (floor + lift).min(anchor - 1 - radius).max(1 + radius);
                rooms.push(([entry_x + dx, centre_y, entry_z + dz], radius));
            }
            out.push(Cavity {
                rooms,
                route: entry.route,
            });
        }
        out
    }
}

pub(crate) fn nest_entry(grown: &Grown, extent: i32, nest: Nest) -> Option<NestEntry> {
    let host = grown.places.get(nest.host)?;
    let [cx, cz] = host.centre;
    let (mut x, mut z, mut anchor) = (cx, cz, 0);
    for dz in -5..=5 {
        for dx in -5..=5 {
            let (px, pz) = (cx + dx, cz + dz);
            // The mouth stays inside the wall too: a burrow is part of the
            // vessel, not a tunnel through it. (TD2b)
            if px.abs() > extent || pz.abs() > extent {
                continue;
            }
            let surface = surface_from(grown, extent, px, pz);
            if surface > anchor {
                (x, z, anchor) = (px, pz, surface);
            }
        }
    }
    if anchor < 4 {
        return None;
    }
    let [dx, dz] = nest_entry_direction(nest.host.0);
    // How many steps this direction has before the route would leave the
    // resident bound; caps depth so the descent never crosses the wall.
    // Measured at 500 real-enclosure seeds before this cap: routes drifted
    // up to 12 voxels past the wall (probe removed once fixed). (TD2b)
    let room_for_direction = extent - (x * dx + z * dz);
    let depth_cap = ((room_for_direction - 1).max(0)) / 3;
    if depth_cap < 1 {
        return None;
    }
    let depth = ((anchor - 2) / 3)
        .clamp(1, nest.depth as i32)
        .min(depth_cap);
    let floor = (anchor - 3 * depth - 1).max(1);
    let route = (0..=anchor - floor)
        .map(|step| [x + dx * step, anchor + 1 - step, z + dz * step])
        .collect();
    Some(NestEntry {
        anchor: [x, z],
        floor,
        route,
    })
}

fn surface_from(grown: &Grown, extent: i32, x: i32, z: i32) -> i32 {
    1 + grown.relief.sample(extent, x, z) * (SURFACE_BAND - 1) / super::relief::CEILING
}

fn nest_entry_direction(host: u16) -> [i32; 2] {
    [[1, 0], [0, 1], [-1, 0], [0, -1]][host as usize % 4]
}

// Split out at the 600-LOC ceiling (2026-08-29, TD2b): same module, just a
// separate file, per the `organism::ecology` / `ecology::tests` precedent.
#[cfg(test)]
mod tests;
