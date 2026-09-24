// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The faces the sun never reaches, and the floor that keeps them visible.
//!
//! A ground face whose normal points away from the sun is lit by the ambient
//! term alone. Soil and rock are dark enough that the ambient product has no
//! channel at or above `0.2`, and the per-channel ladder's first rung needs
//! exactly that — so every such face quantised to pure black, under every
//! phase of the ordered dither. Mesocosm's oblique happens to lean onto lit
//! faces and never showed it; Eponym's default camera leans the other way
//! and showed a black notch where the ground should be.
//!
//! The fix is a floor in the quantiser rather than a brighter ambient: a found
//! ground hit the dither could still black out — brightest channel under the
//! first rung plus the dither's worst reach — is scaled whole until it clears
//! that rung under every phase, so it keeps its material's hue and loses only
//! its claim to be black. The band above the ambient case matters as much as
//! the ambient case itself: a face just over the rung was lit in most Bayer
//! phases and black in a few, which is the speckle the shipped oblique showed.
//! A face the dither cannot reach is left alone, and so is a face fog has
//! already carried clear — which is why the floor sits inside `grade`, under
//! the ladder, rather than on the lit colour before the fog mix.
//!
//! These receipts render both forwards and read the frame against the CPU
//! mirror of the same DDA, so every pixel's face, material and light are known
//! and the floor's reach can be stated rather than assumed.

use crate::{BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Grade, TraceCamera};

use super::ground;

/// The section's own numbers, so these render what the terrarium renders.
const SLAB_DEPTH: f32 = 16.0;
const HALF_HEIGHT: f32 = 28.0;
const PALETTE: u32 = 3;
/// Twenty degrees off `-z` on both free rotations — the shipped oblique.
const TILT: f32 = 20.0;
const WIDTH: u32 = 192;
const HEIGHT: u32 = 128;

