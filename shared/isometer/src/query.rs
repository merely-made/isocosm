// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Asking a completed frame what it drew.
//!
//! Every answer here is addressed the way the draw was: a [`SubjectKey`] the
//! host minted and a [`PartAddress`] carrying the mesh revision that expires
//! it. A pick is a receipt against one encoded frame — the poses, camera,
//! filtered terrain and cut slab of *that* draw own the answer — so it goes
//! stale the moment anything visual changes, and a host must revalidate it
//! before acting on the identity inside.
//!
//! This identity is deliberately outside any world, its serialization and its
//! trace: nothing a query returns can reach a simulation.

use std::sync::atomic::{AtomicU64, Ordering};

use isometer_core::PartId;
use isometer_core::ground::BRICK;
use isometer_lens::{BrickRayHit, TraceCamera};

use crate::anchors::GlyphAnchor;
use crate::bodies::{PartAddress, SceneBody, SceneVolumes, SubjectKey};
use crate::camera::SlabCamera;
use crate::scene::Scene;

/// Receipts also expire across `Scene` replacement, so the counter is global
/// rather than per-scene.
static NEXT_QUERY_FRAME: AtomicU64 = AtomicU64::new(1);

/// A surface hit in one successfully encoded frame. A host must validate this
/// receipt before using its address after a redraw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyPick {
    pub address: PartAddress,
    pub frame: u64,
    pub distance: f32,
    pub point: [f32; 3],
    /// Distinct body/part identities had exactly the same query distance.
    /// The renderer's stable query tie rule is not a GPU colour-owner claim.
    pub tied: bool,
}

/// A terrain surface hit in one successfully encoded frame, addressed the way
/// the tracer addresses terrain: a world point, the cell behind it, and the
/// material that cell holds.
///
/// Unlike a [`BodyPick`] this carries no frame stamp and no revision-bearing
/// address, because there is no minted identity here to expire — the numbers
/// describe where the ground was when the frame was drawn. A host that needs
/// to know whether that frame is still the current one reads
/// [`Scene::query_generation`]. Turning the point into a column, a row and an
/// elevation is the host's own iso math; nothing in this crate knows a tile.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainHit {
    /// Where the ray met the surface, on the cell's boundary face.
    pub point: [f32; 3],
    /// The brick the hit cell belongs to, keyed as [`crate::GroundTerrain`]
    /// keys them: the cell floor-divided by the brick edge.
    pub brick: [i16; 3],
    /// The integer cell the ray entered: the solid one behind `point`, which
    /// is `point` stepped a fraction back along `normal` and floored. Carried
    /// so a host maps a hit to a tile without re-deriving it from the float,
    /// where a boundary point could round to either side.
    pub voxel: [i32; 3],
    /// The face the ray came through: axis-aligned, outward, unit.
    pub normal: [f32; 3],
    pub distance: f32,
    /// The material byte the tracer shaded that cell with.
    pub material: u8,
}

impl TerrainHit {
    fn of(hit: BrickRayHit) -> Self {
        Self {
            point: hit.point,
            brick: hit.voxel.map(|v| v.div_euclid(BRICK) as i16),
            voxel: hit.voxel,
            normal: hit.normal,
            distance: hit.distance,
            material: hit.material,
        }
    }
}

/// What one pick found nearest under a pixel: a body's part, or the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pick {
    Body(BodyPick),
    Terrain(TerrainHit),
}

#[derive(Clone, Debug, PartialEq)]
pub enum BodyPickError {
    /// No complete voxel frame is available after a configuration change,
    /// resize, failed render, or before the first render.
    NotReady,
    InvalidCoordinates,
    /// A visible host fallback has no exact part-surface query. Refusing
    /// prevents picking a mesh through an unqueried stand-in occluder.
    CapsuleFallback,
    Body(isometer_render::live_body::BodyQueryError),
    Terrain(isometer_lens::BrickRayError),
}

/// What one completed encode left behind for the queries to read.
#[derive(Clone, Copy)]
pub(crate) struct PresentedFrame {
    pub camera: SlabCamera,
    pub generation: u64,
    /// Whether the terrain join ran. A pick against a frame without terrain
    /// must not consult the brick map.
    pub terrain: bool,
}

impl Scene {
    /// Drops the query receipt. A host calls this whenever it changes
    /// something visual that its own state, rather than a frame, owns.
    pub fn invalidate_query(&mut self) {
        self.presented = None;
    }

