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
//! **Second declared limit.** Geometry is cached by `VolumeRef` bytes, so two
//! bodies that reuse one tag for parts of *different* declared extents would
//! share the first box. That is counted by [`DeclaredExtentVolumes::conflicts`]
//! rather than drawn wrong silently.
//!
//! Lifted from `eponym-client/src/producer/bodies.rs` as part of the
//! isometer extraction; the producer keeps its own `BODY_MATERIAL` choice,
//! because the palette is presentation.

use std::collections::{BTreeMap, btree_map::Entry};

use isometer_core::{BodyDocument, VolumeRef};
use isometer_mesh::{Volume, VolumeMap, VolumeSource};

/// One solid box per volume reference a set of documents addresses.
#[derive(Clone, Debug, Default)]
pub struct DeclaredExtentVolumes {
    volumes: VolumeMap,
    conflicts: usize,
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
        let mut extents: BTreeMap<VolumeRef, [u32; 3]> = BTreeMap::new();
        let mut conflicts = 0;
        for document in docs {
            for part in document.living() {
                let extent = part.half_extent.map(|half| (half.max(0) as u32) * 2);
                if extent.iter().any(|d| *d == 0) {
                    continue;
                }
                match extents.entry(part.volume) {
                    Entry::Vacant(slot) => {
                        slot.insert(extent);
                    },
                    Entry::Occupied(slot) => {
                        conflicts += usize::from(*slot.get() != extent);
                    },
                }
            }
        }
        let mut volumes = VolumeMap::new();
        for (reference, extent) in extents {
            volumes.insert(reference, Volume::solid(extent, material));
        }
        Self { volumes, conflicts }
    }

    /// One tag declared at two extents: counted, never drawn wrong silently.
    pub fn conflicts(&self) -> usize {
        self.conflicts
    }

    /// Distinct boxes built, one per addressed reference.
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
}

#[cfg(test)]
mod tests {
    use isometer_core::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};

    use super::*;

    fn document(root: VolumeRef, half_extent: [i32; 3]) -> BodyDocument {
        BodyDocument::new(SpeciesId(3), root, 100, half_extent)
    }

    #[test]
    fn one_solid_per_addressed_tag_at_twice_its_declared_half_extent() {
        let mut body = document(VolumeRef::from_tag(1), [1, 2, 3]);
        body.attach(
            VolumeRef::from_tag(2),
            20,
            [2, 2, 2],
            Attachment {
                parent: body.root,
                offset: [5, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();

        let volumes = DeclaredExtentVolumes::from_documents([&body], 245);

        assert_eq!(volumes.len(), 2);
        assert_eq!(volumes.conflicts(), 0);
        let root = volumes.volume(VolumeRef::from_tag(1)).unwrap();
        assert_eq!(root.size, [2, 4, 6]);
        assert_eq!(root.get(0, 0, 0), 245);
        assert_eq!(
            volumes.volume(VolumeRef::from_tag(2)).unwrap().size,
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
                20,
                [1, 0, 1],
                Attachment {
                    parent: second.root,
                    offset: [4, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();

        let volumes = DeclaredExtentVolumes::from_documents([&first, &second], 1);

        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes.conflicts(), 0);
        assert!(volumes.volume(VolumeRef::from_tag(7)).is_none());
    }

    /// One tag declared at two extents is counted once per disagreement, and
    /// the first declaration is the one drawn.
    #[test]
    fn one_tag_at_two_extents_is_counted_rather_than_drawn_wrong() {
        let first = document(VolumeRef::from_tag(4), [1, 1, 1]);
        let second = document(VolumeRef::from_tag(4), [3, 3, 3]);

        let volumes = DeclaredExtentVolumes::from_documents([&first, &second], 1);

        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes.conflicts(), 1);
        assert_eq!(
            volumes.volume(VolumeRef::from_tag(4)).unwrap().size,
            [2, 2, 2]
        );
        assert!(DeclaredExtentVolumes::from_documents(std::iter::empty(), 1).is_empty());
    }
}
