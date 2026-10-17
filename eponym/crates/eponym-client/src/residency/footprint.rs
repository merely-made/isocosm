// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded CPU source-map tracing for exact camera residency.

use std::collections::{BTreeSet, VecDeque};

use isometer::core::ground::{BRICK, Ground};
use isometer::lens::TraceCamera;
use modulus::{AtlasLimits, BrickMap, BrickProjectionRevision, BrickTrace};

use super::{CPU_SOURCE_MAP_BUDGET_BYTES, ResidencyError};

const SOURCE_ATLAS_BUDGET_BYTES: u64 = 32 * 1024 * 1024;
const MAX_CACHED_VIEWS: usize = 128;
const MAX_TRACE_PIXELS: usize = 4 * 1024 * 1024;

type BrickKey = [i16; 3];

struct CachedView {
    ground_revision: u64,
    focus: [i32; 3],
    camera: TraceCamera,
    size: [u32; 2],
    keys: Vec<BrickKey>,
    bytes: usize,
}

/// The source map and exact view-key cache are CPU-only. The source map is
/// independently capped from the GPU's one-MiB resident presentation map.
#[derive(Default)]
pub(super) struct CameraFootprints {
    ground_revision: Option<u64>,
    source: Option<BrickMap>,
    source_bytes: usize,
    views: VecDeque<CachedView>,
    view_bytes: usize,
}

impl CameraFootprints {
    pub(super) fn keys(
        &mut self,
        ground: &Ground,
        focus: [i32; 3],
        camera: TraceCamera,
        size: [u32; 2],
    ) -> Result<BTreeSet<BrickKey>, ResidencyError> {
        (size[0] as usize)
            .checked_mul(size[1] as usize)
            .filter(|pixels| *pixels > 0 && *pixels <= MAX_TRACE_PIXELS)
            .ok_or(ResidencyError::InvalidViewport { size })?;
        self.ensure_source(ground)?;
        let revision = ground.revision();
        if let Some(index) = self.views.iter().position(|view| {
            view.ground_revision == revision
                && view.focus == focus
                && view.camera == camera
                && view.size == size
        }) {
            let view = self.views.remove(index).expect("cached view index exists");
            let keys = view.keys.iter().copied().collect();
            self.views.push_back(view);
            return Ok(keys);
        }

        let source = self.source.as_ref().expect("source map was built");
        let mut keys = BTreeSet::new();
        for y in 0..size[1] {
            for x in 0..size[0] {
                let ndc = [
                    2.0 * (x as f32 + 0.5) / size[0] as f32 - 1.0,
                    1.0 - 2.0 * (y as f32 + 0.5) / size[1] as f32,
                ];
                let (origin, direction) = camera
                    .ray_at(ndc)
                    .ok_or(ResidencyError::InvalidCameraRay { x, y })?;
                match source.trace(origin, direction, camera.far()) {
                    BrickTrace::Hit(hit) => {
                        keys.insert(hit.voxel.map(|coordinate| {
                            i16::try_from(coordinate.div_euclid(BRICK))
                                .expect("Ground brick coordinates fit i16")
                        }));
                    },
                    BrickTrace::Clear => {},
                    BrickTrace::Exhausted => {
                        return Err(ResidencyError::TraceExhausted { x, y });
                    },
                }
            }
        }
        let keys: Vec<_> = keys.into_iter().collect();
        let bytes = std::mem::size_of::<CachedView>().saturating_add(
            keys.capacity()
                .saturating_mul(std::mem::size_of::<BrickKey>()),
        );
        if self.source_bytes.saturating_add(bytes) > CPU_SOURCE_MAP_BUDGET_BYTES {
            return Err(ResidencyError::CpuSourceBudget {
                actual: self.source_bytes.saturating_add(bytes),
                maximum: CPU_SOURCE_MAP_BUDGET_BYTES,
            });
        }
        while self.views.len() >= MAX_CACHED_VIEWS
            || self
                .source_bytes
                .saturating_add(self.view_bytes)
                .saturating_add(bytes)
                > CPU_SOURCE_MAP_BUDGET_BYTES
        {
            let Some(oldest) = self.views.pop_front() else {
                return Err(ResidencyError::CpuSourceBudget {
                    actual: self.source_bytes.saturating_add(bytes),
                    maximum: CPU_SOURCE_MAP_BUDGET_BYTES,
                });
            };
            self.view_bytes = self.view_bytes.saturating_sub(oldest.bytes);
        }
        self.view_bytes = self.view_bytes.saturating_add(bytes);
        self.views.push_back(CachedView {
            ground_revision: revision,
            focus,
            camera,
            size,
            keys: keys.clone(),
            bytes,
        });
        Ok(keys.into_iter().collect())
    }

