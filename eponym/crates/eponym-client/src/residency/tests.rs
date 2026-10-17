// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use modulus::BrickTrace;

#[path = "tests/far_window_measurement.rs"]
mod far_window_measurement;

#[test]
fn stable_cache_obeys_host_texture_limits_before_publication() {
    let scene = ResidencyScene::grow();
    let mut limits = AtlasLimits {
        max_texture_dimension_3d: 128,
        max_atlas_bytes: 8 * RESIDENT_BUDGET_BYTES,
    };
    let stable = StableResidency::with_limits(&scene, limits).unwrap();
    assert!(stable.resident_bytes() <= RESIDENT_BUDGET_BYTES);
    assert!(
        stable
            .map()
            .atlas_extent()
            .into_iter()
            .all(|axis| axis <= 128)
    );
    assert!(
        stable
            .map()
            .pointer_extent()
            .into_iter()
            .all(|axis| axis <= 128)
    );
    assert_eq!(
        stable.capacity(),
        StableResidency::new(&scene).unwrap().capacity()
    );

    // The same scene and budget fail when the device cannot hold a 128-texel
    // atlas row. Replacing host limits with defaults makes this control fail.
    limits.max_texture_dimension_3d = 64;
    assert!(matches!(
        StableResidency::with_limits(&scene, limits),
        Err(ResidencyError::BrickMap(BrickMapError::TooManyBricks {
            maximum: 0,
            ..
        }))
    ));
}

#[test]
fn stable_cache_keeps_a_stricter_host_atlas_budget() {
    let scene = ResidencyScene::grow();
    let atlas_budget = 2 * 128 * 1024;
    let stable = StableResidency::with_limits(
        &scene,
        AtlasLimits {
            max_atlas_bytes: atlas_budget,
            ..AtlasLimits::DEFAULT
        },
    )
    .unwrap();
    assert_eq!(stable.map().atlas().len() as u64, atlas_budget);
    assert_eq!(stable.capacity(), 511);
    assert!(stable.resident_bytes() <= RESIDENT_BUDGET_BYTES);
    assert!(stable.capacity() < StableResidency::new(&scene).unwrap().capacity());
}

#[test]
fn the_zoom_trace_contains_continuous_and_rapid_changes() {
    assert_eq!(zoom_distance(0), CLOSE_DISTANCE);
    assert!(zoom_distance(24) > zoom_distance(23));
    assert_eq!(zoom_distance(47), FAR_DISTANCE);
    assert_eq!(zoom_distance(48), CLOSE_DISTANCE);
    assert_eq!(zoom_distance(60), FAR_DISTANCE);
    assert_eq!(visible_range(CLOSE_DISTANCE, 16.0 / 9.0), 32);
    assert_eq!(visible_range(FAR_DISTANCE, 16.0 / 9.0), 127);
}

