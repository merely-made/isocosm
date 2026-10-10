// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V1: the Eponym-owned residency policy pulled by continuous zoom.
//!
//! Mesocosm owns exact `Ground` and the brick-map allocation. Eponym owns
//! which exact bricks its current camera needs and the budget that bounds that
//! presentation working set.

use std::{collections::BTreeSet, fmt};

use glam::Vec3;
use isocosm::legacy::mesocosm::places::{Places, WALKER_HEIGHT};
use isometer::core::ground::{BRICK, Ground};
use isometer::lens::TraceCamera;
use modulus::{AtlasLimits, BrickMap, BrickMapError, BrickProjectionRevision};

use crate::room::SEED;

/// A region-sized world whose complete exact brick set does not fit the
/// tracer atlas. The working set must therefore be selected, not accidental.
pub const WORLD_EXTENT: i32 = 256;
const WORLD_SIDE: u16 = 16;
/// Close, leading, and planning working-set radii in world voxels.
pub const PAGE_RANGES: [i32; 3] = [40, 88, 128];
/// One MiB for exact terrain pointers and material atlas at the far profile.
pub const RESIDENT_BUDGET_BYTES: u64 = 1_048_576;
/// The deterministic headed zoom and recovery trace.
pub const V1_FRAMES: u64 = 96;
pub const CLOSE_DISTANCE: f32 = 8.0;
pub const FAR_DISTANCE: f32 = 72.0;
const FOV_Y: f32 = std::f32::consts::FRAC_PI_3;
const CLOSE_ELEVATION: f32 = 50.0_f32.to_radians();
const FAR_ELEVATION: f32 = 65.0_f32.to_radians();
/// The camera targets the character's upper body. Residency covers the
/// ground plane beneath it rather than only that target plane.
const TARGET_ABOVE_GROUND: f32 = 1.1;

pub struct ResidencyScene {
    pub ground: Ground,
    pub focus: [i32; 3],
}

impl ResidencyScene {
    pub fn grow() -> Self {
        let grown = Places::grown(SEED, WORLD_SIDE, WORLD_EXTENT);
        let ground = Ground::grow(&grown, WORLD_EXTENT);
        let focus = (0_i32..=64)
            .find_map(|ring| {
                (-ring..=ring).find_map(|z| {
                    (-ring..=ring).find_map(|x| {
                        if x.abs().max(z.abs()) != ring {
                            return None;
                        }
                        let top = ground.surface(x, z)?;
                        let at = [x, top + 1, z];
                        ground.stands(at, WALKER_HEIGHT).then_some(at)
                    })
                })
            })
            .expect("the grown planning region has a surface stance");
        Self { ground, focus }
    }

    /// Moves the camera/body focus to the surface at a horizontal offset.
    pub fn move_focus_x(&mut self, delta: i32) -> bool {
        let x = self.focus[0] + delta;
        let z = self.focus[2];
        let Some(top) = self.ground.surface(x, z) else {
            return false;
        };
        let next = [x, top + 1, z];
        if !self.ground.stands(next, WALKER_HEIGHT) {
            return false;
        }
        self.focus = next;
        true
    }

    pub fn camera(&self, distance: f32, aspect: f32) -> TraceCamera {
        let target = Vec3::new(
            self.focus[0] as f32 + 0.5,
            self.focus[1] as f32 + 1.1,
            self.focus[2] as f32 + 0.5,
        );
        let direction = camera_direction(distance);
        let eye = target + direction * distance;
        TraceCamera::perspective(
            eye.to_array(),
            target.to_array(),
            Vec3::Y.to_array(),
            FOV_Y,
            aspect,
            distance + 192.0,
        )
        .expect("the V1 planning camera is valid")
    }
}

pub fn zoom_distance(frame: u64) -> f32 {
    match frame {
        0..=11 => CLOSE_DISTANCE,
        12..=35 => {
            let t = (frame - 12) as f32 / 23.0;
            let smooth = t * t * (3.0 - 2.0 * t);
            CLOSE_DISTANCE + (FAR_DISTANCE - CLOSE_DISTANCE) * smooth
        },
        36..=47 => FAR_DISTANCE,
        48..=59 => CLOSE_DISTANCE,
        _ => FAR_DISTANCE,
    }
}

