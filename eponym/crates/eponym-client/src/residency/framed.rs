// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The native scene's camera selection over Eponym's fixed terrain budget.

use isometer::lens::BrickMap as SceneMap;
use isometer::lens::bricks::BrickQueryVersion;
use isometer::{GroundTerrain, SlabCamera, TerrainRefresh, TerrainSource, framed_bricks};

use super::{ResidencyError, StableOutcome, StableResidency, camera_visible_range};
use isometer::core::ground::Ground;

impl StableResidency {
    pub(crate) fn resident_count(&self) -> usize {
        self.current_keys.len()
    }

    pub(crate) fn prepare_framed(
        &mut self,
        ground: &Ground,
        camera: SlabCamera,
    ) -> Result<FramedTerrain<'_>, ResidencyError> {
        let previous = self.map.query_version().clone();
        let keys = framed_bricks(camera, 0.0, &GroundTerrain(ground), usize::MAX)
            .keys
            .into_iter()
            .filter(|&key| ground.brick_materials(key).is_some())
            .collect();
        let changed =
            if self.ground_revision == Some(ground.revision()) && self.current_keys == keys {
                None
            } else {
                // Retarget checks capacity and pointer extent before changing the map.
                let range = camera_visible_range(camera);
                self.prepare_keys(ground, keys, range, range)?
            };
        Ok(FramedTerrain {
            residency: self,
            changed,
            previous,
        })
    }
}

/// A frame's fixed-cache projection, with a slot delta relative to the
/// immediately preceding map. The scene owns publication and retry.
pub(crate) struct FramedTerrain<'a> {
    pub residency: &'a StableResidency,
    pub changed: Option<StableOutcome>,
    previous: BrickQueryVersion,
}

impl TerrainSource for FramedTerrain<'_> {
    fn revision(&self) -> u64 {
        self.residency.projection_revision().0
    }

    fn brick_map(&self) -> Result<SceneMap, String> {
        Ok(self.residency.map.clone())
    }

    fn refresh(&self, map: &mut SceneMap, _dirty: &[[i16; 3]]) -> Result<TerrainRefresh, String> {
        if map.query_version() == self.residency.map.query_version() {
            return Ok(TerrainRefresh::Current);
        }
        // A retained-slot material edit changes the payload without changing
        // the map's projection revision. Match the actual payload identity.
        let delta = self
            .changed
            .as_ref()
            .filter(|_| map.query_version() == &self.previous);
        *map = self.brick_map()?;
        Ok(match delta {
            Some(change) => TerrainRefresh::Slots(change.delta.loaded_slots.clone()),
            None => TerrainRefresh::Full,
        })
    }
}