/// The two forwards this is about: the one that hid the bug and the one that
/// showed it.
fn forwards() -> [(&'static str, [f32; 3]); 2] {
    let (yaw, pitch) = (TILT.to_radians(), TILT.to_radians());
    [
        (
            "meso-oblique",
            [
                -yaw.sin() * pitch.cos(),
                -pitch.sin(),
                -yaw.cos() * pitch.cos(),
            ],
        ),
        ("paredros-default", [0.70, -0.28, 0.70]),
    ]
}

/// A live tracer over the shipped procedural ground, plus the same map on the
/// CPU side for classification.
struct Probe {
    ground: isometer_core::ground::Ground,
    map: BrickMap,
    tracer: BrickTracer,
}

impl Probe {
    fn new(reason: &str) -> Option<Self> {
        let ground = ground();
        let map = BrickMap::from_ground(&ground).expect("atlas capacity");
        let Some(tracer) = BrickTracer::headless(WIDTH, HEIGHT) else {
            eprintln!("no adapter; skipping {reason}");
            return None;
        };
        Some(Self {
            ground,
            map,
            tracer,
        })
    }

    fn camera(&self, forward: [f32; 3]) -> TraceCamera {
        let top = self.ground.surface(0, 0).unwrap_or(0) as f32;
        TraceCamera::orthographic_slab(
            [0.0, top, 0.0],
            forward,
            [0.0, 1.0, 0.0],
            HALF_HEIGHT,
            1.5,
            SLAB_DEPTH,
        )
        .expect("a slab camera")
    }

    fn render(&mut self, camera: TraceCamera, grade: &Grade) -> Vec<[u8; 3]> {
        let input = BrickFrameInput::for_camera(
            &self.map,
            BrickRevision(self.ground.revision()),
            camera,
            grade,
        );
        self.tracer
            .capture(input)
            .expect("a graded section")
            .pixels
            .chunks_exact(4)
            .map(|texel| [texel[0], texel[1], texel[2]])
            .collect()
    }
}

/// The retro grade as shipped, dither and all.
fn retro() -> Grade {
    Grade::retro(PALETTE)
}

/// The same grade with the ordered dither off, so a pixel reads the rung its
/// surface actually graded to rather than the speckle over it.
fn steady() -> Grade {
    Grade {
        dither: 0.0,
        ..Grade::retro(PALETTE)
    }
}

// ---------------------------------------------------------------------------
// The CPU mirror of `tracer.wgsl`, from the ray to the byte.
// ---------------------------------------------------------------------------

/// The shader's own ndc for output pixel (x, y): clip Y is up, the frame's is
/// down, and each pixel is sampled at its centre.
fn ndc(x: u32, y: u32) -> [f32; 2] {
    [
        (x as f32 + 0.5) / WIDTH as f32 * 2.0 - 1.0,
        1.0 - (y as f32 + 0.5) / HEIGHT as f32 * 2.0,
    ]
}

#[derive(Clone, Copy, Debug)]
struct Classified {
    found: bool,
    t: f32,
    normal: [f32; 3],
    material: u8,
    in_wall: bool,
}

/// Mirrors `trace_sample`'s ray, hit and wall test on the CPU.
fn classify(map: &BrickMap, camera: TraceCamera, x: u32, y: u32) -> Classified {
    let (origin, direction) = camera.ray_at(ndc(x, y)).expect("a seeded ray");
    let seeded = [0, 1, 2].map(|i| origin[i] + direction[i] * 0.0001);
    let in_wall = map.material_at(seeded.map(|v: f32| v.floor() as i32)) != 0;
    match map.trace_ray(origin, direction, camera.far()) {
        Ok(Some(hit)) => Classified {
            found: true,
            t: hit.distance,
            normal: hit.normal,
            material: hit.material,
            in_wall,
        },
        _ => Classified {
            found: false,
            t: camera.far(),
            normal: [0.0, 1.0, 0.0],
            material: 0,
            in_wall,
        },
    }
}

/// `material_colour` with no `TerrainAppearance` — the classic mode these
/// receipts render.
fn material_colour(material: u8) -> [f32; 3] {
    match material {
        3 => [0.38, 0.24, 0.13],
        2 => [0.32, 0.34, 0.40],
        _ => [0.66, 0.20, 0.72],
    }
}

fn sun() -> [f32; 3] {
    let v = [0.4f32, 0.8, 0.3];
    let length = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v.map(|c| c / length)
}

/// `0.38 + 0.62 * max(0, dot(normal, sun))`.
fn light_for(normal: [f32; 3]) -> f32 {
    let s = sun();
    (normal[0] * s[0] + normal[1] * s[1] + normal[2] * s[2])
        .max(0.0)
        .mul_add(0.62, 0.38)
}

fn brightest(colour: [f32; 3]) -> f32 {
    colour[0].max(colour[1]).max(colour[2])
}

/// The colour a ground hit carries into the grade **before** the floor.
fn pre_floor(hit: &Classified) -> [f32; 3] {
    let light = light_for(hit.normal);
    material_colour(hit.material).map(|c| c * light)
}

/// The lowest value the per-channel ladder's first rung will accept.
const FIRST_RUNG: f32 = 0.2;
/// The shader's gate: every fogged colour the dither could still black out.
/// That is the first rung plus the worst the dither takes off a channel,
/// 0.2 + 0.05, written as the literal the shader compares against.
const FLOOR_GATE: f32 = 0.25;
/// Where the floor puts the brightest channel: clear of the first rung under
/// the worst dither phase, clear of the second under the best. See
/// `tracer.wgsl`.
const FLOOR_LIFT: f32 = 0.26;

/// The ordered dither's sixteen cells, as `bayer` holds them. Scaled to an
/// offset they run -0.5 to +0.4375, so at `Grade::retro`'s 0.10 strength the
/// worst a channel loses is 0.05 and the best it gains is 0.04375.
const BAYER: [f32; 16] = [
    0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0,
];

/// The bytes `steps_per_channel` can produce: `floor(c * 5) / 4` scaled to
/// eight bits, with the overflowing sixth level clamped onto the fifth.
const RUNGS: [u8; 6] = [0, 64, 127, 191, 255, 255];

/// The grade's fog stage, which runs before the floor and either ladder. The
/// floor gates on **this** colour, not on the lit one, which is the whole
/// reason it sits inside `grade`.
fn fogged(colour: [f32; 3], t: f32, far: f32, grade: &Grade) -> [f32; 3] {
    let mut fog =
        ((t / far - grade.fog_start) / (1.0 - grade.fog_start).max(0.001)).clamp(0.0, 1.0);
    if grade.fog_bands > 0.5 {
        fog = (fog * grade.fog_bands).floor() / grade.fog_bands;
    }
    [0, 1, 2].map(|i| colour[i] + (grade.fog[i] - colour[i]) * fog)
}

/// The shader's `grade`, both ladders, down to the byte the texture holds.
/// `floor_it` is the shader's own parameter: the ground asks for it and
/// nothing else does. Passing `false` predicts the frame as it graded before
/// the floor existed.
fn graded(
    colour: [f32; 3],
    t: f32,
    far: f32,
    grade: &Grade,
    dither: f32,
    along_hue: bool,
    floor_it: bool,
) -> [u8; 3] {
    let mixed = fogged(colour, t, far, grade);
    if grade.palette_len == 0 {
        return mixed.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8);
    }
    let mixed = if floor_it {
        let value = brightest(mixed);
        if value > 0.0 && value < FLOOR_GATE {
            mixed.map(|c| c * (FLOOR_LIFT / value))
        } else {
            mixed
        }
    } else {
        mixed
    };
    if along_hue {
        let value = brightest(mixed);
        if value < 1.0 / 512.0 {
            return [0, 0, 0];
        }
        let stepped = ((value + dither).clamp(0.0, 1.0) * 5.0).round() / 4.0;
        return mixed.map(|c| ((c * (stepped / value)).min(1.0) * 255.0).round() as u8);
    }
    mixed.map(|c| RUNGS[(((c + dither).clamp(0.0, 1.0) * 5.0).floor() as usize).min(5)])
}

