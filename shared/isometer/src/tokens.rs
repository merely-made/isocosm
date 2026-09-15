// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tokens as live bodies, reached through the facade.
//!
//! A host with a voxel recipe builds an [`isometer_mesh::TokenBody`] and hands
//! this scene the document and the volumes, exactly as it would for any other
//! body. This module is the one line of glue that saves it from naming the
//! mesh crate's `VolumeMap` itself, plus the honest note about colour.
//!
//! # Colour, and what is not wired
//!
//! [`material_colours`] turns a bake [`Palette`](isometer_mesh::bake::Palette) into the table a token's
//! materials index into — entry `i + 1` is palette index `i`, the offset
//! [`Volume::from_voxels`](isometer_mesh::Volume::from_voxels) introduces so that palette index `0` is not mistaken
//! for an empty cell.
//!
//! Nothing consumes that table yet. [`isometer_render::PartMaterial`], the
//! only per-part material input the body layer takes, is a tissue-channel
//! *density*: a `PartId`, a channel below `TISSUE_CHANNELS` (five), and a
//! fraction. It carries no colour, and the live renderer colours a quad from
//! its own `material_colour` hash times the body's tint. So a token drawn live
//! today has the right silhouette in the wrong colours, and giving the
//! renderer a per-material colour table is an isometer-render change this lane
//! does not own. Recorded for that owner beside the §6 list in
//! `isometry/design_docs/2026-09-15_board_on_isometer_plan.md`.

pub use isometer_mesh::token::{
    MissingLayer, Silhouette, TokenBody, material_colours, mesh_silhouette,
};

use crate::bodies::SceneVolumes;

/// The token's volumes as a frame's volume source.
pub fn scene_volumes(token: &TokenBody) -> SceneVolumes<'_> {
    SceneVolumes::Voxels(&token.volumes)
}

#[cfg(test)]
mod tests {
    use isometer_mesh::bake::demo;
    use isometer_mesh::{LiveBodyProjector, VolumeSource};

    use super::*;
    use crate::core::SpeciesId;

    #[test]
    fn a_token_body_resolves_through_the_scenes_volume_source() {
        let (hero, _) = demo::hero();
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);

        let SceneVolumes::Voxels(volumes) = scene_volumes(&token) else {
            panic!("a token carries authored voxels, not declared boxes");
        };
        assert!(volumes.volume(token.volume).is_some());
        assert_eq!(volumes.len(), 1);
    }

    #[test]
    fn a_token_body_projects_through_the_same_projector_the_body_layer_uses() {
        let (hero, _) = demo::hero();
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);

        let mut projector = LiveBodyProjector::new();
        let (mesh, _) = projector
            .project_body(&token.document, &token.volumes)
            .expect("a token body projects");

        assert_eq!(mesh.placement_count(), 1);
        assert!(mesh.drawn_quads() > 0);
    }

    /// The colour table is available and correctly offset, even though nothing
    /// downstream reads it yet. When the renderer grows a per-material table,
    /// this is the assertion that says what it must be fed.
    #[test]
    fn the_material_colour_table_is_the_palette_shifted_by_one() {
        let (_, palette) = demo::hero();
        let colours = material_colours(&palette);

        assert_eq!(colours.len(), palette.0.len() + 1);
        assert_eq!(colours[1], palette.color(0));
        assert_eq!(
            isometer_render::TISSUE_CHANNELS,
            5,
            "PartMaterial is five tissue channels, not a colour table"
        );
    }
}
