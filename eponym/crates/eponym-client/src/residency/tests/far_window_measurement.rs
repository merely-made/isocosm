// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ignored, measurement-only exact camera-ray footprint diagnostic.
//!
//! This deliberately does not feed the production residency policy. It walks
//! native Ground occupancy using the shared brick DDA's bounds, start offset,
//! tie order, crossing arithmetic, far cutoff, and traversal limit, then
//! reports the unique bricks containing each pixel's first terrain hit.

use super::*;
use std::{collections::BTreeSet, time::Instant};

const V1_SIZE: [u32; 2] = [1280, 720];
const DDA_STEPS: usize = 1024;

#[test]
#[ignore = "full-resolution exact V1 ray footprint measurement"]
fn measure_v1_near_and_far_first_hit_brick_unions() {
    let scene = ResidencyScene::grow();
    let min = [0, 1, 2].map(|axis| {
        scene
            .ground
            .keys()
            .map(|key| i32::from(key[axis]))
            .min()
            .expect("generated Ground has bricks")
    });
    let max = [0, 1, 2].map(|axis| {
        scene
            .ground
            .keys()
            .map(|key| i32::from(key[axis]))
            .max()
            .expect("generated Ground has bricks")
    });
    let low = min.map(|key| key * BRICK);
    let high = max.map(|key| (key + 1) * BRICK);

    for (name, distance, page) in [
        ("near", CLOSE_DISTANCE, PAGE_RANGES[0]),
        ("far", FAR_DISTANCE, PAGE_RANGES[2]),
    ] {
        let camera = scene.camera(distance, V1_SIZE[0] as f32 / V1_SIZE[1] as f32);
        let began = Instant::now();
        let mut keys = BTreeSet::new();
        let mut hits = 0_u64;
        let mut misses = 0_u64;
        let mut exhausted = 0_u64;
        for y in 0..V1_SIZE[1] {
            for x in 0..V1_SIZE[0] {
                // Pixel-center NDC, matching the full-screen shader varying.
                let ndc = [
                    (2.0 * (x as f32 + 0.5) / V1_SIZE[0] as f32) - 1.0,
                    1.0 - (2.0 * (y as f32 + 0.5) / V1_SIZE[1] as f32),
                ];
                let (eye, direction) = camera.ray_at(ndc).expect("V1 ray is valid");
                match first_ground_hit(&scene.ground, low, high, eye, direction, camera.far()) {
                    Walk::Hit(voxel) => {
                        hits += 1;
                        keys.insert(voxel.map(|coordinate| {
                            i16::try_from(coordinate.div_euclid(BRICK))
                                .expect("native Ground keys fit i16")
                        }));
                    },
                    Walk::Miss => misses += 1,
                    Walk::Exhausted => exhausted += 1,
                }
            }
        }
        let elapsed_ms = began.elapsed().as_millis();
        println!(
            "[v1-ray-footprint] name={name} size={}x{} distance={distance} page={page} hits={hits} misses={misses} exhausted={exhausted} unique_first_hit_bricks={} fits_map2047={} fits_stable1791={} elapsed_ms={elapsed_ms}",
            V1_SIZE[0],
            V1_SIZE[1],
            keys.len(),
            keys.len() <= 2047,
            keys.len() <= 1791,
        );
        assert_eq!(
            exhausted, 0,
            "diagnostic must not treat an incomplete ray as a miss"
        );
    }
}

