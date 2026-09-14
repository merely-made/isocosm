// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Presentation identity for a part that was actually drawn as a voxel body.

use mesocosm_core::{OrganismId, PartId, World};
use mesocosm_mesh::BodyDependencyRevision;
use std::sync::atomic::{AtomicU64, Ordering};
use wing_scene::{PartAddress, SceneVolumes};

use super::Section;
use super::bodies::{key, organism_of};

// Presentation receipts also expire across Section replacement. This identity
// is deliberately outside the world, its serialization and its trace.
static NEXT_QUERY_FRAME: AtomicU64 = AtomicU64::new(1);

/// A part address carried by the last successful voxel-body projection.
///
/// The revision makes a selection expire when attachment geometry changes.
/// It is presentation state, never a world address or trace input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodySelection {
    pub organism: OrganismId,
    pub part: PartId,
    pub revision: BodyDependencyRevision,
}

impl BodySelection {
    /// The same address, keyed the way the scene keys it. Every key the scene
    /// hands back was minted here, so the round trip is exact.
    pub(super) fn address(self) -> PartAddress {
        PartAddress {
            subject: key(self.organism),
            part: self.part,
            revision: self.revision,
        }
    }

    pub(super) fn from_address(address: PartAddress) -> Self {
        Self {
            organism: organism_of(address.subject),
            part: address.part,
            revision: address.revision,
        }
    }
}

/// A surface hit in one successfully encoded section frame. A host must
/// validate this receipt before using its semantic selection after a redraw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyPick {
    pub selection: BodySelection,
    pub frame: u64,
    pub distance: f32,
    pub point: [f32; 3],
    /// Distinct body/part identities had exactly the same query distance.
    /// The renderer's stable query tie rule is not a GPU colour-owner claim.
    pub tied: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BodyPickError {
    /// No complete voxel section is available after a configuration change,
    /// resize, failed render, or before the first render.
    NotReady,
    InvalidCoordinates,
    /// A visible capsule fallback has no exact part-surface query. Refusing
    /// prevents picking a mesh through an unqueried capsule occluder.
    CapsuleFallback,
    Body(mesocosm_render::live_body::BodyQueryError),
    Terrain(mesocosm_lens::BrickRayError),
}

#[derive(Clone, Copy)]
pub(super) struct PresentedFrame {
    pub view: super::view::View,
    pub generation: u64,
    pub terrain: bool,
}

impl Section {
    /// An isolated body view reuses the resident body renderer and depth target.
    /// The host restores ordinary scene rendering when its menu closes.
    pub fn set_body_preview(&mut self, isolated: bool, depth: f32) {
        let depth = if depth.is_finite() && depth > 0.0 {
            depth
        } else {
            super::SLAB_DEPTH
        };
        let bodies = self.scene.bodies_mut();
        if bodies.isolated != isolated || bodies.preview_depth != depth {
            bodies.isolated = isolated;
            bodies.preview_depth = depth;
            self.invalidate_query();
        }
    }

    /// A host-owned continuous body pose. Core attachment yaw, geometry,
    /// authoritative positions and simulation state remain unchanged.
    pub fn set_body_yaw(
        &mut self,
        subject: OrganismId,
        radians: f32,
    ) -> Result<(), mesocosm_render::live_body::LiveBodyError> {
        if !radians.is_finite() {
            return Err(mesocosm_render::live_body::LiveBodyError::InvalidBody);
        }
        if self.host_bodies.set_yaw(subject, radians) {
            self.invalidate_query();
        }
        Ok(())
    }

    pub fn body_yaw(&self, subject: OrganismId) -> f32 {
        self.host_bodies.yaw(subject)
    }

    /// Exact transformed quad bounds for host framing, before viewport and
    /// cutaway clipping. Uses the same configured pose as the next body draw.
    pub fn presentation_bounds(
        &mut self,
        organism: &mesocosm_core::Organism,
        volumes: &mesocosm_mesh::VolumeMap,
    ) -> Result<Option<([f32; 3], [f32; 3])>, String> {
        let body = self.host_bodies.scene_body(organism, &[], None);
        self.scene
            .bodies_mut()
            .presentation_bounds(&body, SceneVolumes::Voxels(volumes))
    }

