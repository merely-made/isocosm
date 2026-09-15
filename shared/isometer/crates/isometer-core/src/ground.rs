// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Brick truth: the ground as voxels, in the one coordinate space.
//!
//! A product's terrain model decides every column's surface and describes the
//! voids under it; what lives here is the container those answers fill. An
//! interior is a hole in the same ground everything walks on, never a scene.
//! Dense 8³ bricks in an ordered map: hashable, diffable, and serialized flat.
//! Carves mark their bricks dirty and bump one revision, which is the fact a
//! renderer's upload discipline consumes.
//!
//! [`Terrain`] is the whole of the seam. No generation, no relief model and no
//! world appears in this file.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

/// Voxels per brick edge.
pub const BRICK: i32 = 8;
/// The world-height band relief maps onto. Three bricks of headroom.
pub const SURFACE_BAND: i32 = 24;

/// Materials. Zero is air; the rest are palette indices for projections.
pub const AIR: u8 = 0;
pub const ROCK: u8 = 2;
pub const SOIL: u8 = 3;

/// One dense 8³ brick, y-major then z then x.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brick {
    materials: Vec<u8>,
}

impl Brick {
    fn empty() -> Self {
        Self {
            materials: vec![AIR; (BRICK * BRICK * BRICK) as usize],
        }
    }

    fn index(local: [i32; 3]) -> usize {
        ((local[1] * BRICK + local[2]) * BRICK + local[0]) as usize
    }

    pub fn get(&self, local: [i32; 3]) -> u8 {
        self.materials[Self::index(local)]
    }

    fn set(&mut self, local: [i32; 3], material: u8) {
        self.materials[Self::index(local)] = material;
    }

    pub fn is_empty(&self) -> bool {
        self.materials.iter().all(|m| *m == AIR)
    }
}

/// The ground: every solid voxel the world owns, plus the lifecycle facts
/// a projection needs (revision, dirty bricks).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ground {
    extent: i32,
    /// World y of the water line, derived from the relief's sea.
    pub sea_level: i32,
    bricks: BTreeMap<[i16; 3], Brick>,
    revision: u64,
    /// Projection work queue, not world authority. Hosts may drain this at
    /// different frame rates, so it must never alter snapshots or replay
    /// hashes; the revision and brick bytes carry the authoritative change.
    #[serde(skip)]
    dirty: BTreeSet<[i16; 3]>,
}

impl PartialEq for Ground {
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent
            && self.sea_level == other.sea_level
            && self.bricks == other.bricks
            && self.revision == other.revision
    }
}

impl Eq for Ground {}

fn brick_of(at: [i32; 3]) -> [i16; 3] {
    [
        at[0].div_euclid(BRICK) as i16,
        at[1].div_euclid(BRICK) as i16,
        at[2].div_euclid(BRICK) as i16,
    ]
}

fn local_of(at: [i32; 3]) -> [i32; 3] {
    [
        at[0].rem_euclid(BRICK),
        at[1].rem_euclid(BRICK),
        at[2].rem_euclid(BRICK),
    ]
}

/// One carved void and the way in to it.
///
/// A product describes its burrows, caves or cellars as rooms plus an access
/// route; [`Ground::grow`] is what turns that description into missing voxels.
/// Rooms are hollowed in order, then the route is dug, which is the order the
/// geometry was authored in and the order the bytes depend on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cavity {
    /// Room centres with their radii, applied in order.
    pub rooms: Vec<([i32; 3], i32)>,
    /// Ordered standable voxels from the surface mouth to the inner room.
    /// Empty leaves the rooms sealed.
    pub route: Vec<[i32; 3]>,
}

/// What [`Ground::grow`] needs to know to raise a world, and nothing else.
///
/// The seam that keeps generation a product's business. A relief model, a
/// heightmap, a hand-authored fixture or a procedural world all satisfy this;
/// none of them appears in this crate.
pub trait Terrain {
    /// World y of the water line.
    fn sea_level(&self, extent: i32) -> i32;

    /// Highest solid world y at a column.
    fn surface(&self, extent: i32, x: i32, z: i32) -> i32;

    /// Voids to carve once the terrain stands, in application order.
    fn cavities(&self, extent: i32) -> Vec<Cavity> {
        let _ = extent;
        Vec::new()
    }
}

impl Ground {
    /// Raises the ground a [`Terrain`] described. `extent` is the resident
    /// bound: bricks are laid from `-extent` to `extent` on both horizontal
    /// axes.
    pub fn grow<T: Terrain + ?Sized>(terrain: &T, extent: i32) -> Self {
        let mut ground = Self {
            extent,
            sea_level: terrain.sea_level(extent),
            bricks: BTreeMap::new(),
            revision: 0,
            dirty: BTreeSet::new(),
        };

        for z in -extent..=extent {
            for x in -extent..=extent {
                let surface = terrain.surface(extent, x, z);
                for y in 0..=surface {
                    let material = if y + 2 > surface { SOIL } else { ROCK };
                    ground.place([x, y, z], material);
                }
            }
        }

        // Rooms first, then the way in. The entry descends one voxel per
        // horizontal step, because a vertical hollow is a picture of a burrow
        // and not a route a walker can traverse.
        for cavity in terrain.cavities(extent) {
            for (centre, radius) in &cavity.rooms {
                ground.hollow(*centre, *radius);
            }
            ground.carve_route(&cavity.route);
        }

        // Generation is the world's starting fact, not an edit.
        ground.dirty.clear();
        ground.revision = 0;
        ground
    }