#[test]
#[ignore = "full-resolution shared source-map DDA parity and CPU sizing"]
fn measure_v1_shared_source_map_trace_parity() {
    let scene = ResidencyScene::grow();
    let keys: Vec<_> = scene.ground.keys().collect();
    let min = [0, 1, 2].map(|axis| {
        keys.iter()
            .map(|key| i32::from(key[axis]))
            .min()
            .expect("generated Ground has bricks")
    });
    let max = [0, 1, 2].map(|axis| {
        keys.iter()
            .map(|key| i32::from(key[axis]))
            .max()
            .expect("generated Ground has bricks")
    });
    let pointer_extent = [0, 1, 2].map(|axis| (max[axis] - min[axis] + 1) as u32);
    // This transient CPU-only source index is capped independently of V1's
    // one-MiB residency budget. It is never passed to an uploader or renderer.
    let source_limits = AtlasLimits {
        max_texture_dimension_3d: 2048,
        max_atlas_bytes: 32 * 1024 * 1024,
    };
    assert!(keys.len() <= source_limits.max_bricks());
    assert!(
        pointer_extent
            .iter()
            .all(|axis| *axis <= source_limits.max_texture_dimension_3d)
    );
    let pointer_bytes = pointer_extent
        .iter()
        .try_fold(1_usize, |cells, axis| cells.checked_mul(*axis as usize))
        .and_then(|cells| cells.checked_mul(std::mem::size_of::<u32>()))
        .expect("generated source pointer volume fits usize");
    let row_slots = (modulus::ATLAS_SLOTS_X * modulus::ATLAS_SLOTS_Z) as usize;
    let atlas_rows = (keys.len() + 1).div_ceil(row_slots).max(1);
    let expected_atlas_bytes = atlas_rows * row_slots * modulus::BRICK_EDGE.pow(3) as usize;
    let cpu_source_ceiling = 64 * 1024 * 1024;
    let ray_count = V1_SIZE[0] as usize * V1_SIZE[1] as usize;
    let expected_bytes = ray_count * std::mem::size_of::<Option<[i32; 3]>>();
    assert!(expected_atlas_bytes <= source_limits.max_atlas_bytes as usize);
    assert!(pointer_bytes + expected_atlas_bytes + expected_bytes <= cpu_source_ceiling);
    let build_started = Instant::now();
    let mut source = BrickMap::with_limits(
        BrickProjectionRevision(0),
        keys.len(),
        pointer_extent,
        source_limits,
    )
    .expect("the bounded source map fits the diagnostic CPU cap");
    source
        .retarget(BrickProjectionRevision(1), keys.iter().copied(), |key| {
            scene
                .ground
                .brick_materials(key)
                .map(|(brick, _)| brick.raw())
        })
        .expect("all exact Ground payloads populate the shared map");
    let build_ms = build_started.elapsed().as_millis();
    let pointer_bytes = source.pointers().len() * std::mem::size_of::<u32>();
    let atlas_bytes = source.atlas().len();
    let cpu_source_bytes = pointer_bytes + atlas_bytes;
    assert_eq!(atlas_bytes, expected_atlas_bytes);
    assert!(cpu_source_bytes + expected_bytes <= cpu_source_ceiling);

    let min_voxel = min.map(|key| key * BRICK);
    let high_voxel = max.map(|key| (key + 1) * BRICK);
    for (name, distance, page) in [
        ("near", CLOSE_DISTANCE, PAGE_RANGES[0]),
        ("far", FAR_DISTANCE, PAGE_RANGES[2]),
    ] {
        let camera = scene.camera(distance, V1_SIZE[0] as f32 / V1_SIZE[1] as f32);
        let native_started = Instant::now();
        let mut native_expected = Vec::with_capacity(ray_count);
        let mut native_hits = 0_u64;
        let mut native_misses = 0_u64;
        let mut native_exhausted = 0_u64;
        for y in 0..V1_SIZE[1] {
            for x in 0..V1_SIZE[0] {
                let ndc = [
                    (2.0 * (x as f32 + 0.5) / V1_SIZE[0] as f32) - 1.0,
                    1.0 - (2.0 * (y as f32 + 0.5) / V1_SIZE[1] as f32),
                ];
                let (eye, direction) = camera.ray_at(ndc).expect("V1 ray is valid");
                match first_ground_hit(
                    &scene.ground,
                    min_voxel,
                    high_voxel,
                    eye,
                    direction,
                    camera.far(),
                ) {
                    Walk::Hit(voxel) => {
                        native_hits += 1;
                        native_expected.push(Some(voxel));
                    },
                    Walk::Miss => {
                        native_misses += 1;
                        native_expected.push(None);
                    },
                    Walk::Exhausted => {
                        native_exhausted += 1;
                        native_expected.push(None);
                    },
                }
            }
        }
        let native_ms = native_started.elapsed().as_millis();
        assert_eq!(native_expected.len(), ray_count);
        let expected_capacity_bytes =
            native_expected.capacity() * std::mem::size_of::<Option<[i32; 3]>>();
        assert!(cpu_source_bytes + expected_capacity_bytes <= cpu_source_ceiling);
        assert_eq!(native_exhausted, 0, "native baseline ray budget exhausted");

        let shared_started = Instant::now();
        let mut keys = BTreeSet::new();
        let mut hits = 0_u64;
        let mut misses = 0_u64;
        let mut exhausted = 0_u64;
        for index in 0..ray_count {
            let x = index % V1_SIZE[0] as usize;
            let y = index / V1_SIZE[0] as usize;
            let ndc = [
                (2.0 * (x as f32 + 0.5) / V1_SIZE[0] as f32) - 1.0,
                1.0 - (2.0 * (y as f32 + 0.5) / V1_SIZE[1] as f32),
            ];
            let (eye, direction) = camera.ray_at(ndc).expect("V1 ray is valid");
            match (
                native_expected[index],
                source.trace(eye, direction, camera.far()),
            ) {
                (Some(voxel), modulus::BrickTrace::Hit(hit)) => {
                    assert_eq!(
                        hit.voxel, voxel,
                        "{name} shared hit differs at pixel ({x},{y})"
                    );
                    hits += 1;
                    keys.insert(voxel.map(|coordinate| {
                        i16::try_from(coordinate.div_euclid(BRICK))
                            .expect("native Ground keys fit i16")
                    }));
                },
                (None, modulus::BrickTrace::Clear) => misses += 1,
                (_, modulus::BrickTrace::Exhausted) => exhausted += 1,
                (expected, trace) => panic!(
                    "{name} DDA parity differs at pixel ({x},{y}): ground={expected:?}, shared={trace:?}"
                ),
            }
        }
        let shared_ms = shared_started.elapsed().as_millis();
        assert_eq!(exhausted, 0, "shared source-map ray budget exhausted");
        assert_eq!(hits, native_hits);
        assert_eq!(misses, native_misses);
        println!(
            "[v1-shared-source-map] name={name} size={}x{} distance={distance} page={page} ground_keys={} pointer_extent={pointer_extent:?} pointer_bytes={pointer_bytes} atlas_bytes={atlas_bytes} cpu_source_bytes={cpu_source_bytes} expected_bytes={expected_capacity_bytes} combined_transient_bytes={} build_ms={build_ms} native_hits={native_hits} native_misses={native_misses} native_exhausted={native_exhausted} native_ms={native_ms} shared_hits={hits} shared_misses={misses} shared_exhausted={exhausted} unique_first_hit_bricks={} cpu_source_bytes_le_v1_budget={} shared_trace_ms={shared_ms} production_gpu_upload_bytes=0 production_v1_budget_bytes={RESIDENT_BUDGET_BYTES} production_cap_changed=false",
            V1_SIZE[0],
            V1_SIZE[1],
            scene.ground.keys().count(),
            cpu_source_bytes + expected_capacity_bytes,
            keys.len(),
            cpu_source_bytes as u64 <= RESIDENT_BUDGET_BYTES,
        );
        assert_eq!(
            exhausted, 0,
            "shared and native walks must not exhaust the DDA budget"
        );
        drop(native_expected);
    }
}

