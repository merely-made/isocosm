// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where a scene's terrain comes from, addressed by revision.
//!
//! The scene never walks a world: it asks a [`TerrainSource`] for the brick
//! map once, then asks it each frame what moved. An unchanged ground answers
//! [`TerrainRefresh::Current`] and costs no CPU walk.

use std::cell::RefCell;

use isometer_core::ground::Ground;
use isometer_lens::BrickMap;

/// What one frame's refresh did to the map, and therefore what the tracer has
/// to re-upload.
///
/// Three answers rather than two, because "nothing was attempted" and "an
/// incremental upload was attempted" differ: the second leaves a full retry
/// pending until an encode consumes it, so a failed frame cannot strand a
/// partially uploaded map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TerrainRefresh {
    /// The map is already current. Upload nothing, pend nothing.
    Current,
    /// Only these atlas slots moved.
    Slots(Vec<u32>),
    /// The map was replaced wholesale; re-upload it.
    Full,
}

/// A terrain the scene traces against.
pub trait TerrainSource {
    /// Stamps the frame, so an unchanged terrain is recognised as unchanged.
    fn revision(&self) -> u64;

    /// The map itself, built once when the scene has none.
    fn brick_map(&self) -> Result<BrickMap, String>;

    /// Brings `map` up to date, given the bricks the host drained this frame.
    fn refresh(&self, map: &mut BrickMap, dirty: &[[i16; 3]]) -> Result<TerrainRefresh, String>;
}

/// The impl both products get for free: the shared [`Ground`], refreshed
/// incrementally from the host's drained dirty bricks.
pub struct GroundTerrain<'a>(pub &'a Ground);

impl super::residency::BrickSource for GroundTerrain<'_> {
    fn bounds(&self) -> Option<[[i16; 3]; 2]> {
        let mut keys = self.0.keys();
        let first = keys.next()?;
        Some(keys.fold([first, first], |mut bounds, key| {
            for axis in 0..3 {
                bounds[0][axis] = bounds[0][axis].min(key[axis]);
                bounds[1][axis] = bounds[1][axis].max(key[axis]);
            }
            bounds
        }))
    }

    fn layers(&self, column: [i16; 2]) -> std::ops::Range<i16> {
        let mut layers = self
            .0
            .keys()
            .filter(|key| [key[0], key[2]] == column)
            .map(|key| key[1]);
        let Some(first) = layers.next() else {
            return 0..0;
        };
        let (low, high) = layers.fold((first, first), |(lo, hi), y| (lo.min(y), hi.max(y)));
        low..high.saturating_add(1)
    }

    fn fill(&self, key: [i16; 3], out: &mut [u8]) {
        out.fill(0);
        if let Some((brick, _)) = self.0.brick_materials(key) {
            out.copy_from_slice(brick.raw());
        }
    }
}

impl TerrainSource for GroundTerrain<'_> {
    fn revision(&self) -> u64 {
        self.0.revision()
    }

    fn brick_map(&self) -> Result<BrickMap, String> {
        BrickMap::from_ground(self.0).map_err(|error| error.to_string())
    }

    fn refresh(&self, map: &mut BrickMap, dirty: &[[i16; 3]]) -> Result<TerrainRefresh, String> {
        if dirty.is_empty() {
            return Ok(TerrainRefresh::Current);
        }
        map.refresh(self.0, dirty.iter().copied())
            .map(TerrainRefresh::Slots)
            .map_err(|error| error.to_string())
    }
}

/// A terrain whose map the host builds itself.
///
/// A filtered or cut-away map — Mesocosm's terrarium rebuild — is not
/// derivable from the ground alone, so the host hands the replacement over and
/// the scene re-uploads it whole. With no replacement the scene keeps the map
/// it already holds; the host's own cache decided nothing changed.
pub struct HostTerrain {
    revision: u64,
    rebuilt: RefCell<Option<BrickMap>>,
}

impl HostTerrain {
    pub fn new(revision: u64, rebuilt: Option<BrickMap>) -> Self {
        Self {
            revision,
            rebuilt: RefCell::new(rebuilt),
        }
    }
}

impl TerrainSource for HostTerrain {
    fn revision(&self) -> u64 {
        self.revision
    }

    fn brick_map(&self) -> Result<BrickMap, String> {
        self.rebuilt
            .borrow_mut()
            .take()
            .ok_or_else(|| "host terrain supplied no brick map".to_string())
    }

    fn refresh(&self, map: &mut BrickMap, _dirty: &[[i16; 3]]) -> Result<TerrainRefresh, String> {
        match self.rebuilt.borrow_mut().take() {
            Some(rebuilt) => {
                *map = rebuilt;
                Ok(TerrainRefresh::Full)
            },
            None => Ok(TerrainRefresh::Current),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PagedTerrain, Residency, ResidencySettings, SlabCamera, framed_bricks};
    use isometer_core::ground::{BRICK, Terrain};
    use isometer_lens::bricks::AtlasLimits;

    #[test]
    fn a_ground_larger_than_the_packed_atlas_pages_and_preserves_materials() {
        struct Flat;
        impl Terrain for Flat {
            fn sea_level(&self, _: i32) -> i32 {
                -1
            }
            fn surface(&self, _: i32, _: i32, _: i32) -> i32 {
                15
            }
        }
        let ground = Ground::grow_with(&Flat, 128, |x, z, depth| {
            ((x + z + depth).rem_euclid(62) + 1) as u8
        });
        assert!(
            BrickMap::from_ground(&ground).is_err(),
            "exceeds the legacy packed atlas"
        );
        let source = GroundTerrain(&ground);
        let residency = RefCell::new(Residency::new(
            ResidencySettings::default(),
            AtlasLimits::DEFAULT,
        ));
        let mut map = None;
        for x in [-80.0, 80.0] {
            let camera = SlabCamera::dimetric_2_1([x, 8.0, 0.0], 16.0, 1.5, 64.0).unwrap();
            let framed =
                framed_bricks(camera, BRICK as f32, &source, residency.borrow().capacity());
            assert_eq!(framed.overflow, 0);
            let paged = PagedTerrain::new(&residency, &source, &framed, ground.revision());
            if let Some(map) = &mut map {
                paged.refresh(map, &[]).unwrap();
            } else {
                map = Some(paged.brick_map().unwrap());
            }
            let map = map.as_ref().unwrap();
            assert!(residency.borrow().stats().resident > 0);
            for key in residency.borrow().resident() {
                for y in 0..BRICK {
                    for z in 0..BRICK {
                        for x in 0..BRICK {
                            let at = [
                                i32::from(key[0]) * BRICK + x,
                                i32::from(key[1]) * BRICK + y,
                                i32::from(key[2]) * BRICK + z,
                            ];
                            assert_eq!(map.material_at(at), ground.material(at).unwrap_or(0));
                        }
                    }
                }
            }
        }
    }
}
