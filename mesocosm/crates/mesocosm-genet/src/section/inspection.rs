// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's half of the presentation queries.
//!
//! The queries themselves — the frame receipt, the ray, the terrain
//! occlusion test, the part walk and the glyph anchor geometry — are
//! [`isometer::Scene`]'s now. What stays here is the product's own address:
//! a [`BodySelection`] keyed on `OrganismId`, which is what the bench's saved
//! spatial request writes to disk, and the world lookup an adapter does
//! before the scene sees a body.

use isometer::mesh::BodyDependencyRevision;
use isometer::{BodyPickError, PartAddress, SceneVolumes};
use mesocosm_core::{OrganismId, PartId, World};

use super::Section;
use super::bodies::{key, organism_of};

/// A part address carried by the last successful voxel-body projection.
///
/// The revision makes a selection expire when attachment geometry changes.
/// It is presentation state, never a world address or trace input.
///
/// Kept keyed on `OrganismId` rather than renamed to the scene's
/// [`PartAddress`]: `app/bench/spatial/saved.rs` writes `(organism, part,
/// revision)` into the saved spatial request, and that file's format is not
/// this extraction's to change.
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

impl BodyPick {
    fn from_scene(pick: isometer::BodyPick) -> Self {
        Self {
            selection: BodySelection::from_address(pick.address),
            frame: pick.frame,
            distance: pick.distance,
            point: pick.point,
            tied: pick.tied,
        }
    }
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
    ) -> Result<(), isometer::render::live_body::LiveBodyError> {
        if !radians.is_finite() {
            return Err(isometer::render::live_body::LiveBodyError::InvalidBody);
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
        volumes: &isometer::mesh::VolumeMap,
    ) -> Result<Option<([f32; 3], [f32; 3])>, String> {
        let body = self.host_bodies.scene_body(organism, &[], None);
        self.scene
            .presentation_bounds(&body, SceneVolumes::Voxels(volumes))
    }

    /// Queries the centre of a texture pixel; coordinates start at top-left.
    /// Host window/CSS coordinate conversion belongs outside this section.
    pub fn pick_pixel(&self, pixel: [u32; 2]) -> Result<Option<BodyPick>, BodyPickError> {
        Ok(self.scene.pick_pixel(pixel)?.map(BodyPick::from_scene))
    }

    /// Queries a completed voxel section at normalized clip coordinates:
    /// x points right, y points up, and each lies in [-1, 1]. The last complete
    /// draw's poses, camera, filtered terrain and cut slab own the answer.
    pub fn pick_ndc(&self, ndc: [f32; 2]) -> Result<Option<BodyPick>, BodyPickError> {
        Ok(self.scene.pick_ndc(ndc)?.map(BodyPick::from_scene))
    }

    /// A hit receipt expires on redraw or visual configuration changes;
    /// the underlying semantic selection can separately survive movement.
    pub fn validate_pick(
        &mut self,
        pick: BodyPick,
        world: &World,
        volumes: &isometer::mesh::VolumeMap,
    ) -> bool {
        self.scene.query_generation() == Some(pick.frame)
            && self.validate_selection(pick.selection, world, volumes)
    }

    pub(super) fn invalidate_query(&mut self) {
        self.scene.invalidate_query();
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
        self.scene
            .select_part(key(subject), current.map(BodySelection::address), backwards)
            .map(BodySelection::from_address)
    }

    /// Confirms that a selected part remains both drawable and geometrically
    /// current before the host presents it again.
    pub fn validate_selection(
        &mut self,
        selection: BodySelection,
        world: &World,
        volumes: &isometer::mesh::VolumeMap,
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
        self.scene.bodies_mut().validate_address(
            selection.address(),
            &body,
            SceneVolumes::Voxels(volumes),
        )
    }

    /// Sets host-owned inspection emphasis for the next body draw.
    pub fn set_body_focus(&mut self, subject: Option<OrganismId>, selected: Option<BodySelection>) {
        self.scene
            .set_body_focus(subject.map(key), selected.map(BodySelection::address));
    }

    /// Largest meshed face per living part, ordered by PartId and capped at
    /// 32. An explicit selection must match this body's current mesh revision.
    /// Uses the configured pose for the next draw, before cutaway/occlusion.
    ///
    /// The geometry is the scene's; what happens here is turning one Mesocosm
    /// organism into the body it reads.
    pub fn glyph_anchors(
        &mut self,
        organism: &mesocosm_core::Organism,
        volumes: &isometer::mesh::VolumeMap,
        selected: Option<BodySelection>,
    ) -> Result<Vec<isometer::GlyphAnchor>, String> {
        let body = self.host_bodies.scene_body(organism, &[], None);
        self.scene.glyph_anchors(
            &body,
            SceneVolumes::Voxels(volumes),
            selected.map(BodySelection::address),
        )
    }
}