enum Walk {
    Hit([i32; 3]),
    Miss,
    Exhausted,
}

/// Temporary diagnostic-only copy of the shared map walk over a Ground
/// occupancy callback. Keep this out of production until its footprint wins.
fn first_ground_hit(
    ground: &isometer::core::ground::Ground,
    low: [i32; 3],
    high: [i32; 3],
    eye: [f32; 3],
    direction: [f32; 3],
    far: f32,
) -> Walk {
    let [enter, exit] = ground_ray_box(low, high, far, eye, direction);
    let start_t = enter.max(0.0) + 0.0001;
    if enter > exit || start_t > exit {
        return Walk::Miss;
    }
    let mut voxel = [0, 1, 2].map(|axis| {
        ((eye[axis] + direction[axis] * start_t).floor() as i32)
            .max(low[axis])
            .min(high[axis] - 1)
    });
    let step = direction.map(|value| if value >= 0.0 { 1 } else { -1 });
    let mut crossings = [0, 1, 2].map(|axis| crossing(eye[axis], direction[axis], voxel[axis]));
    for _ in 0..DDA_STEPS {
        // BrickMap reads air outside its pointer volume. Ground::solid treats
        // y < 0 as bedrock, so apply the map bounds before its occupancy query.
        if (0..3).all(|axis| low[axis] <= voxel[axis] && voxel[axis] < high[axis])
            && ground.solid(voxel)
        {
            return Walk::Hit(voxel);
        }
        let axis = if crossings[0] <= crossings[1] && crossings[0] <= crossings[2] {
            0
        } else if crossings[1] <= crossings[2] {
            1
        } else {
            2
        };
        let t = crossings[axis];
        voxel[axis] += step[axis];
        crossings[axis] = crossing(eye[axis], direction[axis], voxel[axis]);
        if t > exit || t > far {
            return Walk::Miss;
        }
    }
    Walk::Exhausted
}

fn ground_ray_box(
    low: [i32; 3],
    high: [i32; 3],
    far: f32,
    eye: [f32; 3],
    direction: [f32; 3],
) -> [f32; 2] {
    let mut enter = 0.0_f32;
    let mut exit = far;
    for axis in 0..3 {
        let d = direction[axis];
        if d.abs() < 1e-6 {
            if eye[axis] < low[axis] as f32 || eye[axis] >= high[axis] as f32 {
                return [1.0, -1.0];
            }
            continue;
        }
        let a = (low[axis] as f32 - eye[axis]) / d;
        let b = (high[axis] as f32 - eye[axis]) / d;
        enter = enter.max(a.min(b));
        exit = exit.min(a.max(b));
    }
    [enter, exit]
}

fn crossing(origin: f32, direction: f32, voxel: i32) -> f32 {
    if direction > 1e-6 {
        return ((voxel + 1) as f32 - origin) / direction;
    }
    if direction < -1e-6 {
        return (voxel as f32 - origin) / direction;
    }
    1e30
}
