// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The host's door onto `wing-scene`'s glyph attachments. The geometry — the
//! largest meshed face per living part, its axes and extent — is the scene's;
//! what stays here is turning one Mesocosm organism into the body it reads.

use mesocosm_core::Organism;
use mesocosm_mesh::VolumeMap;
use wing_scene::{GlyphAnchor, SceneVolumes};

use crate::section::{BodySelection, Section};

impl Section {
    /// Largest meshed face per living part, ordered by PartId and capped at
    /// 32. An explicit selection must match this body's current mesh revision.
    /// Uses the configured pose for the next draw, before cutaway/occlusion.
    pub fn glyph_anchors(
        &mut self,
        organism: &Organism,
        volumes: &VolumeMap,
        selected: Option<BodySelection>,
    ) -> Result<Vec<GlyphAnchor>, String> {
        let body = self.host_bodies.scene_body(organism, &[], None);
        self.scene.glyph_anchors(
            &body,
            SceneVolumes::Voxels(volumes),
            selected.map(BodySelection::address),
        )
    }
}