    fn place(&mut self, at: [i32; 3], material: u8) {
        let key = brick_of(at);
        self.bricks
            .entry(key)
            .or_insert_with(Brick::empty)
            .set(local_of(at), material);
        self.dirty.insert(key);
    }

    /// Clears a two-voxel walker volume while preserving (or supplying) its
    /// footing. This is generation geometry, not an edit: callers reset the
    /// revision and dirty queue once the initial world has been raised.
    fn make_stance(&mut self, at: [i32; 3]) {
        let floor = [at[0], at[1] - 1, at[2]];
        if !self.solid(floor) {
            self.place(floor, SOIL);
        }
        for dy in 0..2 {
            let air = [at[0], at[1] + dy, at[2]];
            if self.solid(air) {
                self.place(air, AIR);
            }
        }
    }

    /// Authors one wide walker stance into initial terrain geometry.
    ///
    /// This preserves exactly one chosen floor voxel while clearing the
    /// walker's occupied volume. It is for deterministic world construction;
    /// simulation edits continue to use [`Self::carve`].
    pub fn author_walker_stance(
        &mut self,
        at: [i32; 3],
        radius: i32,
        height: i32,
        support_offset: [i32; 2],
    ) {
        let radius = radius.max(0);
        debug_assert!(support_offset[0].abs() <= radius);
        debug_assert!(support_offset[1].abs() <= radius);
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                for dy in 0..height.max(1) {
                    let air = [at[0] + dx, at[1] + dy, at[2] + dz];
                    if self.solid(air) {
                        self.place(air, AIR);
                    }
                }
            }
        }
        self.place(
            [
                at[0] + support_offset[0],
                at[1] - 1,
                at[2] + support_offset[1],
            ],
            SOIL,
        );
    }

    /// Digs one access route, roofing its inner end. Its directness matters:
    /// an embodied ecology that follows a visible target vector must not need
    /// an unimplemented pathfinder merely to enter a burrow.
    fn carve_route(&mut self, route: &[[i32; 3]]) {
        if route.is_empty() {
            return;
        }
        for at in route {
            self.make_stance(*at);
        }
        let inside = *route.last().expect("a non-empty route has an end");
        let roof = [inside[0], inside[1] + 2, inside[2]];
        if !self.solid(roof) {
            self.place(roof, ROCK);
        }
    }

    fn hollow(&mut self, centre: [i32; 3], radius: i32) {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let at = [centre[0] + dx, centre[1] + dy, centre[2] + dz];
                    if at[1] < 1 {
                        continue;
                    }
                    if self.solid(at) {
                        self.place(at, AIR);
                    }
                }
            }
        }
    }

    /// The resident bound: how far `Ground::grow` laid bricks from the
    /// origin on either horizontal axis. The enclosure's wall, not its
    /// floor — `step_for` is what refuses a step past it. (TD2b)
    pub fn extent(&self) -> i32 {
        self.extent
    }

    /// Whether a voxel is solid. Below y = 0 is bedrock, always solid.
    pub fn solid(&self, at: [i32; 3]) -> bool {
        if at[1] < 0 {
            return true;
        }
        self.bricks
            .get(&brick_of(at))
            .map(|brick| brick.get(local_of(at)) != AIR)
            .unwrap_or(false)
    }

    /// Whether a creature of the given height can occupy `at`: solid
    /// footing below, air through the body.
    pub fn stands(&self, at: [i32; 3], height: i32) -> bool {
        self.solid([at[0], at[1] - 1, at[2]])
            && (0..height.max(1)).all(|dy| !self.solid([at[0], at[1] + dy, at[2]]))
    }

    /// Line of sight between voxel centres: no solid voxel strictly
    /// between them. Integer sampling at half-voxel strides, so the walk
    /// is deterministic and never skips a corner.
    pub fn sees(&self, from: [i32; 3], to: [i32; 3]) -> bool {
        let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let strides = 2 * delta.iter().map(|d| d.abs()).max().unwrap_or(0);
        let mut last = from;
        for stride in 1..strides {
            let at = [
                from[0] + (delta[0] * stride).div_euclid(strides.max(1)),
                from[1] + (delta[1] * stride).div_euclid(strides.max(1)),
                from[2] + (delta[2] * stride).div_euclid(strides.max(1)),
            ];
            if at == last || at == to {
                continue;
            }
            last = at;
            if self.solid(at) {
                return false;
            }
        }
        true
    }

    /// Carves a cube of air around `at`. One revision per carve, however
    /// many voxels it removed; the dirty set records which bricks changed.
    pub fn carve(&mut self, at: [i32; 3], radius: i32) -> u32 {
        let before = self.dirty.clone();
        let mut removed = 0;
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let voxel = [at[0] + dx, at[1] + dy, at[2] + dz];
                    if voxel[1] >= 1 && self.solid(voxel) {
                        self.place(voxel, AIR);
                        removed += 1;
                    }
                }
            }
        }
        if removed > 0 {
            self.revision += 1;
        } else {
            self.dirty = before;
        }
        removed
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The bricks changed since the last drain, for a projection's
    /// region-upload discipline. Draining is a projection act and does not
    /// touch the revision.
    pub fn drain_dirty(&mut self) -> Vec<[i16; 3]> {
        std::mem::take(&mut self.dirty).into_iter().collect()
    }

    pub fn brick_count(&self) -> usize {
        self.bricks.len()
    }

    /// One brick's materials as a mesh-crate volume, for the raster
    /// projection. `None` for a brick that was never touched (all air).
    pub fn brick_materials(&self, key: [i16; 3]) -> Option<(&Brick, [i32; 3])> {
        self.bricks.get(&key).map(|brick| {
            (
                brick,
                [
                    key[0] as i32 * BRICK,
                    key[1] as i32 * BRICK,
                    key[2] as i32 * BRICK,
                ],
            )
        })
    }

    pub fn keys(&self) -> impl Iterator<Item = [i16; 3]> + '_ {
        self.bricks.keys().copied()
    }

    /// The highest solid y at a column, if any solid exists there.
    pub fn surface(&self, x: i32, z: i32) -> Option<i32> {
        (0..SURFACE_BAND + 1).rev().find(|y| self.solid([x, *y, z]))
    }
}