    /// Queries the centre of a texture pixel; coordinates start at top-left.
    /// Host window/CSS coordinate conversion belongs outside this section.
    pub fn pick_pixel(&self, pixel: [u32; 2]) -> Result<Option<BodyPick>, BodyPickError> {
        if pixel[0] >= self.width || pixel[1] >= self.height {
            return Err(BodyPickError::InvalidCoordinates);
        }
        self.pick_ndc([
            2.0 * (pixel[0] as f32 + 0.5) / self.width as f32 - 1.0,
            1.0 - 2.0 * (pixel[1] as f32 + 0.5) / self.height as f32,
        ])
    }

    /// Queries a completed voxel section at normalized clip coordinates:
    /// x points right, y points up, and each lies in [-1, 1]. The last complete
    /// draw's poses, camera, filtered terrain and cut slab own the answer.
    pub fn pick_ndc(&self, ndc: [f32; 2]) -> Result<Option<BodyPick>, BodyPickError> {
        if ndc
            .iter()
            .any(|v| !v.is_finite() || !(-1.0..=1.0).contains(v))
        {
            return Err(BodyPickError::InvalidCoordinates);
        }
        let frame = self.presented.ok_or(BodyPickError::NotReady)?;
        if self.scene.bodies().stats.fallback_bodies != 0 {
            return Err(BodyPickError::CapsuleFallback);
        }
        let camera = frame.view.trace().ok_or(BodyPickError::NotReady)?;
        let (origin, direction) = camera
            .ray_at(ndc)
            .ok_or(BodyPickError::InvalidCoordinates)?;
        let Some((selection, hit)) = self
            .scene
            .bodies()
            .pick(origin, direction, camera.far(), frame.view.clip())
            .map_err(BodyPickError::Body)?
        else {
            return Ok(None);
        };
        if frame.terrain {
            let terrain = self
                .scene
                .terrain_ray(origin, direction, camera.far())
                .map_err(BodyPickError::Terrain)?;
            // Terrain draws after bodies with LessEqual depth testing.
            if terrain.is_some_and(|terrain| terrain.distance <= hit.distance) {
                return Ok(None);
            }
        }
        Ok(Some(BodyPick {
            selection: BodySelection::from_address(selection),
            frame: frame.generation,
            distance: hit.distance,
            point: hit.point,
            tied: hit.tied,
        }))
    }

    /// A hit receipt expires on redraw or visual configuration changes;
    /// the underlying semantic selection can separately survive movement.
    pub fn validate_pick(
        &mut self,
        pick: BodyPick,
        world: &World,
        volumes: &mesocosm_mesh::VolumeMap,
    ) -> bool {
        self.presented
            .is_some_and(|frame| frame.generation == pick.frame)
            && self.validate_selection(pick.selection, world, volumes)
    }

    pub(super) fn invalidate_query(&mut self) {
        self.presented = None;
    }

    pub(super) fn complete_query_frame(&mut self, view: super::view::View, terrain: bool) {
        let Ok(generation) =
            NEXT_QUERY_FRAME.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
        else {
            self.presented = None;
            return;
        };
        self.presented = Some(PresentedFrame {
            view,
            generation,
            terrain,
        });
    }

    /// Walks parts in the last successful voxel-body draw for `subject`.
    /// Capsule fallbacks deliberately contribute no selectable identity.
    pub fn select_part(
        &self,
        subject: OrganismId,
        current: Option<BodySelection>,
        backwards: bool,
    ) -> Option<BodySelection> {
        if self.body_mode != super::BodyMode::Voxels {
            return None;
        }
        self.presented?;
        self.scene
            .bodies()
            .select_part(key(subject), current.map(BodySelection::address), backwards)
            .map(BodySelection::from_address)
    }

    /// Confirms that a selected part remains both drawable and geometrically
    /// current before the host presents it again.
    pub fn validate_selection(
        &mut self,
        selection: BodySelection,
        world: &World,
        volumes: &mesocosm_mesh::VolumeMap,
    ) -> bool {
        if self.body_mode != super::BodyMode::Voxels {
            return false;
        }
        let Some(organism) = world
            .organisms
            .iter()
            .find(|organism| organism.id == selection.organism)
        else {
            return false;
        };
        let body = self.host_bodies.scene_body(organism, &[], None);
        self.scene
            .bodies_mut()
            .validate_address(selection.address(), &body, SceneVolumes::Voxels(volumes))
    }

    /// Sets host-owned inspection emphasis for the next body draw.
    pub fn set_body_focus(&mut self, subject: Option<OrganismId>, selected: Option<BodySelection>) {
        if self
            .scene
            .bodies_mut()
            .set_focus(subject.map(key), selected.map(BodySelection::address))
        {
            self.invalidate_query();
        }
    }
}
