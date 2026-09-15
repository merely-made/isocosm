// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The material palette: what a brick material is drawn in, and the exact
//! arithmetic a frame that binds no palette keeps.
//!
//! Every frame here is flat ground under an orthographic camera looking
//! straight down, which makes the probe arithmetic exact rather than
//! approximate: the camera's own right and up vectors place a named world
//! column at a named pixel, the grade is clay with fog switched off, and the
//! only thing between a palette entry and the byte read back is the top
//! face's fixed light term.

use isometer_core::ground::{Ground, Terrain};

use crate::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Grade, TerrainAppearance,
    TerrainPalette, TraceCamera,
};

/// Bands two columns wide, one material each. 0..=15 are palette entries
/// 1..=16; the last band is past the table and has to read as entry 0.
const BANDS: i32 = 17;
const EXTENT: i32 = 17;
const SURFACE: i32 = 6;
const WIDTH: u32 = 256;
const HEIGHT: u32 = 64;
/// Orthographic half-height in world units; the whole fixture plus a margin.
const HALF: f32 = 18.0;

struct Bands;

impl Terrain for Bands {
    fn sea_level(&self, _extent: i32) -> i32 {
        1
    }

    fn surface(&self, _extent: i32, _x: i32, _z: i32) -> i32 {
        SURFACE
    }
}

/// The band a column belongs to, and so the material laid through it.
fn band_of(x: i32) -> i32 {
    ((x + EXTENT) / 2).min(BANDS - 1)
}

fn material_of(band: i32) -> u8 {
    if band == BANDS - 1 {
        // Deliberately past the bound table: the fallback's own receipt.
        40
    } else {
        band as u8 + 1
    }
}

/// The first column of a band, which is what the probe aims at.
fn column_of(band: i32) -> i32 {
    band * 2 - EXTENT
}

fn banded_ground() -> Ground {
    Ground::grow_with(&Bands, EXTENT, |x, _z, _depth| material_of(band_of(x)))
}

/// A camera 8 voxels above the surface looking straight down, with a slab
/// deep enough to reach it. Level by construction, so the slab wall's advance
/// is exactly zero and a world column sits where the arithmetic below puts it.
fn overhead() -> TraceCamera {
    TraceCamera::orthographic_slab(
        [0.5, SURFACE as f32 + 8.0, 0.5],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, -1.0],
        HALF,
        1.0,
        16.0,
    )
    .expect("a level overhead slab")
}

/// The pixel a world column's centre lands on. The camera's `right` is
/// `(1, 0, 0) * HALF` and its plane stands at x = 0.5, so a ray at `ndc.x`
/// meets the ground at `0.5 + HALF * ndc.x`.
fn probe(world_x: f32) -> usize {
    let ndc = (world_x - 0.5) / HALF;
    let column = (((ndc + 1.0) * WIDTH as f32 / 2.0) - 0.5).round() as usize;
    let row = (HEIGHT / 2) as usize;
    (row * WIDTH as usize + column) * 4
}

/// The clay grade with fog switched off: no fog mix, no quantiser, no floor.
/// What the trace writes is the material's colour times the face's light.
fn unfogged() -> Grade {
    Grade {
        fog_start: 1.0,
        ..Grade::clay()
    }
}

/// The shader's own top-face light term, written once here rather than
/// guessed: `0.38 + 0.62 * dot(normal, sun)` with the normal straight up.
fn top_face_light() -> f32 {
    let sun = [0.4f32, 0.8, 0.3];
    let length = (sun[0] * sun[0] + sun[1] * sun[1] + sun[2] * sun[2]).sqrt();
    0.38 + 0.62 * (sun[1] / length)
}

fn expect_colour(pixels: &[u8], at: usize, colour: [f32; 3], what: &str) {
    let light = top_face_light();
    for channel in 0..3 {
        let expected = (colour[channel] * light * 255.0).round() as i32;
        let actual = pixels[at + channel] as i32;
        assert!(
            (expected - actual).abs() <= 2,
            "{what}: channel {channel} read {actual}, expected {expected}"
        );
    }
}