impl Brick {
    /// The brick as raw material bytes, for building a mesh-crate volume.
    pub fn raw(&self) -> &[u8] {
        &self.materials
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat plain with one burrow: the smallest terrain that exercises both
    /// halves of the seam. Products bring their own; this file must not know
    /// about theirs.
    struct Flat {
        height: i32,
        cavity: Option<Cavity>,
    }

    impl Terrain for Flat {
        fn sea_level(&self, _extent: i32) -> i32 {
            1
        }

        fn surface(&self, _extent: i32, _x: i32, _z: i32) -> i32 {
            self.height
        }

        fn cavities(&self, _extent: i32) -> Vec<Cavity> {
            self.cavity.clone().into_iter().collect()
        }
    }

    fn plain() -> Ground {
        Ground::grow(
            &Flat {
                height: 6,
                cavity: None,
            },
            8,
        )
    }

    #[test]
    fn a_grown_plain_is_solid_to_its_surface_and_air_above() {
        let ground = plain();
        assert_eq!(ground.extent(), 8);
        assert_eq!(ground.sea_level, 1);
        assert!(ground.solid([0, 6, 0]));
        assert!(!ground.solid([0, 7, 0]));
        assert_eq!(ground.surface(0, 0), Some(6));
        assert!(ground.stands([0, 7, 0], 2));
    }

    #[test]
    fn growth_is_a_starting_fact_rather_than_an_edit() {
        let mut ground = plain();
        assert_eq!(ground.revision(), 0);
        assert!(ground.drain_dirty().is_empty());
    }

    #[test]
    fn a_cavity_is_hollowed_then_roofed_at_the_inner_end() {
        let ground = Ground::grow(
            &Flat {
                height: 6,
                cavity: Some(Cavity {
                    rooms: vec![([5, 3, 0], 1)],
                    route: vec![[0, 7, 0], [1, 6, 0], [2, 5, 0], [3, 4, 0]],
                }),
            },
            8,
        );
        assert!(!ground.solid([5, 3, 0]), "the room is a void");
        assert!(ground.solid([3, 6, 0]), "and it is roofed");
        for at in [[0, 7, 0], [1, 6, 0], [2, 5, 0], [3, 4, 0]] {
            assert!(ground.stands(at, 2), "the route at {at:?} is walkable");
        }
    }

    #[test]
    fn a_carve_is_a_revision_and_a_dirty_brick() {
        let mut ground = plain();
        let removed = ground.carve([0, 6, 0], 1);
        assert!(removed > 0);
        assert_eq!(ground.revision(), 1);
        assert!(!ground.drain_dirty().is_empty());
        // A carve that removes nothing leaves both alone.
        assert_eq!(ground.carve([0, 40, 0], 1), 0);
        assert_eq!(ground.revision(), 1);
    }

    #[test]
    fn draining_projection_dirt_does_not_change_the_bytes() {
        let mut ground = plain();
        ground.carve([0, 6, 0], 1);
        let before = crate::snapshot::encode(&ground).unwrap();
        ground.drain_dirty();
        assert_eq!(crate::snapshot::encode(&ground).unwrap(), before);
    }

    #[test]
    fn the_same_terrain_raises_the_same_ground() {
        let a = crate::snapshot::encode(&plain()).unwrap();
        assert_eq!(crate::snapshot::encode(&plain()).unwrap(), a);
        assert_eq!(
            crate::snapshot::decode::<Ground>(&a).unwrap().brick_count(),
            plain().brick_count()
        );
    }
}