/// Which ladder a ground pixel climbs, mirroring the gate at the ground hit.
fn from_above(hit: &Classified, camera: TraceCamera, pixel: (u32, u32)) -> bool {
    let down = camera.ray_at(ndc(pixel.0, pixel.1)).expect("a seeded ray").1[1] < 0.0;
    hit.normal[1] > 0.5 && down && !hit.in_wall
}

/// The byte a ground pixel would have carried **before** the floor existed,
/// at one dither offset.
fn before_floor_byte(
    hit: &Classified,
    camera: TraceCamera,
    grade: &Grade,
    pixel: (u32, u32),
    dither: f32,
) -> [u8; 3] {
    let hue = from_above(hit, camera, pixel);
    graded(pre_floor(hit), hit.t, camera.far(), grade, dither, hue, false)
}

/// **Whether a pixel was black before the floor in any Bayer phase at all.**
/// The dither is what made the old failure intermittent — a face just above
/// the first rung was lit in most phases and black in a few — so "was black"
/// has to be asked of every phase, not of the undithered frame.
fn was_black_in_some_phase(hit: &Classified, camera: TraceCamera, pixel: (u32, u32)) -> bool {
    let grade = retro();
    BAYER.iter().any(|value| {
        before_floor_byte(hit, camera, &grade, pixel, (value / 16.0 - 0.5) * grade.dither)
            == [0, 0, 0]
    })
}

/// Every pixel of a frame, classified once.
fn walk(map: &BrickMap, camera: TraceCamera) -> Vec<Classified> {
    (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).map(move |x| (x, y)))
        .map(|(x, y)| classify(map, camera, x, y))
        .collect()
}

// ---------------------------------------------------------------------------
// The receipts.
// ---------------------------------------------------------------------------