#[test]
fn one_budget_pages_a_region_that_cannot_be_wholly_resident() {
    let scene = ResidencyScene::grow();
    assert!(matches!(
        crate::brick::from_ground(&scene.ground),
        Err(BrickMapError::TooManyBricks { .. })
    ));
    let mut policy = ResidencyPolicy::default();
    let (close_visible, close_camera) = view(&scene, CLOSE_DISTANCE);
    let close = policy
        .prepare_view(&scene, close_visible, close_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("initial page");
    let close_keys = policy.current_keys.clone();
    assert!(
        policy
            .prepare_view(&scene, close_visible, close_camera, crate::gpu::SIZE)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        policy.prepare_view(&scene, PAGE_RANGES[2] + 1, close_camera, crate::gpu::SIZE),
        Err(ResidencyError::VisibleRange { .. })
    ));
    let (far_visible, far_camera) = view(&scene, FAR_DISTANCE);
    let far = policy
        .prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("far page");
    assert!(far.metrics.loaded_bricks > 0);
    assert!(far.metrics.resident_bricks > close.metrics.resident_bricks);
    assert!(far.metrics.resident_bytes <= RESIDENT_BUDGET_BYTES);
    assert!(policy.current_keys.is_superset(&close_keys));
    assert_exact_view(
        policy.footprints.source_map(),
        &far.map,
        far_camera,
        crate::gpu::SIZE,
    );
    assert_exact_view(
        policy.footprints.source_map(),
        &close.map,
        close_camera,
        crate::gpu::SIZE,
    );
    let (close_visible, close_camera) = view(&scene, CLOSE_DISTANCE);
    let recovered = policy
        .prepare_view(&scene, close_visible, close_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("rapid close recovery");
    assert!(recovered.metrics.evicted_bricks > 0);
    let ground_at = [scene.focus[0], scene.focus[1] - 1, scene.focus[2]];
    assert!(scene.ground.solid(ground_at));
    assert_ne!(recovered.map.material_at(ground_at), 0);
}

#[test]
fn the_stable_cache_retargets_without_changing_its_extents() {
    let mut scene = ResidencyScene::grow();
    let mut stable = StableResidency::new(&scene).expect("stable cache under budget");
    assert!(stable.resident_bytes() <= RESIDENT_BUDGET_BYTES);
    // Every whole row the budget leaves: one more would not fit.
    let rows = u64::from(stable.map().atlas_extent()[1] / 8);
    let row = stable.map().atlas().len() as u64 / rows;
    assert!(stable.resident_bytes() + row > RESIDENT_BUDGET_BYTES);
    let extents = (stable.map().pointer_extent(), stable.map().atlas_extent());

    let (close_visible, close_camera) = view(&scene, CLOSE_DISTANCE);
    let first = stable
        .prepare_view(&scene, close_visible, close_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("initial page");
    let close_keys = stable.current_keys.clone();
    assert_eq!(
        first.delta.loaded_slots.len(),
        first.metrics.resident_bricks,
        "an empty cache loads its whole first page"
    );
    assert!(
        stable
            .prepare_view(&scene, close_visible, close_camera, crate::gpu::SIZE)
            .unwrap()
            .is_none()
    );

    let (far_visible, far_camera) = view(&scene, FAR_DISTANCE);
    let far = stable
        .prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("far page");
    assert!(far.metrics.resident_bricks <= stable.capacity());
    assert_eq!(far.delta.evicted, 0, "zooming out keeps the close page");
    assert!(far.delta.retained > 0);
    assert!(stable.current_keys.is_superset(&close_keys));
    assert_exact_view(
        stable.footprints.source_map(),
        stable.map(),
        far_camera,
        crate::gpu::SIZE,
    );
    assert_eq!(
        (stable.map().pointer_extent(), stable.map().atlas_extent()),
        extents,
        "a band change must not move the fixed extents"
    );

    assert!(scene.move_focus_x(BRICK));
    let far_camera = scene.camera(
        FAR_DISTANCE,
        crate::gpu::SIZE[0] as f32 / crate::gpu::SIZE[1] as f32,
    );
    let travelled = stable
        .prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("same-band travel");
    assert!(!travelled.delta.loaded_slots.is_empty());
    assert_eq!(
        travelled.delta.loaded_slots.len(),
        travelled.metrics.loaded_bricks,
        "camera travel over unchanged Ground uploads only newly loaded slots"
    );
    assert!(travelled.delta.evicted > 0);
    assert_eq!(
        (stable.map().pointer_extent(), stable.map().atlas_extent()),
        extents,
        "travel must not move the fixed extents"
    );
    let ground_at = [scene.focus[0], scene.focus[1] - 1, scene.focus[2]];
    assert!(scene.ground.solid(ground_at));
    assert_ne!(stable.map().material_at(ground_at), 0);
}

#[test]
fn travel_within_one_page_band_advances_projection_identity() {
    let mut scene = ResidencyScene::grow();
    let mut policy = ResidencyPolicy::default();
    let (far_visible, far_camera) = view(&scene, FAR_DISTANCE);
    let first = policy
        .prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("initial page");
    assert!(scene.move_focus_x(BRICK));
    let far_camera = scene.camera(
        FAR_DISTANCE,
        crate::gpu::SIZE[0] as f32 / crate::gpu::SIZE[1] as f32,
    );
    let travelled = policy
        .prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("same-band travel page");

    assert_eq!(
        first.metrics.resident_range,
        travelled.metrics.resident_range
    );
    assert!(travelled.metrics.loaded_bricks > 0);
    assert!(travelled.metrics.evicted_bricks > 0);
    assert_eq!(
        first.map.pointer_extent(),
        travelled.map.pointer_extent(),
        "the headed far-page move must preserve pointer texture extent"
    );
    assert_eq!(
        first.map.atlas_extent(),
        travelled.map.atlas_extent(),
        "the headed far-page move must preserve atlas texture extent"
    );
    assert_eq!(
        travelled.metrics.projection_revision.0,
        first.metrics.projection_revision.0 + 1
    );
    assert_eq!(
        travelled.map.projection_revision(),
        travelled.metrics.projection_revision
    );
    assert!(
        policy
            .prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE)
            .unwrap()
            .is_none()
    );
}

#[test]
fn actual_intermediate_camera_keeps_its_shared_first_hits_resident() {
    let scene = ResidencyScene::grow();
    let mut policy = ResidencyPolicy::default();
    let distance = 44.0;
    let (visible, camera) = view(&scene, distance);
    let prepared = policy
        .prepare_view(&scene, visible, camera, crate::gpu::SIZE)
        .unwrap()
        .expect("intermediate camera page");
    assert!(prepared.metrics.resident_bytes <= RESIDENT_BUDGET_BYTES);
    assert_exact_view(
        policy.footprints.source_map(),
        &prepared.map,
        camera,
        crate::gpu::SIZE,
    );
    assert!(
        policy
            .prepare_view(&scene, visible, camera, crate::gpu::SIZE)
            .unwrap()
            .is_none()
    );
}

#[test]
fn refused_world_change_preserves_stable_anchor_and_map_then_recovers() {
    let mut scene = ResidencyScene::grow();
    let original_focus = scene.focus;
    let limits = AtlasLimits {
        max_texture_dimension_3d: 2048,
        max_atlas_bytes: 256 * 1024,
    };
    let mut stable = StableResidency::with_limits(&scene, limits).unwrap();
    let (near_visible, near_camera) = view(&scene, CLOSE_DISTANCE);
    let near = stable
        .prepare_view(&scene, near_visible, near_camera, crate::gpu::SIZE)
        .unwrap()
        .expect("initial near page");
    let original_anchor = stable.near_anchor_keys.clone();
    let original_keys = stable.current_keys.clone();
    let original_revision = stable.projection_revision();
    let original_pointers = stable.map().pointers().to_vec();
    let original_atlas = stable.map().atlas().to_vec();

    assert!(scene.move_focus_x(BRICK));
    let (far_visible, far_camera) = view(&scene, FAR_DISTANCE);
    let far_keys = stable
        .footprints
        .keys(&scene.ground, scene.focus, far_camera, crate::gpu::SIZE)
        .unwrap();
    assert!(far_keys.len() > stable.capacity());
    let mut requested = far_keys;
    requested.extend(original_anchor.iter().copied());
    assert!(requested.len() > stable.capacity());
    assert!(matches!(
        stable.prepare_view(&scene, far_visible, far_camera, crate::gpu::SIZE),
        Err(ResidencyError::BrickMap(
            BrickMapError::TooManyBricks { .. }
        ))
    ));

    assert_eq!(stable.near_anchor_keys, original_anchor);
    assert_eq!(stable.current_keys, original_keys);
    assert_eq!(stable.projection_revision(), original_revision);
    assert_eq!(stable.map().pointers(), original_pointers);
    assert_eq!(stable.map().atlas(), original_atlas);

    assert!(scene.move_focus_x(-BRICK));
    assert_eq!(scene.focus, original_focus);
    let low_resolution = [1, 1];
    let far_camera = scene.camera(FAR_DISTANCE, 1.0);
    let far_visible = visible_range(FAR_DISTANCE, 1.0);
    let recovered = stable
        .prepare_view(&scene, far_visible, far_camera, low_resolution)
        .unwrap()
        .expect("coarse far view fits alongside the retained near page");
    assert_eq!(recovered.metrics.projection_revision, stable.map().projection_revision());
    if stable.current_keys == original_keys {
        assert_eq!(recovered.metrics.projection_revision, original_revision);
    } else {
        assert!(recovered.metrics.projection_revision.0 > original_revision.0);
    }
    assert_eq!(recovered.delta.evicted, 0);
    assert!(stable.current_keys.is_superset(&original_keys));
    assert_eq!(stable.near_anchor_keys, original_anchor);
    assert!(stable.current_keys.len() <= stable.capacity());
    assert_eq!(near.metrics.resident_bricks, original_keys.len());
}

#[test]
fn same_key_ground_edit_refreshes_content_without_changing_projection_identity() {
    use isometer::core::ground::{ROCK, SOIL};

    let mut scene = ResidencyScene::grow();
    let size = [1, 1];
    let camera = scene.camera(CLOSE_DISTANCE, 1.0);
    let visible = visible_range(CLOSE_DISTANCE, 1.0);
    let mut stable = StableResidency::new(&scene).unwrap();
    stable
        .prepare_view(&scene, visible, camera, size)
        .unwrap()
        .expect("initial one-pixel selection");
    let selected_keys = stable.current_keys.clone();
    let key = *selected_keys.iter().next().expect("camera hits terrain");
    let projection = stable.map().projection_revision();
    let (brick, origin) = scene.ground.brick_materials(key).unwrap();
    let (index, old_material) = brick
        .raw()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, material)| *material != 0)
        .expect("resident brick has solid material");
    let local = [
        (index % BRICK as usize) as i32,
        (index / (BRICK * BRICK) as usize) as i32,
        (index / BRICK as usize % BRICK as usize) as i32,
    ];
    let at = [
        origin[0] + local[0],
        origin[1] + local[1],
        origin[2] + local[2],
    ];
    let new_material = if old_material == SOIL { ROCK } else { SOIL };
    assert_eq!(scene.ground.write([(at, new_material)]).unwrap(), 1);
    let content_revision = scene.ground.revision();

    let updated = stable
        .prepare_view(&scene, visible, camera, size)
        .unwrap()
        .expect("content revision refresh");
    assert_eq!(stable.current_keys, selected_keys);
    assert_eq!(stable.ground_revision, Some(content_revision));
    assert_eq!(updated.metrics.projection_revision, projection);
    assert_eq!(stable.map().projection_revision(), projection);
    assert_eq!(
        updated.metrics.projection_revision,
        stable.map().projection_revision()
    );
    assert_eq!(updated.delta.loaded_slots.len(), selected_keys.len());
    assert_eq!(stable.map().material_at(at), new_material);
}

fn view(scene: &ResidencyScene, distance: f32) -> (i32, TraceCamera) {
    let aspect = crate::gpu::SIZE[0] as f32 / crate::gpu::SIZE[1] as f32;
    (
        visible_range(distance, aspect),
        scene.camera(distance, aspect),
    )
}

fn assert_exact_view(source: &BrickMap, resident: &BrickMap, camera: TraceCamera, size: [u32; 2]) {
    for y in 0..size[1] {
        for x in 0..size[0] {
            let ndc = [
                2.0 * (x as f32 + 0.5) / size[0] as f32 - 1.0,
                1.0 - 2.0 * (y as f32 + 0.5) / size[1] as f32,
            ];
            let (origin, direction) = camera.ray_at(ndc).unwrap();
            match (
                source.trace(origin, direction, camera.far()),
                resident.trace(origin, direction, camera.far()),
            ) {
                (BrickTrace::Hit(source), BrickTrace::Hit(resident)) => {
                    assert_eq!(source.voxel, resident.voxel, "pixel ({x},{y})")
                },
                (BrickTrace::Clear, BrickTrace::Clear) => {},
                (source, resident) => {
                    panic!("pixel ({x},{y}) differs: source={source:?}, resident={resident:?}")
                },
            }
        }
    }
}
