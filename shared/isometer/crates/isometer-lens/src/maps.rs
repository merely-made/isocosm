// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The march's input images: the world painted as a heightmap and a colormap.
//!
//! The march consumes these two and nothing else, so a biosphere is whatever
//! gets painted into them. Painting them is the caller's job; Mesocosm's probe
//! synthesiser lives in `mesocosm-genet` because it partitions with `Places`
//! and seeds with `Rng`, which are the game's, not the lens's.

/// The two images the march eats, plus the palette the grade may use.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BiomeMaps {
    pub side: u32,
    /// R8 heights, one byte per texel.
    pub height: Vec<u8>,
    /// RGBA colours.
    pub color: Vec<u8>,
    /// The palette implied by the biomes: base tints at four shades each,
    /// plus fog and sky entries. What a retro grade quantises against.
    pub palette: Vec<[f32; 3]>,
}

/// A painted world for the crate's own receipts: quadrant tints over a smooth
/// ridge, which is all the renderer tests need of a map. Deterministic in the
/// seed, and free of any dependency on the simulation.
#[cfg(test)]
pub(crate) fn probe(seed: u64, side: u32) -> BiomeMaps {
    let tints: Vec<[f32; 3]> = (0..4)
        .map(|quadrant| {
            let hue = ((seed.wrapping_mul(7919) + quadrant) % 360) as f32;
            [
                (hue / 360.0) * 0.6 + 0.2,
                ((hue + 120.0) % 360.0) / 360.0 * 0.6 + 0.2,
                ((hue + 240.0) % 360.0) / 360.0 * 0.6 + 0.2,
            ]
        })
        .collect();

    let mut height = vec![0u8; (side * side) as usize];
    let mut color = vec![0u8; (side * side * 4) as usize];
    let half = side.max(2) / 2;
    for row in 0..side {
        for col in 0..side {
            let i = (row * side + col) as usize;
            let (u, v) = (col as f32 / side as f32, row as f32 / side as f32);
            // A single broad ridge, so the march has relief to hit and the
            // grade has a range to quantise.
            let h = (0.25 + 0.5 * (u * std::f32::consts::PI).sin() * (v * std::f32::consts::PI).sin())
                .clamp(0.0, 1.0);
            height[i] = (h * 255.0) as u8;
            let tint = tints[((row / half) * 2 + (col / half)) as usize % 4];
            let shade = 0.55 + h * 0.6;
            for channel in 0..3 {
                color[i * 4 + channel] = ((tint[channel] * shade).min(1.0) * 255.0) as u8;
            }
            color[i * 4 + 3] = 255;
        }
    }

    let mut palette = Vec::new();
    for tint in &tints {
        for shade in [0.5, 0.72, 0.95, 1.2] {
            palette.push([
                (tint[0] * shade).min(1.0),
                (tint[1] * shade).min(1.0),
                (tint[2] * shade).min(1.0),
            ]);
        }
    }
    palette.push([0.66, 0.66, 0.72]); // fog
    palette.push([0.65, 0.72, 0.80]); // sky

    BiomeMaps {
        side,
        height,
        color,
        palette,
    }
}
