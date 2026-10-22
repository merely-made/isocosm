// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Disposable tissue material projection for live body instances.

use std::collections::BTreeMap;

use isometer_core::PartId;

use super::LiveBody;

/// How many tissue channels one part's appearance carries: four in
/// `expression`, one in `expression_tail`.
///
/// The shader's `tissue_mark` reads them positionally, so the order is a wire
/// contract between a producer and this crate, not something either side may
/// reorder alone.
pub const TISSUE_CHANNELS: u8 = 5;

/// One material allocation summarized for one addressed part.
///
/// `material` is a neutral channel index below [`TISSUE_CHANNELS`], in the
/// same plain-`u8` grammar as `isometer_mesh`'s `MATERIAL_*` voxel codes. The
/// renderer gives each channel a mark colour and nothing else: what a channel
/// *means* is the producer's vocabulary, and the producer owns the mapping.
///
/// `fraction` is that material's living mosaic cells divided by the part's
/// living capacity. It is deliberately a summary: allocation cells are a
/// graph, not render voxels. The renderer therefore shows a voxel-aligned
/// density mark for the supplied amount, and never claims to know which mesh
/// voxel owns a particular cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PartMaterial {
    pub part: PartId,
    pub material: u8,
    pub fraction: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PartAppearance {
    /// RGB organism tint, plus a restrained amber inspection accent.
    pub(super) tint: [f32; 4],
    /// Channel 0 to 3 fractions.
    pub(super) expression: [f32; 4],
    /// Channel 4's fraction, then reserved instance fields.
    pub(super) expression_tail: [f32; 4],
}

pub(super) fn valid_materials(materials: &[PartMaterial]) -> bool {
    let mut total_by_part = BTreeMap::<PartId, f32>::new();
    for material in materials {
        if material.material >= TISSUE_CHANNELS {
            return false;
        }
        if !material.fraction.is_finite() || material.fraction < 0.0 {
            return false;
        }
        let total = total_by_part.entry(material.part).or_default();
        *total += material.fraction;
        if *total > 1.0 + f32::EPSILON {
            return false;
        }
    }
    true
}

pub(super) fn part_appearance(body: LiveBody<'_>, part: PartId) -> PartAppearance {
    let mut fractions = [0.0; TISSUE_CHANNELS as usize];
    for material in body
        .materials
        .iter()
        .filter(|material| material.part == part)
    {
        // `valid_materials` checks the input at the draw boundary, an
        // out-of-range channel included. Keeping aggregation here makes this
        // helper useful for direct appearance tests without turning it into a
        // second validation authority, so the lookup drops what it rejects.
        if let Some(slot) = fractions.get_mut(usize::from(material.material)) {
            *slot += material.fraction;
        }
    }
    let tint = if body.focused {
        body.tint.map(|channel| channel * 1.08)
    } else {
        body.tint
    };
    PartAppearance {
        tint: [
            tint[0],
            tint[1],
            tint[2],
            if body.selected_part == Some(part) {
                1.0
            } else {
                0.0
            },
        ],
        expression: [fractions[0], fractions[1], fractions[2], fractions[3]],
        expression_tail: [fractions[4], 0.0, 0.0, 0.0],
    }
}

/// Shared marks for raster faces and implicit part surfaces.
pub fn tissue_colour(base: [f32; 3], phase: f32, fractions: [f32; 5]) -> [f32; 3] {
    let mut end = 0.;
    for (fraction, mark) in fractions.into_iter().zip(TISSUE_COLOURS) {
        end += fraction;
        if phase < end {
            let shade = base.into_iter().fold(0., f32::max);
            return [0, 1, 2].map(|i| base[i] * 0.28 + mark[i] * shade * 0.72);
        }
    }
    base
}
const TISSUE_COLOURS: [[f32; 3]; 5] = [
    [0.82, 0.30, 0.25],
    [0.82, 0.55, 0.20],
    [0.32, 0.72, 0.92],
    [0.34, 0.76, 0.30],
    [0.78, 0.25, 0.72],
];

/// Area-average of the same tissue marks, useful when a part is smaller
/// than the marked voxel pattern. Unallocated tissue keeps its base colour.
pub fn tissue_average(base: [f32; 3], materials: &[PartMaterial], part: PartId) -> [f32; 3] {
    let shade = base.into_iter().fold(0., f32::max);
    let mut colour = base;
    for material in materials.iter().filter(|m| m.part == part) {
        if let Some(mark) = TISSUE_COLOURS.get(material.material as usize) {
            for i in 0..3 {
                colour[i] += material.fraction * 0.72 * (mark[i] * shade - base[i]);
            }
        }
    }
    colour
}
