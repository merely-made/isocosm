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
use mesocosm_lens::BrickMap;

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
