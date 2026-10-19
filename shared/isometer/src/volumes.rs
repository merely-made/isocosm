// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The declared-extent volume fallback.
//!
//! **Declared limit.** An anatomy may address volumes by [`VolumeRef`] and
//! carry no voxel data at all — every Eponym body today, and any product
//! whose bodies are boxes before they are sculpture. Each intact part is then
//! drawn as one [`Volume::solid`] of the part's own declared extent
//! (`half_extent * 2`) in a single neutral material. A part's shape is its
//! declared box and nothing finer, and two parts of one body are told apart
//! geometrically, never by colour.
//!
//! Boxes are addressed by their actual content. An unresolved tag may occur
//! at several extents, so resolving a part reads its dimensions rather than
//! treating the tag as a shared shape. The native document is never rewritten.
//!
//! Lifted from `eponym-client/src/producer/bodies.rs` as part of the
//! isometer extraction; the producer keeps its own `BODY_MATERIAL` choice,
//! because the palette is presentation.

use std::collections::BTreeMap;

use isometer_core::{BodyDocument, VolumeRef};
use isometer_mesh::{Volume, VolumeMap, VolumeSource};

/// One solid box per distinct declared extent, shared across documents.
#[derive(Clone, Debug, Default)]
pub struct DeclaredExtentVolumes {
    volumes: VolumeMap,
    extents: BTreeMap<[u32; 3], VolumeRef>,
}

impl DeclaredExtentVolumes {
    /// Builds the fallback for every living part of every document supplied.
    ///
    /// A part whose declared extent is zero on any axis is skipped: it has no
    /// box to draw, and a zero-sized [`Volume`] would only mesh to nothing.
    pub fn from_documents<'a>(
        docs: impl IntoIterator<Item = &'a BodyDocument>,
        material: u8,
    ) -> Self {
        let mut extents = BTreeMap::new();
        let mut volumes = VolumeMap::new();
        for document in docs {
            for part in document.living() {
                let extent = part.half_extent.map(|half| (half.max(0) as u32) * 2);
                if extent.iter().any(|d| *d == 0) {
                    continue;
                }
                extents.entry(extent).or_insert_with(|| {
                    let volume = Volume::solid(extent, material);
                    let reference = volume.content_ref();
                    volumes.insert(reference, volume);
                    reference
                });
            }
        }
        Self { volumes, extents }
    }

    /// Distinct boxes built, one per extent and material.
    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }

    /// The resolved map, for a consumer that wants the source by value.
    pub fn volumes(&self) -> &VolumeMap {
        &self.volumes
    }
}

impl VolumeSource for DeclaredExtentVolumes {
    fn volume(&self, reference: VolumeRef) -> Option<&Volume> {
        self.volumes.volume(reference)
    }

    fn resolve_part(&self, part: &isometer_core::Part) -> Option<(VolumeRef, &Volume)> {
        let extent = part.half_extent.map(|half| (half.max(0) as u32) * 2);
        let reference = *self.extents.get(&extent)?;
        self.volume(reference).map(|volume| (reference, volume))
    }
}

#[cfg(test)]
mod tests {
    use isometer_core::{Attachment, BodyDocument, VolumeRef, Yaw};
    use isometer_mesh::{LiveBodyProjector, flatten, mesh_body};

    use super::*;

    fn document(root: VolumeRef, half_extent: [i32; 3]) -> BodyDocument {
        BodyDocument::new(root, half_extent)
    }