/// **The bug, as an invariant.** Ground the ray found is ground the frame
/// shows. Rendered with the shipped dither on, because the dither was part of
/// the old failure rather than a way out of it: every Bayer phase crushed an
/// ambient-lit face to black, so a black pixel here would be black under all
/// sixteen.
///
/// The control is in the same run: the Eponym forward has to actually put
/// ambient-lit ground on the screen, or the assertion is about an empty set.
#[test]
fn no_ground_pixel_grades_to_black_under_either_forward() {
    let Some(mut probe) = Probe::new("the unlit-ground receipt") else {
        return;
    };
    let mut unlit_seen = 0usize;
    for (view, forward) in forwards() {
        let camera = probe.camera(forward);
        let walked = walk(&probe.map, camera);
        let ground = walked.iter().filter(|hit| hit.found).count();
        let mut black = 0usize;
        for (label, grade) in [("dithered", retro()), ("steady", steady())] {
            let pixels = probe.render(camera, &grade);
            let dark: Vec<(u32, u32, f32, u8)> = walked
                .iter()
                .zip(&pixels)
                .enumerate()
                .filter(|(_, (hit, texel))| hit.found && **texel == [0, 0, 0])
                .map(|(i, (hit, _))| {
                    (
                        i as u32 % WIDTH,
                        i as u32 / WIDTH,
                        brightest(pre_floor(hit)),
                        hit.material,
                    )
                })
                .collect();
            println!(
                "[{view}/{label}] ground={ground} black={} e.g. {:?}",
                dark.len(),
                &dark[..dark.len().min(6)]
            );
            if label == "dithered" {
                black = dark.len();
            }
        }
        assert_eq!(black, 0, "{view}: {black} of {ground} ground pixels graded to pure black");
        unlit_seen += walked
            .iter()
            .filter(|hit| hit.found && brightest(pre_floor(hit)) < FIRST_RUNG)
            .count();
    }
    assert!(
        unlit_seen > 100,
        "only {unlit_seen} ambient-lit ground pixels across both forwards; \
         the black-pixel assertion has nothing to bite on"
    );
}

/// **The floor's own value, read off the frame.** Soil's brightest channel is
/// its red and rock's is its blue; both are lifted to the floor and both must
/// land on the ladder's first rung, which is 64. Read where no fog has reached
/// the face yet, so the rung is the floor's and not the fog's.
///
/// **Only the brightest channel is pinned, because only it is the floor's.**
/// Soil lands `(64, 0, 0)` and keeps its red; rock lands `(64, 64, 64)` and
/// reads neutral grey rather than blue. That is the five-rung ladder, not a
/// fault in the floor: rock's channels are 0.32 / 0.34 / 0.40, close enough
/// that scaling the brightest to the lift carries all three over the first
/// rung together. A material reads as itself here only when its channels are
/// further apart than one rung.
///
/// The two forwards are pooled rather than asserted one at a time: the whole
/// point of the shipped oblique is that it leans away from these faces, so it
/// contributes few of them and sometimes none of a given material.
#[test]
fn an_ambient_lit_soil_or_rock_face_sits_on_the_first_rung() {
    let Some(mut probe) = Probe::new("the first-rung receipt") else {
        return;
    };
    let grade = steady();
    // soil's red, then rock's blue.
    let wanted = [(3u8, "soil", 0usize), (2, "rock", 2)];
    let mut seen = [0usize; 2];
    let mut sample = [[0u8; 3]; 2];
    let mut stray: [Vec<(&str, [u8; 3])>; 2] = [Vec::new(), Vec::new()];
    for (view, forward) in forwards() {
        let camera = probe.camera(forward);
        let pixels = probe.render(camera, &grade);
        let walked = walk(&probe.map, camera);
        for (hit, texel) in walked.iter().zip(&pixels) {
            if !hit.found
                || brightest(pre_floor(hit)) >= FIRST_RUNG
                || hit.t / camera.far() > grade.fog_start
            {
                continue;
            }
            for (slot, (material, _, channel)) in wanted.iter().enumerate() {
                if hit.material != *material {
                    continue;
                }
                seen[slot] += 1;
                sample[slot] = *texel;
                if texel[*channel] != RUNGS[1] {
                    stray[slot].push((view, *texel));
                }
            }
        }
    }
    for (slot, (_, name, _)) in wanted.iter().enumerate() {
        println!(
            "[first-rung] {name}: {} unfogged ambient-lit faces, landing on {:?}",
            seen[slot], sample[slot]
        );
        assert!(
            seen[slot] > 0,
            "no unfogged ambient-lit {name} face in either frame"
        );
        let off = &stray[slot];
        assert!(
            off.is_empty(),
            "{} of {} ambient-lit {name} faces are off the first rung in their \
             brightest channel, e.g. (forward, texel) {:?}",
            off.len(),
            seen[slot],
            &off[..off.len().min(4)]
        );
    }
}

