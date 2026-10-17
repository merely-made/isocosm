// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn producer_window_stamp_tracks_the_exact_camera_page() {
    let scene = ResidencyScene::grow();
    let close = selection_revision(&scene.ground, scene.focus, PAGE_RANGES[0]).unwrap();
    assert_eq!(
        close,
        selection_revision(&scene.ground, scene.focus, PAGE_RANGES[0]).unwrap()
    );

    let mut moved = ResidencyScene {
        ground: scene.ground.clone(),
        focus: scene.focus,
    };
    assert!(moved.move_focus_x(BRICK));
    assert_ne!(
        close,
        selection_revision(&moved.ground, moved.focus, PAGE_RANGES[0]).unwrap()
    );
}

#[test]
fn producer_cache_refreshes_changed_ground_inside_the_same_resident_page() {
    let mut scene = ResidencyScene::grow();
    let focus = scene.focus;
    let mut stable = StableResidency::new(&scene).expect("fixed cache under the V1 budget");
    let initial_revision = selection_revision(&scene.ground, focus, PAGE_RANGES[0]).unwrap();
    let first = stable
        .prepare_at(&scene.ground, focus, PAGE_RANGES[0])
        .unwrap()
        .expect("the first resident page");
    assert!(stable.resident_bytes() <= RESIDENT_BUDGET_BYTES);

    let floor = [focus[0], focus[1] - 1, focus[2]];
    let original = scene.ground.material(floor).expect("resident floor");
    let replacement = if original == 1 { 2 } else { 1 };
    assert_eq!(scene.ground.write([(floor, replacement)]).unwrap(), 1);
    let source_revision = selection_revision(&scene.ground, focus, PAGE_RANGES[0]).unwrap();
    assert_ne!(source_revision, initial_revision);

    let refreshed = stable
        .prepare_at(&scene.ground, focus, PAGE_RANGES[0])
        .unwrap()
        .expect("a ground edit refreshes retained resident slots");
    assert_eq!(
        refreshed.metrics.resident_range,
        first.metrics.resident_range
    );
    assert_eq!(
        refreshed.metrics.resident_bricks,
        first.metrics.resident_bricks
    );
    assert_eq!(stable.map().material_at(floor), replacement);
    assert!(
        stable.resident_bytes() <= RESIDENT_BUDGET_BYTES,
        "the producer cache stays within the fixed V1 allocation"
    );
    assert_eq!(
        source_revision,
        selection_revision(&scene.ground, focus, PAGE_RANGES[0]).unwrap()
    );
}

#[test]
fn producer_uses_the_camera_window_instead_of_the_full_depth_as_radius() {
    let camera = isometer::SlabCamera::new([0.0; 3], [0.70, -0.28, 0.70], 10.0, 16.0 / 9.0, 48.0)
        .expect("the producer's default camera");
    let window = camera.window();
    let exact = [0, 2]
        .into_iter()
        .map(|world_axis| {
            (0..3)
                .map(|axis| window.axes[axis][world_axis].abs() * window.half[axis])
                .sum::<f32>()
        })
        .fold(0.0_f32, f32::max)
        .ceil() as i32;
    assert_eq!(camera_visible_range(camera), exact);
    assert!(
        (exact as f32) < 10.0_f32 * (16.0 / 9.0) + 48.0,
        "the viewport's horizontal footprint is smaller than the slab depth"
    );
}
