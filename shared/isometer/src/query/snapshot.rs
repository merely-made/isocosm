// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::bodies::BodyQueries;
use isometer_lens::{BrickMap, BrickRayError};
use std::sync::Arc;

/// Immutable queries for an exact encoded image, independent of a Scene's
/// next resize, camera, geometry, palette or terrain preparation. Capturing
/// owns the body queries and shares the current immutable terrain payload.
/// Encoding alone is not presentation: a host promotes this on acceptance.
#[derive(Clone)]
pub struct QuerySnapshot(Arc<Data>);

/// O(1) structural sizes of the captured query inputs. Dense terrain payload
/// excludes key-slot trees, allocator capacity, Arc headers and body storage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct QuerySnapshotStats {
    pub placed_bodies: usize,
    pub surfaces: usize,
    pub terrain_present: bool,
    /// This capture copied a new immutable map for its terrain payload.
    pub terrain_payload_copied: bool,
    pub terrain_pointer_words: usize,
    pub terrain_atlas_bytes: usize,
}

struct Data {
    frame: PresentedFrame,
    extent: [u32; 2],
    bodies: BodyQueries,
    maps: Vec<(Arc<BrickMap>, f32)>,
    terrain_payload_copied: bool,
    fallback: bool,
}

impl QuerySnapshot {
    pub(crate) fn new(
        frame: PresentedFrame,
        extent: [u32; 2],
        bodies: BodyQueries,
        maps: Vec<(Arc<BrickMap>, f32)>,
        terrain_payload_copied: bool,
        fallback: bool,
    ) -> Self {
        Self(Arc::new(Data {
            frame,
            extent,
            bodies,
            maps,
            terrain_payload_copied,
            fallback,
        }))
    }
    pub fn capture_stats(&self) -> QuerySnapshotStats {
        let (placed_bodies, surfaces) = self.0.bodies.record_counts();
        let terrain_pointer_words = self.0.maps.iter().map(|(m, _)| m.pointers().len()).sum();
        let terrain_atlas_bytes = self.0.maps.iter().map(|(m, _)| m.atlas().len()).sum();
        QuerySnapshotStats {
            placed_bodies,
            surfaces,
            terrain_present: !self.0.maps.is_empty(),
            terrain_payload_copied: self.0.terrain_payload_copied,
            terrain_pointer_words,
            terrain_atlas_bytes,
        }
    }
    pub fn generation(&self) -> u64 {
        self.0.frame.generation
    }
    pub fn camera(&self) -> SlabCamera {
        self.0.frame.camera
    }
    pub fn extent(&self) -> [u32; 2] {
        self.0.extent
    }
    pub fn validates_address(&self, address: PartAddress) -> bool {
        !self.0.fallback && self.0.bodies.contains(address)
    }
    pub fn validate_pick(&self, pick: BodyPick) -> bool {
        pick.frame == self.generation() && self.validates_address(pick.address)
    }
    pub fn select_part(
        &self,
        subject: SubjectKey,
        current: Option<PartAddress>,
        backwards: bool,
    ) -> Option<PartAddress> {
        if self.0.fallback {
            return None;
        }
        self.0.bodies.select_part(subject, current, backwards)
    }
    pub fn pick_at_pixel(&self, pixel: [u32; 2]) -> Result<Option<Pick>, BodyPickError> {
        let [w, h] = self.0.extent;
        if pixel[0] >= w || pixel[1] >= h {
            return Err(BodyPickError::InvalidCoordinates);
        }
        self.pick([
            2.0 * (pixel[0] as f32 + 0.5) / w as f32 - 1.0,
            1.0 - 2.0 * (pixel[1] as f32 + 0.5) / h as f32,
        ])
    }
    pub fn pick(&self, ndc: [f32; 2]) -> Result<Option<Pick>, BodyPickError> {
        if ndc
            .iter()
            .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        {
            return Err(BodyPickError::InvalidCoordinates);
        }
        if self.0.fallback {
            return Err(BodyPickError::CapsuleFallback);
        }
        let camera = self.camera().trace().ok_or(BodyPickError::NotReady)?;
        let (origin, direction) = camera
            .ray_at(ndc)
            .ok_or(BodyPickError::InvalidCoordinates)?;
        let body = self
            .0
            .bodies
            .pick(origin, direction, camera.far(), self.camera().clip())
            .map_err(BodyPickError::Body)?;
        let terrain = if self.0.frame.terrain {
            nearest_terrain(
                self.0.maps.iter().map(|(m, c)| (m.as_ref(), *c)),
                origin,
                direction,
                camera.far(),
            )
            .map_err(BodyPickError::Terrain)?
        } else {
            None
        };
        Ok(nearest(self.generation(), body, terrain))
    }
}

pub(crate) fn terrain_ray(
    map: Option<&BrickMap>,
    cell: f32,
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
) -> Result<Option<BrickRayHit>, BrickRayError> {
    match map {
        Some(map) => map
            .trace_ray(origin.map(|v| v / cell), direction, far / cell)
            .map(|hit| {
                hit.map(|mut h| {
                    h.distance *= cell;
                    h.point = h.point.map(|v| v * cell);
                    h.voxel = h.voxel.map(|v| v * cell as i32);
                    h
                })
            }),
        None => Ok(None),
    }
}

pub(crate) fn nearest_terrain<'a>(
    maps: impl IntoIterator<Item = (&'a BrickMap, f32)>,
    origin: [f32; 3],
    direction: [f32; 3],
    far: f32,
) -> Result<Option<BrickRayHit>, BrickRayError> {
    let mut best: Option<BrickRayHit> = None;
    for (map, cell) in maps {
        if let Some(hit) = terrain_ray(Some(map), cell, origin, direction, far)?
            && best.as_ref().is_none_or(|b| hit.distance < b.distance)
        {
            best = Some(hit);
        }
    }
    Ok(best)
}
