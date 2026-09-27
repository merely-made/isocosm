// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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
    let close = policy
        .prepare(&scene, PAGE_RANGES[0])
        .unwrap()
        .expect("initial page");
    assert!(policy.prepare(&scene, PAGE_RANGES[0]).unwrap().is_none());
    assert!(matches!(
        policy.prepare(&scene, PAGE_RANGES[2] + 1),
        Err(ResidencyError::VisibleRange { .. })
    ));
    let far = policy
        .prepare(&scene, PAGE_RANGES[2])
        .unwrap()
        .expect("far page");
    assert!(far.metrics.loaded_bricks > 0);
    assert!(far.metrics.resident_bricks > close.metrics.resident_bricks);
    assert!(far.metrics.resident_bytes <= RESIDENT_BUDGET_BYTES);
    let recovered = policy
        .prepare(&scene, PAGE_RANGES[0])
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

    let first = stable
        .prepare(&scene, PAGE_RANGES[0])
        .unwrap()
        .expect("initial page");
    assert_eq!(
        first.delta.loaded_slots.len(),
        first.metrics.resident_bricks,
        "an empty cache loads its whole first page"
    );
    assert!(stable.prepare(&scene, PAGE_RANGES[0]).unwrap().is_none());

    let far = stable
        .prepare(&scene, PAGE_RANGES[2])
        .unwrap()
        .expect("far page");
    assert!(far.metrics.resident_bricks <= stable.capacity());
    assert_eq!(far.delta.evicted, 0, "zooming out keeps the close page");
    assert!(far.delta.retained > 0);
    assert_eq!(
        (stable.map().pointer_extent(), stable.map().atlas_extent()),
        extents,
        "a band change must not move the fixed extents"
    );

    assert!(scene.move_focus_x(BRICK));
    let travelled = stable
        .prepare(&scene, PAGE_RANGES[2])
        .unwrap()
        .expect("same-band travel");
    assert!(!travelled.delta.loaded_slots.is_empty());
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
    let first = policy
        .prepare(&scene, PAGE_RANGES[2])
        .unwrap()
        .expect("initial page");
    assert!(scene.move_focus_x(BRICK));
    let travelled = policy
        .prepare(&scene, PAGE_RANGES[2])
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
    assert!(policy.prepare(&scene, PAGE_RANGES[2]).unwrap().is_none());
}
