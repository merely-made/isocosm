// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Projecting a [`BodyProfile`] into [`Voxels`] + [`Palette`] for the baker.
//!
//! Before the merge this was a second crate's importer, reading the profile
//! back out of bytes into its own newtype. One crate now owns both ends, so
//! the wire type is the same type and this file is only the projection: the
//! grid the writer flattened, coloured either by material or by history.
//!
//! # Axes agree, and that was checked
//!
//! `.vox` needs an axis remap because MagicaVoxel is Z-up. The profile is
//! **Y-up** like [`Voxels`] — the wing's `Above` facing is axis 1, and yaw
//! rotates about axis 1 — and its grid uses the same `x + y * dx + z * dx * dy`
//! cell order, so this copies straight across.

use crate::bake::recipe::Palette;
use crate::profile::BodyProfile;
use crate::voxel::{Rgb, Voxels};

impl BodyProfile {
    /// The body as a volume indexed by **material**, which is how the writing
    /// game coloured it.
    pub fn voxels(&self) -> Voxels {
        self.build(|index| self.cells[index])
    }

    /// The body as a volume indexed by **part**, so a caller can colour by
    /// where each piece came from rather than what it is made of.
    ///
    /// This is the half that matters for a body grown by incorporation: the
    /// world is colour-coded by role, and a creature is colour-coded by
    /// history. Palette index is the part slot + 1, matching
    /// [`origin_palette`](Self::origin_palette).
    ///
    /// Bodies with more than 255 parts saturate, because a palette index is a
    /// `u8`; nothing that grows a limb at a time approaches that.
    pub fn voxels_by_part(&self) -> Voxels {
        self.build(|index| self.attribution[index].min(u8::MAX as u16) as u8)
    }

    /// A palette matching [`voxels_by_part`](Self::voxels_by_part): founding
    /// parts take `own`, incorporated parts take `taken`.
    ///
    /// Deliberately two colours rather than a gradient. The question a viewer
    /// asks of a grown body is "which of this was always mine", and a two-way
    /// answer reads at sprite scale where a per-species hue would not.
    pub fn origin_palette(&self, own: Rgb, taken: Rgb) -> Palette {
        let mut colors = Vec::with_capacity(self.parts.len() + 1);
        colors.push([0, 0, 0]); // slot 0 is empty and never drawn
        for part in &self.parts {
            colors.push(if part.is_incorporated() { taken } else { own });
        }
        Palette::new(colors)
    }

    /// How many of this body's parts were taken from other organisms.
    pub fn incorporated_parts(&self) -> usize {
        self.parts
            .iter()
            .filter(|part| part.is_incorporated())
            .count()
    }

    /// Reads a cell by grid coordinate. The profile's own accessors take body
    /// space; the baker walks the grid, so it addresses cells directly.
    fn build(&self, pick: impl Fn(usize) -> u8) -> Voxels {
        let [dx, dy, dz] = self.size.map(|d| (d as i32).max(1));
        let mut voxels = Voxels::new(dx, dy, dz);
        for z in 0..dz {
            for y in 0..dy {
                for x in 0..dx {
                    let index = (x + y * dx + z * dx * dy) as usize;
                    if self.cells[index] != 0 {
                        voxels.set(x, y, z, pick(index));
                    }
                }
            }
        }
        voxels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bake::{BakeParams, bake_facing};
    use crate::profile::{PROFILE_MAGIC, PROFILE_VERSION};
    use isometer_core::PartOrigin;
    use serde::Serialize;

    /// A foreign writer's side, mirrored for the test. The profile's own tests
    /// prove the writer emits no local type; this proves the *reader* accepts
    /// bytes laid out by somebody who never linked this crate.
    #[derive(Serialize)]
    struct Wire {
        species: u32,
        size: [u32; 3],
        origin: [i32; 3],
        cells: Vec<u8>,
        attribution: Vec<u16>,
        parts: Vec<PartOrigin>,
    }

    /// A 2x1x1 body: one founding cell, one taken from species 42.
    fn wire() -> Wire {
        Wire {
            species: 7,
            size: [2, 1, 1],
            origin: [-1, 0, 0],
            cells: vec![1, 2],
            attribution: vec![1, 2],
            parts: vec![
                PartOrigin {
                    from_species: None,
                    from_part: None,
                    epoch: 0,
                },
                PartOrigin {
                    from_species: Some(42),
                    from_part: Some(3),
                    epoch: 5,
                },
            ],
        }
    }

    fn framed(wire: &Wire) -> Vec<u8> {
        let mut bytes = PROFILE_MAGIC.to_vec();
        bytes.extend_from_slice(&PROFILE_VERSION.to_le_bytes());
        bytes.extend_from_slice(&postcard::to_allocvec(wire).unwrap());
        bytes
    }

    fn profile() -> BodyProfile {
        BodyProfile::from_bytes(&framed(&wire())).expect("a foreign profile reads")
    }

    #[test]
    fn a_foreign_body_profile_reads() {
        let body = profile();
        assert_eq!(body.species, 7);
        assert_eq!(body.size, [2, 1, 1]);
        assert_eq!(body.parts.len(), 2);
    }

    #[test]
    fn provenance_survives_the_crossing() {
        let body = profile();
        assert_eq!(body.incorporated_parts(), 1);
        assert_eq!(
            body.origin_at([0, 0, 0]),
            Some(PartOrigin {
                from_species: Some(42),
                from_part: Some(3),
                epoch: 5
            }),
            "body space is origin-relative: grid cell 1 sits at x = 0"
        );
        assert!(!body.origin_at([-1, 0, 0]).unwrap().is_incorporated());
    }

    #[test]
    fn a_body_bakes_to_a_sprite() {
        let body = profile();
        let sheet = bake_facing(
            &body.voxels_by_part(),
            &body.origin_palette([90, 140, 60], [200, 120, 70]),
            0,
            &BakeParams::default(),
        );
        assert!(
            sheet.w > 0 && sheet.h > 0,
            "a body profile produces a sprite"
        );
    }

    #[test]
    fn colouring_by_part_separates_taken_from_own() {
        let body = profile();
        let voxels = body.voxels_by_part();
        assert_eq!(voxels.get(0, 0, 0), Some(1));
        assert_eq!(voxels.get(1, 0, 0), Some(2));

        let palette = body.origin_palette([90, 140, 60], [200, 120, 70]);
        assert_eq!(
            palette.color(1),
            [90, 140, 60],
            "the founding part is its own"
        );
        assert_eq!(
            palette.color(2),
            [200, 120, 70],
            "the taken part reads as taken"
        );
    }

    #[test]
    fn material_colouring_is_still_available() {
        let body = profile();
        let voxels = body.voxels();
        assert_eq!(voxels.get(0, 0, 0), Some(1));
        assert_eq!(voxels.get(1, 0, 0), Some(2));
    }
}
