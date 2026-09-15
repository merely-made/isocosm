// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Voxel volumes and where they come from.
//!
//! The core carries [`VolumeRef`] content addresses and never the voxels
//! themselves, so a projection resolves them through a [`VolumeSource`]. That
//! keeps the portable body document free of the bytes a particular renderer
//! wants, and it is what lets one part's mesh be built once and reused
//! wherever that part appears.

use std::collections::BTreeMap;

use isometer_core::VolumeRef;
use serde::{Deserialize, Serialize};

use crate::voxel::Voxels;

/// A part's occupancy grid. `0` is empty; any other value is a material id
/// that a projection maps to a colour or palette entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume {
    pub size: [u32; 3],
    voxels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeError {
    /// `size` does not match the number of voxels supplied.
    SizeMismatch { expected: usize, got: usize },
}

impl Volume {
    pub fn new(size: [u32; 3], voxels: Vec<u8>) -> Result<Self, VolumeError> {
        let expected = size.iter().map(|d| *d as usize).product::<usize>();
        if voxels.len() != expected {
            return Err(VolumeError::SizeMismatch {
                expected,
                got: voxels.len(),
            });
        }
        Ok(Self { size, voxels })
    }

    /// A solid box of one material.
    pub fn solid(size: [u32; 3], material: u8) -> Self {
        let count = size.iter().map(|d| *d as usize).product::<usize>();
        Self {
            size,
            voxels: vec![material; count],
        }
    }

    pub fn empty(size: [u32; 3]) -> Self {
        Self::solid(size, 0)
    }

    /// Builds a mesh-lane volume from a bake-lane [`Voxels`] grid.
    ///
    /// # The material offset
    ///
    /// The two grids disagree about zero, and that is the whole of this
    /// function. A `Voxels` cell is `Option<u8>`: absent, or a **palette
    /// index**, and index `0` is a real colour (the demo rig paints skin with
    /// it). A `Volume` cell is a plain `u8` where `0` means empty. So palette
    /// index `i` becomes **material `i + 1`**, and an absent cell stays `0`.
    ///
    /// The material is therefore the palette index, shifted by one: a
    /// consumer that wants a token's colours maps materials to colours with
    /// [`crate::token::material_colours`], never by reading
    /// `Palette::color(material)` directly.
    ///
    /// Palette index `255` saturates onto material `255`, colliding with
    /// `254`. A palette that large has no consumer here — the board's
    /// recipes run to single digits — and losing the top entry is preferable
    /// to silently emptying it.
    ///
    /// Axes and cell order are unchanged. Both grids are **Y-up** and both
    /// index `x + y * dx + z * dx * dy`; `bake::body` states that agreement
    /// for the profile side and it holds here, so this copies straight
    /// across with no remap. (`.vox` needs one; these two do not.)
    pub fn from_voxels(voxels: &Voxels) -> Self {
        let size = [voxels.dx as u32, voxels.dy as u32, voxels.dz as u32];
        let mut volume = Self::empty(size);
        for (x, y, z, palette_index) in voxels.iter() {
            volume.set(
                x as u32,
                y as u32,
                z as u32,
                palette_index.saturating_add(1),
            );
        }
        volume
    }

    /// This volume's content address: the same dimensions and the same cells
    /// hash to the same [`VolumeRef`] on every machine.
    ///
    /// Domain-separated from the content pack's own `content_ref`, which
    /// hashes a role and slot beside the cells, so the two never collide.
    pub fn content_ref(&self) -> VolumeRef {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"isometer.mesh.volume.v1");
        for size in self.size {
            hasher.update(&size.to_le_bytes());
        }
        hasher.update(&self.voxels);
        VolumeRef(*hasher.finalize().as_bytes())
    }

    /// Consumes the volume, yielding its cells in `x + y * sx + z * sx * sy`
    /// order. Used by the interchange profile, which carries a flat array
    /// rather than this type so a foreign reader needs none of this crate.
    pub fn into_voxels(self) -> Vec<u8> {
        self.voxels
    }

    /// Number of stored cells, including empty cells. A deserialised volume
    /// must still agree with its dimensions before a resolver can index it.
    pub fn voxel_count(&self) -> usize {
        self.voxels.len()
    }

    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        (x + y * self.size[0] + z * self.size[0] * self.size[1]) as usize
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> u8 {
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return 0;
        }
        self.voxels[self.index(x, y, z)]
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, material: u8) {
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return;
        }
        let i = self.index(x, y, z);
        self.voxels[i] = material;
    }

    /// Reads by signed coordinate, treating anything outside as empty. The
    /// mesher uses this so a face on the boundary is visible.
    pub fn get_signed(&self, coord: [i64; 3]) -> u8 {
        if coord.iter().any(|c| *c < 0) {
            return 0;
        }
        self.get(coord[0] as u32, coord[1] as u32, coord[2] as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|v| *v == 0)
    }

    pub fn solid_count(&self) -> usize {
        self.voxels.iter().filter(|v| **v != 0).count()
    }
}