    pub(super) fn resident_map(
        &self,
        ground: &Ground,
        projection_revision: BrickProjectionRevision,
        keys: impl IntoIterator<Item = BrickKey>,
    ) -> Result<BrickMap, ResidencyError> {
        let source = self.source.as_ref().expect("source map was built");
        let pointer_extent = source.pointer_extent();
        let pointer_bytes = pointer_extent
            .into_iter()
            .try_fold(1_usize, |volume, axis| volume.checked_mul(axis as usize))
            .and_then(|volume| volume.checked_mul(std::mem::size_of::<u32>()))
            .ok_or(ResidencyError::CpuSourceBudget {
                actual: usize::MAX,
                maximum: CPU_SOURCE_MAP_BUDGET_BYTES,
            })?;
        let atlas_budget = (super::RESIDENT_BUDGET_BYTES as usize).saturating_sub(pointer_bytes);
        let limits = AtlasLimits {
            max_texture_dimension_3d: 2048,
            max_atlas_bytes: atlas_budget as u64,
        };
        let base_revision = BrickProjectionRevision(
            projection_revision
                .0
                .checked_sub(1)
                .ok_or(ResidencyError::ProjectionRevisionOverflow)?,
        );
        let mut map =
            BrickMap::with_limits(base_revision, limits.max_bricks(), pointer_extent, limits)?;
        map.retarget(projection_revision, keys, |key| {
            ground.brick_materials(key).map(|(brick, _)| brick.raw())
        })?;
        Ok(map)
    }

    #[cfg(test)]
    pub(super) fn source_map(&self) -> &BrickMap {
        self.source.as_ref().expect("source map was built")
    }

    fn ensure_source(&mut self, ground: &Ground) -> Result<(), ResidencyError> {
        let revision = ground.revision();
        if self.ground_revision == Some(revision) && self.source.is_some() {
            return Ok(());
        }
        self.source = None;
        self.ground_revision = None;
        self.source_bytes = 0;
        self.views.clear();
        self.view_bytes = 0;

        let keys: Vec<_> = ground.keys().collect();
        let low = [0, 1, 2].map(|axis| {
            keys.iter()
                .map(|key| i32::from(key[axis]))
                .min()
                .expect("Ground has at least one brick")
        });
        let high = [0, 1, 2].map(|axis| {
            keys.iter()
                .map(|key| i32::from(key[axis]))
                .max()
                .expect("Ground has at least one brick")
        });
        let pointer_extent = [0, 1, 2].map(|axis| (high[axis] - low[axis] + 1) as u32);
        let limits = AtlasLimits {
            max_texture_dimension_3d: 2048,
            max_atlas_bytes: SOURCE_ATLAS_BUDGET_BYTES,
        };
        let pointer_bytes = pointer_extent
            .into_iter()
            .try_fold(1_usize, |volume, axis| volume.checked_mul(axis as usize))
            .and_then(|volume| volume.checked_mul(std::mem::size_of::<u32>()));
        let row_slots = (modulus::ATLAS_SLOTS_X * modulus::ATLAS_SLOTS_Z) as usize;
        let atlas_bytes = keys
            .len()
            .checked_add(1)
            .and_then(|bricks| bricks.div_ceil(row_slots).checked_mul(row_slots))
            .and_then(|slots| slots.checked_mul(modulus::BRICK_EDGE.pow(3) as usize));
        let source_bytes = pointer_bytes
            .and_then(|pointer| atlas_bytes.and_then(|atlas| pointer.checked_add(atlas)))
            .ok_or(ResidencyError::CpuSourceBudget {
                actual: usize::MAX,
                maximum: CPU_SOURCE_MAP_BUDGET_BYTES,
            })?;
        if pointer_extent
            .iter()
            .any(|axis| *axis > limits.max_texture_dimension_3d)
            || atlas_bytes.is_none_or(|bytes| bytes as u64 > limits.max_atlas_bytes)
            || source_bytes > CPU_SOURCE_MAP_BUDGET_BYTES
        {
            return Err(ResidencyError::CpuSourceBudget {
                actual: source_bytes,
                maximum: CPU_SOURCE_MAP_BUDGET_BYTES,
            });
        }
        let mut source = BrickMap::with_limits(
            BrickProjectionRevision(0),
            keys.len(),
            pointer_extent,
            limits,
        )?;
        source.retarget(BrickProjectionRevision(1), keys.iter().copied(), |key| {
            ground.brick_materials(key).map(|(brick, _)| brick.raw())
        })?;
        let actual_source_bytes = source
            .pointers()
            .len()
            .saturating_mul(std::mem::size_of::<u32>())
            .saturating_add(source.atlas().len());
        if actual_source_bytes > CPU_SOURCE_MAP_BUDGET_BYTES || actual_source_bytes > source_bytes {
            return Err(ResidencyError::CpuSourceBudget {
                actual: actual_source_bytes,
                maximum: CPU_SOURCE_MAP_BUDGET_BYTES,
            });
        }
        self.source = Some(source);
        self.source_bytes = source_bytes;
        self.ground_revision = Some(revision);
        Ok(())
    }
}