    #[test]
    fn unresolved_parts_keep_their_own_dimensions_in_both_mesh_paths() {
        let tag = VolumeRef::from_tag(0);
        let mut body = document(tag, [2, 2, 2]);
        body.attach(
            tag,
            [4, 1, 1],
            Attachment {
                parent: body.root,
                offset: [0, 6, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
        let original = body.clone();
        let volumes = DeclaredExtentVolumes::from_documents([&body], 1);
        let mut projector = LiveBodyProjector::new();
        let (live, _) = projector.project_body(&body, &volumes).unwrap();
        let once = mesh_body(&body, &volumes).unwrap();
        assert_eq!(flatten(&body, &volumes).unwrap().volume.solid_count(), 96);
        for mesh in [&live, &once] {
            for placed in &mesh.placements {
                let quads = &mesh.mesh_for(placed.volume).unwrap().quads;
                let extent = [0, 1, 2].map(|axis| {
                    quads
                        .iter()
                        .flat_map(|q| q.corners())
                        .map(|p| p[axis])
                        .max()
                        .unwrap()
                });
                assert_eq!(
                    extent,
                    body.part(placed.part).unwrap().half_extent.map(|h| h * 2)
                );
            }
        }
        assert_eq!(
            body, original,
            "projection must not rewrite the saved anatomy"
        );
    }

    #[test]
    fn changed_extents_do_not_reuse_an_old_cached_box() {
        let first = document(VolumeRef::from_tag(0), [1, 1, 1]);
        let second = document(VolumeRef::from_tag(0), [2, 1, 1]);
        let mut projector = LiveBodyProjector::new();
        let (before, revision) = projector
            .project_body(&first, &DeclaredExtentVolumes::from_documents([&first], 1))
            .unwrap();
        let (after, changed) = projector
            .project_body(
                &second,
                &DeclaredExtentVolumes::from_documents([&second], 1),
            )
            .unwrap();
        assert_eq!(before.bounds(), Some(([-1, -1, -1], [1, 1, 1])));
        assert_eq!(after.bounds(), Some(([-2, -1, -1], [2, 1, 1])));
        assert_ne!(revision, changed);
        assert_eq!(projector.cached_mesh_count(), 2);
    }

    #[test]
    fn different_tags_with_identical_boxes_share_content() {
        let first = document(VolumeRef::from_tag(1), [1, 2, 3]);
        let second = document(VolumeRef::from_tag(2), [1, 2, 3]);
        let volumes = DeclaredExtentVolumes::from_documents([&first, &second], 1);
        assert_eq!(volumes.len(), 1);
        assert_eq!(
            volumes.resolve_part(&first.parts[0]),
            volumes.resolve_part(&second.parts[0])
        );
    }

    #[test]
    fn one_solid_per_extent_at_twice_its_declared_half_extent() {
        let mut body = document(VolumeRef::from_tag(1), [1, 2, 3]);
        body.attach(
            VolumeRef::from_tag(2),
            [2, 2, 2],
            Attachment {
                parent: body.root,
                offset: [5, 0, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();

        let volumes = DeclaredExtentVolumes::from_documents([&body], 245);

        assert_eq!(volumes.len(), 2);
        let (_, root) = volumes.resolve_part(&body.parts[0]).unwrap();
        assert_eq!(root.size, [2, 4, 6]);
        assert_eq!(root.get(0, 0, 0), 245);
        assert_eq!(
            volumes.resolve_part(&body.parts[1]).unwrap().1.size,
            [4, 4, 4]
        );
        assert!(volumes.volume(VolumeRef::from_tag(9)).is_none());
    }

    /// The same reference across two bodies is one box, and a part with no
    /// declared extent is skipped rather than meshed to nothing.
    #[test]
    fn a_shared_tag_is_one_box_and_a_flat_part_is_skipped() {
        let first = document(VolumeRef::from_tag(1), [1, 1, 1]);
        let mut second = document(VolumeRef::from_tag(1), [1, 1, 1]);
        second
            .attach(
                VolumeRef::from_tag(7),
                [1, 0, 1],
                Attachment {
                    parent: second.root,
                    offset: [4, 0, 0],
                    yaw: Yaw::Zero,
                },
                None,
            )
            .unwrap();

        let volumes = DeclaredExtentVolumes::from_documents([&first, &second], 1);

        assert_eq!(volumes.len(), 1);
        assert!(volumes.resolve_part(&second.parts[1]).is_none());
    }

    /// An undrawn tag shared by different bodies must not alias their geometry.
    #[test]
    fn one_tag_at_two_extents_resolves_two_boxes() {
        let first = document(VolumeRef::from_tag(4), [1, 1, 1]);
        let second = document(VolumeRef::from_tag(4), [3, 3, 3]);

        let volumes = DeclaredExtentVolumes::from_documents([&first, &second], 1);

        assert_eq!(volumes.len(), 2);
        assert_eq!(
            volumes.resolve_part(&first.parts[0]).unwrap().1.size,
            [2, 2, 2]
        );
        assert_eq!(
            volumes.resolve_part(&second.parts[0]).unwrap().1.size,
            [6, 6, 6]
        );
        assert!(DeclaredExtentVolumes::from_documents(std::iter::empty(), 1).is_empty());
    }
}