/// Resolves the volumes a body's parts refer to.
pub trait VolumeSource {
    fn volume(&self, reference: VolumeRef) -> Option<&Volume>;
}

/// An in-memory source. Ordered so iteration never depends on hashing.
#[derive(Clone, Debug, Default)]
pub struct VolumeMap {
    volumes: BTreeMap<[u8; 32], Volume>,
}

impl VolumeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, reference: VolumeRef, volume: Volume) {
        self.volumes.insert(reference.0, volume);
    }

    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }
}

impl VolumeSource for VolumeMap {
    fn volume(&self, reference: VolumeRef) -> Option<&Volume> {
        self.volumes.get(&reference.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_volume_is_fully_occupied() {
        let v = Volume::solid([2, 3, 4], 7);
        assert_eq!(v.solid_count(), 24);
        assert_eq!(v.get(1, 2, 3), 7);
        assert!(!v.is_empty());
    }

    #[test]
    fn out_of_bounds_reads_as_empty() {
        let v = Volume::solid([2, 2, 2], 1);
        assert_eq!(v.get(9, 0, 0), 0);
        assert_eq!(v.get_signed([-1, 0, 0]), 0);
        assert_eq!(v.get_signed([0, 0, 0]), 1);
    }

    #[test]
    fn size_and_voxels_must_agree() {
        let err = Volume::new([2, 2, 2], vec![1; 7]).unwrap_err();
        assert_eq!(
            err,
            VolumeError::SizeMismatch {
                expected: 8,
                got: 7
            }
        );
    }

    #[test]
    fn voxels_round_trip_into_a_volume_with_the_material_offset() {
        // Palette index 0 is a real colour, so it must not land on the
        // volume's empty cell. Every filled cell shifts by one; every hole
        // stays 0.
        let mut voxels = Voxels::new(3, 4, 5);
        voxels.set(0, 0, 0, 0);
        voxels.set(2, 3, 4, 7);
        voxels.set(1, 2, 3, 254);
        voxels.set(1, 0, 0, 255);

        let volume = Volume::from_voxels(&voxels);

        assert_eq!(volume.size, [3, 4, 5], "dimensions carry across unchanged");
        assert_eq!(volume.voxel_count(), 60);
        assert_eq!(volume.solid_count(), voxels.filled());

        assert_eq!(volume.get(0, 0, 0), 1, "palette index 0 is material 1");
        assert_eq!(volume.get(2, 3, 4), 8);
        assert_eq!(volume.get(1, 2, 3), 255);
        assert_eq!(
            volume.get(1, 0, 0),
            255,
            "index 255 saturates, as documented"
        );
        assert_eq!(volume.get(0, 1, 0), 0, "an absent cell stays empty");

        // Every filled cell, at its own coordinate, with the offset undone.
        for (x, y, z, palette_index) in voxels.iter() {
            assert_eq!(
                volume.get(x as u32, y as u32, z as u32),
                palette_index.saturating_add(1),
                "cell ({x}, {y}, {z}) moved"
            );
        }
    }

    /// Y is up in both grids and neither transposes the other: a cell that is
    /// tall in `Voxels` is tall in `Volume`, and the `x + y * dx + z * dx * dy`
    /// order `bake::body` states for the profile holds here too.
    #[test]
    fn the_two_grids_agree_on_axes_and_cell_order() {
        let mut voxels = Voxels::new(2, 3, 4);
        voxels.set(1, 0, 0, 0); // +x
        voxels.set(0, 2, 0, 1); // +y, the up axis
        voxels.set(0, 0, 3, 2); // +z

        let volume = Volume::from_voxels(&voxels);

        assert_eq!(volume.get(1, 0, 0), 1);
        assert_eq!(volume.get(0, 2, 0), 2, "the tall cell stays tall");
        assert_eq!(volume.get(0, 0, 3), 3, "the deep cell stays deep");

        // The flat order is the same, so a cell's linear index agrees.
        let [sx, sy, _] = volume.size;
        let cells = volume.into_voxels();
        assert_eq!(cells[(0 + 2 * sx + 0 * sx * sy) as usize], 2);
        assert_eq!(cells[(0 + 0 * sx + 3 * sx * sy) as usize], 3);
    }

    #[test]
    fn content_addresses_follow_content() {
        let a = Volume::solid([2, 2, 2], 1);
        let b = Volume::solid([2, 2, 2], 1);
        let different_cells = Volume::solid([2, 2, 2], 2);
        let different_size = Volume::solid([2, 2, 4], 1);

        assert_eq!(a.content_ref(), b.content_ref());
        assert_ne!(a.content_ref(), different_cells.content_ref());
        assert_ne!(a.content_ref(), different_size.content_ref());
    }

    #[test]
    fn map_resolves_by_reference() {
        let mut map = VolumeMap::new();
        let r = VolumeRef::from_tag(3);
        map.insert(r, Volume::solid([1, 1, 1], 2));
        assert!(map.volume(r).is_some());
        assert!(map.volume(VolumeRef::from_tag(4)).is_none());
    }
}