/// **The floor changes only what the dither could have blacked out.** Every
/// ground pixel on the per-channel ladder is predicted from the CPU mirror as
/// it would have graded before the floor existed, and compared with the frame.
/// A pixel that moved has to have been black in at least one of the sixteen
/// Bayer phases — the old failure was intermittent, so "was black" is asked of
/// every phase rather than of the undithered frame — and no pixel outside the
/// shader's gate may move at all. The gate is read off the **fogged** colour,
/// because that is where the floor now sits — directly under the ladder — so a
/// face fog has already carried clear of the first rung is never touched.
///
/// Those are the two sides of the claim: the floor lifted only what the ladder
/// could crush, and it left everything else exactly where it was.
///
/// **Sky and top faces are out of scope, and cannot be in it.** The floor sits
/// inside the ground-hit branch, so a miss never reaches it; and a face the
/// `from_above` gate sends up the hue ladder has `normal.y > 0.5`, which is
/// the sun's own direction — its brightest channel is 0.344 for soil and 0.362
/// for rock, well clear of the gate. Both are also the two paths this mirror
/// cannot predict to the byte: the sky has its own colour branch, and the hue
/// ladder lands off the rung table, where a CPU `round` and the texture's
/// unorm conversion disagree at a tie.
#[test]
fn the_floor_changes_only_pixels_the_dither_could_black_out() {
    let Some(mut probe) = Probe::new("the floor-reach receipt") else {
        return;
    };
    let grade = steady();
    let mut changed = 0usize;
    let mut never_black = Vec::new();
    let mut outside_gate = Vec::new();
    for (view, forward) in forwards() {
        let camera = probe.camera(forward);
        let pixels = probe.render(camera, &grade);
        let walked = walk(&probe.map, camera);
        let mut here = 0usize;
        for (index, (hit, texel)) in walked.iter().zip(&pixels).enumerate() {
            let pixel = (index as u32 % WIDTH, index as u32 / WIDTH);
            if !hit.found || from_above(hit, camera, pixel) {
                continue;
            }
            let before = before_floor_byte(hit, camera, &grade, pixel, 0.0);
            if before == *texel {
                continue;
            }
            changed += 1;
            here += 1;
            let value = brightest(fogged(pre_floor(hit), hit.t, camera.far(), &grade));
            let report = (view, pixel, value, hit.material, before, *texel);
            if value >= FLOOR_GATE {
                outside_gate.push(report);
            } else if !was_black_in_some_phase(hit, camera, pixel) {
                never_black.push(report);
            }
        }
        println!("[{view}/floor] changed={here}");
    }
    {
        assert!(changed > 0, "the floor changed no pixel at all");
        assert!(
            outside_gate.is_empty(),
            "{} of {changed} changed pixels are outside the floor's gate, \
             e.g. (forward, pixel, brightest, material, before, after) {:?}",
            outside_gate.len(),
            &outside_gate[..outside_gate.len().min(6)]
        );
        assert!(
            never_black.is_empty(),
            "{} of {changed} changed pixels were black in no dither phase, \
             e.g. (forward, pixel, brightest, material, before, after) {:?}",
            never_black.len(),
            &never_black[..never_black.len().min(6)]
        );
    }
}