    /// The camera the frame the queries answer against was drawn with, so a
    /// caller can name the pixel a world point landed on.
    pub fn presented_camera(&self) -> Option<SlabCamera> {
        self.presented.map(|frame| frame.camera)
    }

    /// The generation of the frame the queries currently answer against, or
    /// `None` when there is no complete frame.
    pub fn query_generation(&self) -> Option<u64> {
        self.presented.map(|frame| frame.generation)
    }

    /// Stamps a completed encode with a fresh generation.
    pub(crate) fn complete_query_frame(&mut self, camera: SlabCamera, terrain: bool) {
        let Ok(generation) =
            NEXT_QUERY_FRAME.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
        else {
            self.presented = None;
            return;
        };
        self.presented = Some(PresentedFrame {
            camera,
            generation,
            terrain,
        });
    }

    /// Sets host-owned inspection emphasis for the next body draw.
    pub fn set_body_focus(&mut self, subject: Option<SubjectKey>, selected: Option<PartAddress>) {
        if self.bodies_mut().set_focus(subject, selected) {
            self.invalidate_query();
        }
    }

    /// Exact transformed quad bounds for host framing, before viewport and
    /// cutaway clipping. Uses the same configured pose as the next body draw.
    pub fn presentation_bounds(
        &mut self,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
    ) -> Result<Option<([f32; 3], [f32; 3])>, String> {
        self.bodies_mut().presentation_bounds(body, volumes)
    }

    /// World bounds of one drawn part, posed exactly as the last prepare
    /// posed it. The oracle a severance or fractional-pose test reads.
    pub fn part_bounds(&self, subject: SubjectKey, part: PartId) -> Option<([f32; 3], [f32; 3])> {
        self.bodies().part_bounds(subject, part)
    }

    /// Largest meshed face per living part, ordered by `PartId` and capped at
    /// [`crate::MAX_GLYPH_ANCHORS`]. An explicit selection must match this
    /// body's current mesh revision.
    pub fn glyph_anchors(
        &mut self,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
        selected: Option<PartAddress>,
    ) -> Result<Vec<GlyphAnchor>, String> {
        self.bodies_mut().glyph_anchors(body, volumes, selected)
    }

    /// The centre of a texture pixel in normalized clip coordinates, or
    /// `InvalidCoordinates` off the frame. Coordinates start at top-left;
    /// host window/CSS conversion belongs outside this crate.
    fn ndc_of_pixel(&self, pixel: [u32; 2]) -> Result<[f32; 2], BodyPickError> {
        if pixel[0] >= self.width || pixel[1] >= self.height {
            return Err(BodyPickError::InvalidCoordinates);
        }
        Ok([
            2.0 * (pixel[0] as f32 + 0.5) / self.width as f32 - 1.0,
            1.0 - 2.0 * (pixel[1] as f32 + 0.5) / self.height as f32,
        ])
    }

    /// The presented frame and the one world ray an NDC point casts through
    /// it. Bodies and terrain are both asked with this ray, so their two
    /// answers are comparable by construction and belong to the frame the
    /// host is actually looking at.
    fn pick_ray(
        &self,
        ndc: [f32; 2],
    ) -> Result<(PresentedFrame, TraceCamera, [f32; 3], [f32; 3]), BodyPickError> {
        if ndc
            .iter()
            .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        {
            return Err(BodyPickError::InvalidCoordinates);
        }
        let frame = self.presented.ok_or(BodyPickError::NotReady)?;
        if self.bodies().stats.fallback_bodies != 0 {
            return Err(BodyPickError::CapsuleFallback);
        }
        let camera = frame.camera.trace().ok_or(BodyPickError::NotReady)?;
        let (origin, direction) = camera
            .ray_at(ndc)
            .ok_or(BodyPickError::InvalidCoordinates)?;
        Ok((frame, camera, origin, direction))
    }

    /// The terrain the presented frame drew, under this ray. A frame that
    /// drew none has none to hit, whatever map the scene still holds.
    fn terrain_under(
        &self,
        frame: PresentedFrame,
        camera: TraceCamera,
        origin: [f32; 3],
        direction: [f32; 3],
    ) -> Result<Option<BrickRayHit>, BodyPickError> {
        if !frame.terrain {
            return Ok(None);
        }
        self.terrain_ray(origin, direction, camera.far())
            .map_err(BodyPickError::Terrain)
    }

