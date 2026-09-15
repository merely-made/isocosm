// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A token's recipe as a live body.
//!
//! The bake lane turns a voxel recipe into pixels; this turns the same recipe
//! into a one-part [`BodyDocument`] and the [`VolumeMap`] that resolves it, so
//! the mesh lane can draw the token live under a scene camera instead of
//! blitting a sprite. One recipe, two projections, and the volume is the
//! shared truth rather than two copies of it.
//!
//! Nothing here decides how a token *looks* beyond its silhouette: colours are
//! palette entries mapped onto material ids by [`material_colours`], and the
//! renderer that consumes them is the host's business.

use std::collections::BTreeSet;

use isometer_core::{BodyDocument, SpeciesId, VolumeRef};

use crate::bake::sheet::{build_stamp, model_centre, project_voxel, sheet_frame};
use crate::bake::{Appearance, BakeParams, Palette, compose};
use crate::greedy::PartMesh;
use crate::volume::{Volume, VolumeMap};
use crate::voxel::{Rgb, Voxels};

/// A named layer an [`Appearance`] asks for that the host library could not
/// resolve. Loud rather than silently drawing a token without its hat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingLayer(pub String);

/// One token, ready for the scene: a single-part body and the volume it
/// addresses.
///
/// The part count is deliberately one. A token is rigid; the body graph's
/// attachment machinery is for creatures that grow limbs, and paying for it
/// per token would buy nothing. A later rig that wants moving parts attaches
/// to this root rather than replacing it.
#[derive(Clone, Debug)]
pub struct TokenBody {
    pub document: BodyDocument,
    pub volumes: VolumeMap,
    /// The root part's volume address, derived from the volume's own content.
    pub volume: VolumeRef,
}

impl TokenBody {
    /// Builds a token from a ready voxel grid.
    ///
    /// The volume's [`VolumeRef`] is its content hash, so two tokens built
    /// from the same recipe address the same geometry and a projector meshes
    /// it once.
    ///
    /// The root part's half extent is the volume's size halved, rounded **up**
    /// on an odd axis, so a consumer that falls back to the declared box
    /// (`half_extent * 2`) never under-covers the voxels. `BodyDocument::new`
    /// puts the pivot on that half extent, which is the volume's centre, so a
    /// yaw turns the token about itself rather than about a corner.
    pub fn from_voxels(species: SpeciesId, mass_mg: u64, voxels: &Voxels) -> Self {
        Self::from_volume(species, mass_mg, Volume::from_voxels(voxels))
    }

    /// Builds a token from paper-doll layers, stacked by [`compose`].
    pub fn from_layers(species: SpeciesId, mass_mg: u64, layers: &[&Voxels]) -> Self {
        Self::from_voxels(species, mass_mg, &compose(layers))
    }

    /// Builds a token from an [`Appearance`], resolving its layer names
    /// through a host library.
    ///
    /// `Appearance::layers` is a list of *names*: the recipe stays data and
    /// the host owns the rig library, so resolution cannot live in this crate.
    /// An unresolved name is returned rather than skipped.
    pub fn from_appearance<'a>(
        species: SpeciesId,
        mass_mg: u64,
        appearance: &Appearance,
        library: impl Fn(&str) -> Option<&'a Voxels>,
    ) -> Result<Self, MissingLayer> {
        let mut layers = Vec::with_capacity(appearance.layers.len());
        for name in &appearance.layers {
            match library(name) {
                Some(layer) => layers.push(layer),
                None => return Err(MissingLayer(name.clone())),
            }
        }
        Ok(Self::from_layers(species, mass_mg, &layers))
    }

    /// The root volume's dimensions, which a silhouette needs to know the
    /// centre the bake turns about.
    pub fn size(&self) -> [u32; 3] {
        use crate::VolumeSource;
        self.volumes
            .volume(self.volume)
            .map(|volume| volume.size)
            .unwrap_or([0; 3])
    }

    fn from_volume(species: SpeciesId, mass_mg: u64, volume: Volume) -> Self {
        let reference = volume.content_ref();
        let half_extent = volume.size.map(|d| d.div_ceil(2) as i32);
        let mut volumes = VolumeMap::new();
        volumes.insert(reference, volume);
        Self {
            document: BodyDocument::new(species, reference, mass_mg, half_extent),
            volumes,
            volume: reference,
        }
    }
}

/// The colour table a token's materials index into.
///
/// Entry `0` is the empty material and is never drawn; entry `i + 1` is
/// palette index `i`. That offset is [`Volume::from_voxels`]'s, and this is
/// the only correct way to undo it: `palette.color(material)` is off by one
/// and would draw a token in its neighbour's colours.
///
/// **What this is not.** `isometer_render::PartMaterial` cannot carry these:
/// it is a per-part tissue-channel *density* (a `PartId`, a channel below
/// `TISSUE_CHANNELS` = 5, and a fraction), and the live renderer colours a
/// quad with its own `material_colour` hash times the body's tint. A
/// per-material colour table is not an input it takes yet. So this hands back
/// the table; wiring it through is an isometer-render change, outside this
/// lane.
pub fn material_colours(palette: &Palette) -> Vec<Rgb> {
    let mut colours = Vec::with_capacity(palette.0.len() + 1);
    colours.push([0, 0, 0]); // material 0 is empty and never drawn
    colours.extend(palette.0.iter().copied());
    colours
}

