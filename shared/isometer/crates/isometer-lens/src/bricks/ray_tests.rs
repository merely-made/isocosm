// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use modulus::{BrickMap as SharedBrickMap, BrickProjectionRevision};

fn map(cells: &[([i32; 3], u8)]) -> BrickMap {
    let mut bricks = std::collections::BTreeMap::new();
    for &(cell, material) in cells {
        let key = cell.map(|v| v.div_euclid(8) as i16);
        let local = cell.map(|v| v.rem_euclid(8) as usize);
        let raw = bricks.entry(key).or_insert([0; 512]);
        raw[(local[1] * 8 + local[2]) * 8 + local[0]] = material;
    }
    BrickMap(
        SharedBrickMap::from_keys(BrickProjectionRevision(0), bricks.keys().copied(), |k| {
            bricks.get(&k).map(|raw| raw.as_slice())
        })
        .unwrap(),
    )
}

#[test]
fn nearest_material_uses_world_distance_with_negative_keys_and_nonunit_rays() {
    let scene = map(&[([-2, 2, 3], 7), ([3, 2, 3], 9)]);
    let hit = scene
        .trace_ray([-6.0, 2.5, 3.5], [4.0, 0.0, 0.0], 20.0)
        .unwrap()
        .unwrap();
    assert_eq!(
        (hit.voxel, hit.material, hit.normal),
        ([-2, 2, 3], 7, [-1.0, 0.0, 0.0])
    );
    assert!((hit.distance - 4.0).abs() < 1e-5);
    let reverse = scene
        .trace_ray([7.0, 2.5, 3.5], [-2.0, 0.0, 0.0], 20.0)
        .unwrap()
        .unwrap();
    assert_eq!((reverse.voxel, reverse.material), ([3, 2, 3], 9));
    assert!((reverse.distance - 3.0).abs() < 1e-5);
}

#[test]
fn removed_foreground_is_air_in_the_queried_presentation() {
    let full = map(&[([1, 1, 1], 2), ([4, 1, 1], 8)]);
    let cutaway = map(&[([4, 1, 1], 8)]);
    let ray = ([-1.0, 1.5, 1.5], [1.0, 0.0, 0.0], 20.0);
    assert_eq!(
        full.trace_ray(ray.0, ray.1, ray.2)
            .unwrap()
            .unwrap()
            .material,
        2
    );
    assert_eq!(
        cutaway
            .trace_ray(ray.0, ray.1, ray.2)
            .unwrap()
            .unwrap()
            .material,
        8
    );
    assert!(cutaway.trace_ray(ray.0, ray.1, 4.9).unwrap().is_none());
    assert!(
        cutaway
            .trace_ray([0.0, 8.0, 1.5], ray.1, ray.2)
            .unwrap()
            .is_none()
    );
}

#[test]
fn front_wall_inside_solid_reports_the_initial_sample() {
    let scene = map(&[([2, 1, 1], 6)]);
    let hit = scene
        .trace_ray([2.25, 1.5, 1.5], [1.0, 0.0, 0.0], 2.0)
        .unwrap()
        .unwrap();
    assert_eq!(hit.voxel, [2, 1, 1]);
    assert!((hit.distance - 0.0001).abs() < 1e-7);
    assert!(
        scene
            .trace_ray([2.25, 1.5, 1.5], [1.0, 0.0, 0.0], 0.00001)
            .unwrap()
            .is_none()
    );
}

#[test]
fn boundary_ties_follow_shader_x_then_y_order() {
    let scene = map(&[([1, 0, 0], 7), ([0, 1, 0], 9)]);
    let hit = scene
        .trace_ray([0.5, 0.5, 0.5], [1.0, 1.0, 0.0], 5.0)
        .unwrap()
        .unwrap();
    assert_eq!(hit.voxel, [1, 0, 0]);
    assert!((hit.distance - 0.5 * 2.0f32.sqrt()).abs() < 1e-5);
}

#[test]
fn invalid_rays_and_unexamined_terrain_are_not_clear_space() {
    let scene = map(&[([1600, 1, 1], 7), ([0, 0, 0], 0)]);
    for (origin, direction, far) in [
        ([f32::NAN, 0.0, 0.0], [1.0, 0.0, 0.0], 1.0),
        ([0.0; 3], [0.0; 3], 1.0),
        ([0.0; 3], [f32::INFINITY, 0.0, 0.0], 1.0),
        ([0.0; 3], [1.0, 0.0, 0.0], -1.0),
        ([0.0; 3], [1.0, 0.0, 0.0], f32::INFINITY),
    ] {
        assert_eq!(
            scene.trace_ray(origin, direction, far),
            Err(BrickRayError::InvalidRay)
        );
    }
    assert_eq!(
        scene.trace_ray([0.0, 1.5, 1.5], [1.0, 0.0, 0.0], 2000.0),
        Err(BrickRayError::TraversalLimit)
    );
}

#[test]
fn rounded_boundary_start_does_not_wrap_to_the_opposite_brick() {
    let scene = map(&[([-262_144, 0, 0], 7), ([262_143, 7, 7], 9)]);
    assert!(
        scene
            .trace_ray([262_144.0, 0.5, 0.5], [-1.0, 0.0, 0.0], 1.0)
            .unwrap()
            .is_none()
    );
}