/// I1's done-condition: a 16-material fixture drawn with each material at its
/// own table colour, plus the fallback for a material the table cannot reach.
#[test]
fn a_bound_palette_draws_every_material_at_its_table_colour() {
    let ground = banded_ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(WIDTH, HEIGHT) else {
        eprintln!("no adapter; skipping terrain palette receipt");
        return;
    };
    let unknown = [0.9, 0.1, 0.9];
    let colour = |material: u8| {
        let index = material as f32;
        [
            index / 20.0,
            ((material * 3) % 20) as f32 / 20.0,
            ((material * 7) % 20) as f32 / 20.0,
        ]
    };
    let palette =
        TerrainPalette::new(std::iter::once(unknown).chain((1..=(BANDS - 1) as u8).map(colour)));
    assert_eq!(palette.len(), BANDS as usize);

    let grade = unfogged();
    let frame = tracer
        .capture(
            BrickFrameInput::for_camera(&map, BrickRevision(ground.revision()), overhead(), &grade)
                .with_terrain_palette(&palette),
        )
        .expect("palette frame");

    for band in 0..BANDS {
        let at = probe(column_of(band) as f32 + 0.5);
        let material = material_of(band);
        let expected = palette.colour(material).expect("a non-empty table");
        if band == BANDS - 1 {
            assert_eq!(
                expected, unknown,
                "a material past the table reads as entry 0"
            );
        } else {
            assert_eq!(expected, colour(material));
        }
        expect_colour(&frame.pixels, at, expected, &format!("material {material}"));
    }
}

/// The other half of I1: with no palette bound, both branches draw exactly
/// what they always drew. The habitat frame is probed against the appearance
/// it was handed and the classic frame against the shader's own constants, so
/// a change to either arithmetic fails here before it reaches a receipt.
#[test]
fn an_unbound_palette_keeps_the_classic_and_habitat_colours_exact() {
    let ground = Ground::grow(&Bands, EXTENT);
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(WIDTH, HEIGHT) else {
        eprintln!("no adapter; skipping unbound palette receipt");
        return;
    };
    let grade = unfogged();
    let at = probe(0.5);
    let input =
        BrickFrameInput::for_camera(&map, BrickRevision(ground.revision()), overhead(), &grade);

    let classic = tracer.capture(input).expect("classic frame");
    expect_colour(&classic.pixels, at, [0.38, 0.24, 0.13], "classic soil");

    let appearance = TerrainAppearance {
        soil: [0.30, 0.22, 0.17],
        rock: [0.25, 0.31, 0.34],
        unknown: [0.60, 0.23, 0.58],
        sky: [0.64, 0.73, 0.76],
        underground: [0.09, 0.12, 0.14],
        section_centre: [0.5, 8.0, 0.5],
        clearing_y: 8.0,
    };
    let habitat = tracer
        .capture(input.with_terrain_appearance(appearance))
        .expect("habitat frame");
    expect_colour(&habitat.pixels, at, appearance.soil, "habitat soil");

    // An empty table is not a bound palette: it leaves both branches alone,
    // to the byte, rather than drawing everything as entry 0.
    let empty = TerrainPalette::default();
    let still_classic = tracer
        .capture(input.with_terrain_palette(&empty))
        .expect("empty-palette classic frame");
    assert_eq!(still_classic.pixels, classic.pixels);
    let still_habitat = tracer
        .capture(
            input
                .with_terrain_appearance(appearance)
                .with_terrain_palette(&empty),
        )
        .expect("empty-palette habitat frame");
    assert_eq!(still_habitat.pixels, habitat.pixels);

    // And a palette does reach the habitat branch, which is the one place the
    // two could have diverged.
    let palette = TerrainPalette::new([[0.0, 0.0, 0.0]]).with_material(3, [0.0, 0.8, 0.4]);
    let repainted = tracer
        .capture(
            input
                .with_terrain_appearance(appearance)
                .with_terrain_palette(&palette),
        )
        .expect("habitat palette frame");
    expect_colour(
        &repainted.pixels,
        at,
        [0.0, 0.8, 0.4],
        "habitat soil repainted",
    );
}
