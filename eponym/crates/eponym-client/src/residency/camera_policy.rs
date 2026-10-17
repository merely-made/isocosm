// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Camera-exact V1 selection paths, separate from ordinary R1 producer pages.

use super::*;

impl ResidencyPolicy {
    /// Selects every brick hit by the supplied camera's actual pixel rays.
    /// The CPU-only full-Ground source map is cached separately from the
    /// one-MiB resident presentation map.
    pub fn prepare_view(
        &mut self,
        scene: &ResidencyScene,
        visible_range: i32,
        camera: TraceCamera,
        size: [u32; 2],
    ) -> Result<Option<PreparedWorkingSet>, ResidencyError> {
        let resident_range = page_range(visible_range)?;
        let ground_revision = scene.ground.revision();
        let world_changed = self.camera_ground_revision != Some(ground_revision)
            || self.camera_focus != Some(scene.focus);
        let mut keys = self
            .footprints
            .keys(&scene.ground, scene.focus, camera, size)?;
        let mut proposed_anchor = self.near_anchor_keys.clone();
        if world_changed {
            proposed_anchor.clear();
        }
        if resident_range == PAGE_RANGES[0]
            && (world_changed || self.current_range != Some(PAGE_RANGES[0]))
        {
            proposed_anchor.clone_from(&keys);
        } else if resident_range > PAGE_RANGES[0] {
            keys.extend(proposed_anchor.iter().copied());
        }

        let selection_changed = self.current_range != Some(resident_range)
            || self.current_keys != keys
            || world_changed;
        if !selection_changed {
            return Ok(None);
        }
        let next_projection = self
            .projection_revision
            .0
            .checked_add(1)
            .ok_or(ResidencyError::ProjectionRevisionOverflow)?;
        let projection_revision = BrickProjectionRevision(next_projection);
        let map = self.footprints.resident_map(
            &scene.ground,
            projection_revision,
            keys.iter().copied(),
        )?;
        let projection_revision = map.projection_revision();
        let loaded_bricks = keys.difference(&self.current_keys).count();
        let evicted_bricks = self.current_keys.difference(&keys).count();
        let pointer_bytes = std::mem::size_of_val(map.pointers()) as u64;
        let atlas_bytes = map.atlas().len() as u64;
        let resident_bytes = pointer_bytes + atlas_bytes;
        if resident_bytes > RESIDENT_BUDGET_BYTES {
            return Err(ResidencyError::Budget {
                actual: resident_bytes,
                maximum: RESIDENT_BUDGET_BYTES,
            });
        }
        let metrics = ResidencyMetrics {
            projection_revision,
            visible_range,
            resident_range,
            loaded_bricks,
            evicted_bricks,
            resident_bricks: keys.len(),
            pointer_bytes,
            atlas_bytes,
            resident_bytes,
        };
        self.current_range = Some(resident_range);
        self.current_keys = keys;
        self.projection_revision = projection_revision;
        self.camera_ground_revision = Some(ground_revision);
        self.camera_focus = Some(scene.focus);
        self.near_anchor_keys = proposed_anchor;
        Ok(Some(PreparedWorkingSet { map, metrics }))
    }
}

impl StableResidency {
    /// Camera-exact counterpart to `prepare_at`, reserved for V1's
    /// continuous-zoom view. The stable GPU cache keeps its allocation while
    /// the bounded CPU source map computes exact visible brick keys.
    pub fn prepare_view(
        &mut self,
        scene: &ResidencyScene,
        visible_range: i32,
        camera: TraceCamera,
        size: [u32; 2],
    ) -> Result<Option<StableOutcome>, ResidencyError> {
        let resident_range = page_range(visible_range)?;
        let ground = &scene.ground;
        let focus = scene.focus;
        let ground_changed = self.ground_revision != Some(ground.revision());
        let world_changed = ground_changed || self.camera_focus != Some(focus);
        let mut keys = self.footprints.keys(ground, focus, camera, size)?;
        let mut proposed_anchor = self.near_anchor_keys.clone();
        if world_changed {
            proposed_anchor.clear();
        }
        if resident_range == PAGE_RANGES[0]
            && (world_changed || self.current_range != Some(PAGE_RANGES[0]))
        {
            proposed_anchor.clone_from(&keys);
        } else if resident_range > PAGE_RANGES[0] {
            keys.extend(proposed_anchor.iter().copied());
        }
        let selection_changed = self.current_range != Some(resident_range)
            || self.current_keys != keys
            || world_changed;
        if !selection_changed {
            return Ok(None);
        }
        let current_projection = self.map.projection_revision();
        let target_projection = if keys == self.current_keys {
            current_projection
        } else {
            BrickProjectionRevision(
                current_projection
                    .0
                    .checked_add(1)
                    .ok_or(ResidencyError::ProjectionRevisionOverflow)?,
            )
        };
        let mut delta = crate::brick::retarget_from_ground(
            &mut self.map,
            ground,
            target_projection,
            keys.iter().copied(),
        )?;
        let projection_revision = self.map.projection_revision();
        let loaded_bricks = delta.loaded_slots.len();
        if self.ground_revision.is_some() && ground_changed && !keys.is_empty() {
            let refreshed = self.map.refresh(keys.iter().copied(), |key| {
                ground.brick_materials(key).map(|(brick, _)| brick.raw())
            })?;
            delta.loaded_slots.extend(refreshed);
            delta.loaded_slots.sort_unstable();
            delta.loaded_slots.dedup();
        }
        let metrics = ResidencyMetrics {
            projection_revision,
            visible_range,
            resident_range,
            loaded_bricks,
            evicted_bricks: delta.evicted,
            resident_bricks: keys.len(),
            pointer_bytes: self.pointer_bytes,
            atlas_bytes: self.atlas_bytes,
            resident_bytes: self.resident_bytes(),
        };
        self.current_range = Some(resident_range);
        self.current_keys = keys;
        self.projection_revision = projection_revision;
        self.ground_revision = Some(ground.revision());
        self.camera_focus = Some(focus);
        self.near_anchor_keys = proposed_anchor;
        Ok(Some(StableOutcome { delta, metrics }))
    }
}