pub fn visible_range(distance: f32, aspect: f32) -> i32 {
    let outward = camera_direction(distance);
    let forward = -outward;
    let right = forward.cross(Vec3::Y).normalize();
    let up = right.cross(forward).normalize();
    let half_height = (FOV_Y * 0.5).tan();
    let eye = outward * distance;
    let mut radius = 0.0_f32;
    for vertical in [-1.0, 1.0] {
        for horizontal in [-1.0, 1.0] {
            let ray =
                (forward + right * half_height * aspect * horizontal + up * half_height * vertical)
                    .normalize();
            assert!(ray.y < 0.0, "the V1 camera horizon must meet the ground");
            let travel = (-TARGET_ABOVE_GROUND - eye.y) / ray.y;
            let ground = eye + ray * travel;
            radius = radius.max(ground.x.abs()).max(ground.z.abs());
        }
    }
    radius.ceil() as i32 + BRICK
}

fn camera_direction(distance: f32) -> Vec3 {
    let zoom = ((distance - CLOSE_DISTANCE) / (FAR_DISTANCE - CLOSE_DISTANCE)).clamp(0.0, 1.0);
    let elevation = CLOSE_ELEVATION + (FAR_ELEVATION - CLOSE_ELEVATION) * zoom;
    let horizontal = elevation.cos() * std::f32::consts::FRAC_1_SQRT_2;
    Vec3::new(horizontal, elevation.sin(), horizontal)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResidencyMetrics {
    pub projection_revision: BrickProjectionRevision,
    pub visible_range: i32,
    pub resident_range: i32,
    pub loaded_bricks: usize,
    pub evicted_bricks: usize,
    pub resident_bricks: usize,
    pub pointer_bytes: u64,
    pub atlas_bytes: u64,
    pub resident_bytes: u64,
}

pub struct PreparedWorkingSet {
    pub map: BrickMap,
    pub metrics: ResidencyMetrics,
}

#[derive(Debug)]
pub enum ResidencyError {
    VisibleRange { actual: i32, maximum: i32 },
    Budget { actual: u64, maximum: u64 },
    ProjectionRevisionOverflow,
    BrickMap(BrickMapError),
}

impl fmt::Display for ResidencyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VisibleRange { actual, maximum } => write!(
                formatter,
                "visible range {actual} exceeds the largest resident page range {maximum}"
            ),
            Self::Budget { actual, maximum } => write!(
                formatter,
                "working set uses {actual} bytes, over the {maximum}-byte V1 budget"
            ),
            Self::ProjectionRevisionOverflow => {
                write!(formatter, "brick projection revision overflow")
            },
            Self::BrickMap(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ResidencyError {}

impl From<BrickMapError> for ResidencyError {
    fn from(error: BrickMapError) -> Self {
        Self::BrickMap(error)
    }
}

#[derive(Default)]
pub struct ResidencyPolicy {
    current_range: Option<i32>,
    current_keys: BTreeSet<[i16; 3]>,
    projection_revision: BrickProjectionRevision,
}

impl ResidencyPolicy {
    pub fn current_range(&self) -> Option<i32> {
        self.current_range
    }

    pub fn projection_revision(&self) -> BrickProjectionRevision {
        self.projection_revision
    }

    pub fn prepare(
        &mut self,
        scene: &ResidencyScene,
        visible_range: i32,
    ) -> Result<Option<PreparedWorkingSet>, ResidencyError> {
        let resident_range = PAGE_RANGES
            .into_iter()
            .find(|range| *range >= visible_range)
            .ok_or(ResidencyError::VisibleRange {
                actual: visible_range,
                maximum: *PAGE_RANGES.last().expect("V1 has residency bands"),
            })?;
        let keys = selected_keys(&scene.ground, scene.focus, resident_range);
        if self.current_range == Some(resident_range) && self.current_keys == keys {
            return Ok(None);
        }
        let projection_revision = if self.current_range.is_none() {
            self.projection_revision
        } else {
            BrickProjectionRevision(
                self.projection_revision
                    .0
                    .checked_add(1)
                    .ok_or(ResidencyError::ProjectionRevisionOverflow)?,
            )
        };
        let map = crate::brick::from_ground_keys(
            &scene.ground,
            projection_revision,
            keys.iter().copied(),
        )?;
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
        Ok(Some(PreparedWorkingSet { map, metrics }))
    }
}

/// V1b: the stable residency policy over one capacity-fixed brick cache.
///
/// Where [`ResidencyPolicy`] builds a fresh map per selection,
/// this owns one [`BrickMap`] whose pointer and atlas extents never change:
/// band transitions and same-band travel retarget it in place, retained
/// bricks keep their atlas slots, and the outcome names exactly the loaded
/// slots a consumer must publish. The fixed allocation is sized under
/// [`RESIDENT_BUDGET_BYTES`] at construction, so the budget bounds the
/// cache rather than each page.
pub struct StableResidency {
    map: BrickMap,
    current_range: Option<i32>,
    current_keys: BTreeSet<[i16; 3]>,
    projection_revision: BrickProjectionRevision,
    pointer_bytes: u64,
    atlas_bytes: u64,
}

pub struct StableOutcome {
    pub delta: modulus::RetargetDelta,
    pub metrics: ResidencyMetrics,
}

impl StableResidency {
    pub fn new(scene: &ResidencyScene) -> Result<Self, ResidencyError> {
        Self::with_limits(scene, AtlasLimits::DEFAULT)
    }

    /// Uses the host's enforced texture limit and atlas ceiling, while the
    /// pointer volume and atlas together still fit the V1 residency budget.
    pub fn with_limits(
        scene: &ResidencyScene,
        mut limits: AtlasLimits,
    ) -> Result<Self, ResidencyError> {
        // The fixed pointer box: the widest band's horizontal reach,
        // clamped by the world, with the world's whole vertical brick span
        // (selection never filters by height).
        let mut low = [i16::MAX; 3];
        let mut high = [i16::MIN; 3];
        for key in scene.ground.keys() {
            for axis in 0..3 {
                low[axis] = low[axis].min(key[axis]);
                high[axis] = high[axis].max(key[axis]);
            }
        }
        let world = [0, 1, 2].map(|axis| (i32::from(high[axis]) - i32::from(low[axis]) + 1) as u32);
        let widest = *PAGE_RANGES.last().expect("V1 has residency bands");
        let reach = (2 * widest / BRICK + 2) as u32;
        let pointer_extent = [world[0].min(reach), world[1], world[2].min(reach)];
        let pointer_bytes = u64::from(pointer_extent[0])
            * u64::from(pointer_extent[1])
            * u64::from(pointer_extent[2])
            * 4;
        // The atlas takes every whole row the combined budget leaves the
        // pointer volume, bounded further by the host's atlas ceiling.
        limits.max_atlas_bytes = limits
            .max_atlas_bytes
            .min(RESIDENT_BUDGET_BYTES.saturating_sub(pointer_bytes));
        let map = BrickMap::with_limits(
            BrickProjectionRevision(0),
            limits.max_bricks(),
            pointer_extent,
            limits,
        )?;
        let atlas_bytes = map.atlas().len() as u64;
        debug_assert!(pointer_bytes + atlas_bytes <= RESIDENT_BUDGET_BYTES);
        Ok(Self {
            map,
            current_range: None,
            current_keys: BTreeSet::new(),
            projection_revision: BrickProjectionRevision(0),
            pointer_bytes,
            atlas_bytes,
        })
    }

    pub fn map(&self) -> &BrickMap {
        &self.map
    }

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// The fixed allocation the budget bounds: pointer volume plus atlas.
    pub const fn resident_bytes(&self) -> u64 {
        self.pointer_bytes + self.atlas_bytes
    }

    /// Selects the band for the visible range and retargets the stable
    /// cache when the selection changed. `None` means the resident page
    /// already covers this frame.
    pub fn prepare(
        &mut self,
        scene: &ResidencyScene,
        visible_range: i32,
    ) -> Result<Option<StableOutcome>, ResidencyError> {
        let resident_range = PAGE_RANGES
            .into_iter()
            .find(|range| *range >= visible_range)
            .ok_or(ResidencyError::VisibleRange {
                actual: visible_range,
                maximum: *PAGE_RANGES.last().expect("V1 has residency bands"),
            })?;
        let keys = selected_keys(&scene.ground, scene.focus, resident_range);
        if self.current_range == Some(resident_range) && self.current_keys == keys {
            return Ok(None);
        }
        let projection_revision = BrickProjectionRevision(
            self.projection_revision
                .0
                .checked_add(1)
                .ok_or(ResidencyError::ProjectionRevisionOverflow)?,
        );
        let delta = crate::brick::retarget_from_ground(
            &mut self.map,
            &scene.ground,
            projection_revision,
            keys.iter().copied(),
        )?;
        let metrics = ResidencyMetrics {
            projection_revision,
            visible_range,
            resident_range,
            loaded_bricks: delta.loaded_slots.len(),
            evicted_bricks: delta.evicted,
            resident_bricks: keys.len(),
            pointer_bytes: self.pointer_bytes,
            atlas_bytes: self.atlas_bytes,
            resident_bytes: self.resident_bytes(),
        };
        self.current_range = Some(resident_range);
        self.current_keys = keys;
        self.projection_revision = projection_revision;
        Ok(Some(StableOutcome { delta, metrics }))
    }
}

fn selected_keys(ground: &Ground, focus: [i32; 3], range: i32) -> BTreeSet<[i16; 3]> {
    ground
        .keys()
        .filter(|key| {
            [0, 2].into_iter().all(|axis| {
                let low = i32::from(key[axis]) * BRICK;
                let high = low + BRICK - 1;
                high >= focus[axis] - range && low <= focus[axis] + range
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;