/// A coverage mask under the bake's projection: `true` where the subject
/// covers a pixel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Silhouette {
    pub w: i32,
    pub h: i32,
    pub mask: Vec<bool>,
}

impl Silhouette {
    pub fn covered(&self) -> usize {
        self.mask.iter().filter(|c| **c).count()
    }

    /// Pixels where two masks disagree, or `None` if they are not even the
    /// same size — which is a failure, not a near miss.
    pub fn disagreements(&self, other: &Silhouette) -> Option<usize> {
        (self.w == other.w && self.h == other.h).then(|| {
            self.mask
                .iter()
                .zip(&other.mask)
                .filter(|(a, b)| a != b)
                .count()
        })
    }
}

/// The silhouette a meshed part covers at `facing` under the bake's own
/// projection.
///
/// Recovers the surface voxels the quads describe and splats the bake's cube
/// stamp at each one, through [`crate::bake`]'s own projection and framing
/// code rather than a second copy of it, so an agreement between this and a
/// baked sheet is an agreement about geometry and not about arithmetic.
///
/// `size` is the volume's own dimensions: the bake turns about the volume's
/// centre, and surface voxels alone cannot tell you where an empty margin
/// ended. Interior voxels are absent by construction — the mesher culls their
/// faces — and cannot change the result, since an enclosed voxel's footprint
/// lies under its neighbours'.
///
/// `None` for a mesh with no quads: an empty body has no silhouette to
/// compare, which a caller should treat as its own failure.
pub fn mesh_silhouette(
    mesh: &PartMesh,
    size: [u32; 3],
    facing: u8,
    params: &BakeParams,
) -> Option<Silhouette> {
    let mut voxels: BTreeSet<[i32; 3]> = BTreeSet::new();
    for quad in &mesh.quads {
        let [u, v] = quad.plane_axes();
        let axis = quad.axis as usize;
        for j in 0..quad.size[1] as i32 {
            for i in 0..quad.size[0] as i32 {
                // A positive face sits on the far plane of its voxel, so the
                // voxel it belongs to is one step back along the axis.
                let mut coord = quad.origin;
                coord[u] = quad.origin[u] + i;
                coord[v] = quad.origin[v] + j;
                coord[axis] = quad.origin[axis] - i32::from(quad.positive);
                voxels.insert(coord);
            }
        }
    }
    if voxels.is_empty() {
        return None;
    }

    let centre = model_centre(size[0] as i32, size[2] as i32);
    let mut points = Vec::with_capacity(voxels.len());
    let (mut minx, mut maxx, mut miny, mut maxy) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
    for coord in &voxels {
        let (sx, sy, _) = project_voxel((coord[0], coord[1], coord[2]), centre, facing, params);
        points.push((sx, sy));
        minx = minx.min(sx);
        maxx = maxx.max(sx);
        miny = miny.min(sy);
        maxy = maxy.max(sy);
    }
    let (w, h, ox, oy) = sheet_frame(minx, maxx, miny, maxy, params);

    let stamp = build_stamp(params.half_w, params.cube_h);
    let mut mask = vec![false; (w * h) as usize];
    for (sx, sy) in points {
        let px = (sx + ox).round() as i32;
        let py = (sy + oy).round() as i32;
        for &(dx, dy, _) in &stamp {
            let (x, y) = (px + dx, py + dy);
            if x >= 0 && x < w && y >= 0 && y < h {
                mask[(x + y * w) as usize] = true;
            }
        }
    }
    Some(Silhouette { w, h, mask })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bake::{bake_facing, demo, watchtower};
    use crate::live::LiveBodyProjector;

    const BOARD: BakeParams = BakeParams {
        half_w: 2,
        cube_h: 2,
        facings: 4,
        margin: 2,
    };

    #[test]
    fn a_token_body_is_one_part_addressing_its_own_content() {
        let (hero, _) = demo::hero();
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);

        assert_eq!(token.document.living().count(), 1);
        assert_eq!(token.document.root, isometer_core::PartId(0));
        assert_eq!(token.volumes.len(), 1);
        assert_eq!(token.size(), [10, 24, 8]);
        assert_eq!(
            token
                .document
                .part(token.document.root)
                .unwrap()
                .half_extent,
            [5, 12, 4],
            "the declared box covers the volume"
        );
        assert_eq!(
            token.volume,
            TokenBody::from_voxels(SpeciesId(9), 1, &hero).volume,
            "the same recipe is the same content address, whoever owns it"
        );
    }

    #[test]
    fn layers_compose_before_the_body_is_built() {
        let mut lower = Voxels::new(2, 1, 1);
        lower.set(0, 0, 0, 0);
        let mut upper = Voxels::new(2, 1, 1);
        upper.set(1, 0, 0, 3);

        let token = TokenBody::from_layers(SpeciesId(2), 10, &[&lower, &upper]);
        let volume = crate::VolumeSource::volume(&token.volumes, token.volume).unwrap();

        assert_eq!(volume.size, [2, 1, 1]);
        assert_eq!(volume.get(0, 0, 0), 1, "palette index 0 is material 1");
        assert_eq!(volume.get(1, 0, 0), 4);
    }

    #[test]
    fn an_appearance_resolves_its_layers_or_names_the_missing_one() {
        let (hero, palette) = demo::hero();
        let appearance = Appearance {
            layers: vec!["body".into()],
            palette,
            clips: Vec::new(),
        };

        let resolved = TokenBody::from_appearance(SpeciesId(4), 100, &appearance, |name| {
            (name == "body").then_some(&hero)
        })
        .expect("the library has the layer");
        assert_eq!(resolved.size(), [10, 24, 8]);

        let missing = TokenBody::from_appearance(SpeciesId(4), 100, &appearance, |_| None);
        assert_eq!(missing.unwrap_err(), MissingLayer("body".into()));
    }

    #[test]
    fn palette_entries_land_at_their_material_ids() {
        let (_, palette) = demo::hero();
        let colours = material_colours(&palette);

        assert_eq!(colours.len(), palette.0.len() + 1);
        assert_eq!(colours[0], [0, 0, 0], "material 0 is empty");
        for (index, colour) in palette.0.iter().enumerate() {
            assert_eq!(&colours[index + 1], colour, "palette index {index} moved");
        }
    }

    /// The I4 done condition's middle clause: the recipe reaches the mesh
    /// lane's live projector and comes back as drawable geometry.
    #[test]
    fn a_token_body_projects_to_a_mesh_with_quads() {
        let (hero, _) = demo::hero();
        let token = TokenBody::from_voxels(SpeciesId(1), 70_000, &hero);

        let mut projector = LiveBodyProjector::new();
        let (mesh, _revision) = projector
            .project_body(&token.document, &token.volumes)
            .expect("a token body projects");

        assert_eq!(mesh.placement_count(), 1, "a token is one part");
        assert_eq!(mesh.mesh_count(), 1);
        assert!(mesh.drawn_quads() > 0, "and it has geometry to draw");
        assert_eq!(
            mesh.mesh_for(token.volume).map(PartMesh::len),
            Some(mesh.drawn_quads())
        );
    }

    /// The I4 done condition's last clause: the baked sprite and the live
    /// body are the same silhouette.
    ///
    /// **Tolerance.** The bake has no coverage tolerance of its own — its
    /// receipts are exact (`palette_swap_keeps_silhouette_changes_color`
    /// compares `alpha_mask` for equality, and the merge pins compare
    /// hashes), so the tolerance reused here is that one: zero. Both sides
    /// run the same projection over the same voxel truth, so anything else
    /// would be a real disagreement rather than raster noise.
    #[test]
    fn a_baked_sprite_and_a_live_body_agree_on_silhouette() {
        let subjects = [
            ("hero", demo::hero().0),
            ("tile", demo::tile().0),
            ("tower_beast", watchtower::tower_beast().0),
        ];
        for (name, voxels) in subjects {
            let (_, palette) = demo::hero();
            let token = TokenBody::from_voxels(SpeciesId(1), 1, &voxels);
            let mut projector = LiveBodyProjector::new();
            let (mesh, _) = projector
                .project_body(&token.document, &token.volumes)
                .expect("a token body projects");
            let part = mesh.mesh_for(token.volume).expect("the root is meshed");

            for facing in 0..4 {
                let sheet = bake_facing(&voxels, &palette, facing, &BOARD);
                let baked = Silhouette {
                    w: sheet.w,
                    h: sheet.h,
                    mask: sheet.alpha_mask(),
                };
                let live = mesh_silhouette(part, token.size(), facing, &BOARD)
                    .expect("a drawn body has a silhouette");

                assert_eq!(
                    (live.w, live.h),
                    (baked.w, baked.h),
                    "{name} facing {facing}: the two projections must frame alike"
                );
                assert!(baked.covered() > 0, "{name} facing {facing}: nothing baked");
                assert_eq!(
                    live.disagreements(&baked),
                    Some(0),
                    "{name} facing {facing}: live coverage {} vs baked {}",
                    live.covered(),
                    baked.covered()
                );
            }
        }
    }

    #[test]
    fn an_empty_mesh_has_no_silhouette() {
        assert!(mesh_silhouette(&PartMesh::default(), [1, 1, 1], 0, &BOARD).is_none());
    }
}