    /// Queries the centre of a texture pixel; coordinates start at top-left.
    /// Bodies only, exactly as [`Scene::pick_ndc`]; see [`Scene::pick_at_pixel`]
    /// for the pick that can also answer with the ground.
    pub fn pick_pixel(&self, pixel: [u32; 2]) -> Result<Option<BodyPick>, BodyPickError> {
        let ndc = self.ndc_of_pixel(pixel)?;
        self.pick_ndc(ndc)
    }

    /// [`Scene::pick`] at the centre of a texture pixel; coordinates start at
    /// top-left.
    pub fn pick_at_pixel(&self, pixel: [u32; 2]) -> Result<Option<Pick>, BodyPickError> {
        let ndc = self.ndc_of_pixel(pixel)?;
        self.pick(ndc)
    }

    /// Queries a completed frame at normalized clip coordinates for whatever
    /// is nearest under them: a body's part, or the ground.
    ///
    /// This supersedes [`Scene::pick_ndc`] and [`Scene::pick_pixel`] for a
    /// host that wants tiles as well as bodies. Those two stay exactly as
    /// they were — bodies only, `None` when the ground is nearer — because
    /// their callers ask the narrower question and their receipts record the
    /// narrower answer.
    ///
    /// The ray, the camera and the brick map are the presented frame's, so
    /// the answer is the one that pixel actually shows: `NotReady` before any
    /// frame, and never terrain from a frame that drew none.
    pub fn pick(&self, ndc: [f32; 2]) -> Result<Option<Pick>, BodyPickError> {
        let (frame, camera, origin, direction) = self.pick_ray(ndc)?;
        let body = self
            .bodies()
            .pick(origin, direction, camera.far(), frame.camera.clip())
            .map_err(BodyPickError::Body)?;
        let terrain = self.terrain_under(frame, camera, origin, direction)?;
        // Terrain draws after bodies with LessEqual depth testing, so a tie
        // belongs to the ground — exactly as `pick_ndc` scores it.
        let body = match body {
            Some((_, hit)) if terrain.is_some_and(|ground| ground.distance <= hit.distance) => None,
            other => other,
        };
        Ok(match (body, terrain) {
            (Some((address, hit)), _) => Some(Pick::Body(BodyPick {
                address,
                frame: frame.generation,
                distance: hit.distance,
                point: hit.point,
                tied: hit.tied,
            })),
            (None, Some(ground)) => Some(Pick::Terrain(TerrainHit::of(ground))),
            (None, None) => None,
        })
    }

    /// Queries a completed frame at normalized clip coordinates: x points
    /// right, y points up, and each lies in [-1, 1]. The last complete draw's
    /// poses, camera, terrain and cut slab own the answer.
    ///
    /// Bodies only: nearer ground answers `None` rather than naming itself.
    /// [`Scene::pick`] is the query that names it.
    pub fn pick_ndc(&self, ndc: [f32; 2]) -> Result<Option<BodyPick>, BodyPickError> {
        let (frame, camera, origin, direction) = self.pick_ray(ndc)?;
        let Some((address, hit)) = self
            .bodies()
            .pick(origin, direction, camera.far(), frame.camera.clip())
            .map_err(BodyPickError::Body)?
        else {
            return Ok(None);
        };
        let terrain = self.terrain_under(frame, camera, origin, direction)?;
        // Terrain draws after bodies with LessEqual depth testing.
        if terrain.is_some_and(|terrain| terrain.distance <= hit.distance) {
            return Ok(None);
        }
        Ok(Some(BodyPick {
            address,
            frame: frame.generation,
            distance: hit.distance,
            point: hit.point,
            tied: hit.tied,
        }))
    }

    /// A hit receipt expires on redraw or visual configuration changes; the
    /// underlying address can separately survive movement. The caller already
    /// holds the body, so nothing is looked up here.
    pub fn validate_pick(
        &mut self,
        pick: BodyPick,
        body: &SceneBody<'_>,
        volumes: SceneVolumes<'_>,
    ) -> bool {
        self.query_generation() == Some(pick.frame)
            && self
                .bodies_mut()
                .validate_address(pick.address, body, volumes)
    }

    /// Walks parts in the last successful voxel-body draw for `subject`.
    /// Host fallbacks deliberately contribute no selectable identity.
    pub fn select_part(
        &self,
        subject: SubjectKey,
        current: Option<PartAddress>,
        backwards: bool,
    ) -> Option<PartAddress> {
        self.presented?;
        self.bodies().select_part(subject, current, backwards)
    }
}

#[cfg(test)]
mod tests;
