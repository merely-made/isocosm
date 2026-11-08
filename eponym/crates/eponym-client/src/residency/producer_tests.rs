// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isometer::{TerrainRefresh, TerrainSource};

use super::*;

#[test]
fn producer_framing_keeps_every_first_hit_through_zoom_and_perspective() {
    use crate::producer::{SceneModel, View};

    let fixture = eponym_play::fixtures::session::timed_action_world_solving(eponym_motion::SOLVER);
    let mut model = SceneModel::timed(fixture.action, fixture.keeper);
    let mut exact = CameraFootprints::default();
    let ground = model.game().world().ground();
    let mut residency = StableResidency::with_ground_limits(ground, AtlasLimits::DEFAULT).unwrap();
    let mut scene_map = None;
    let mut near = None;
    let size = [1440, 880];
    let aspect = size[0] as f32 / size[1] as f32;
    for (view, half_height) in [
        (View::Overview, 10.0),
        (View::Overview, 12.0),
        (View::Overview, 16.0),
        (View::Overview, 20.0),
        (View::Overview, 24.0),
        (View::Overview, 10.0),
        (View::FirstPerson, 10.0),
        (View::OverTheShoulder, 10.0),
    ] {
        model.view = view;
        model.camera.half_height = half_height;
        let ground = model.game().world().ground();
        let camera = model.view_camera(aspect).unwrap();
        let focus = camera.window().centre.map(|n| n.floor() as i32);
        let first_hits = exact
            .keys(ground, focus, camera.trace().unwrap(), size)
            .unwrap();
        assert!(!first_hits.is_empty(), "{view:?} at {half_height}");
        if half_height == 24.0 {
            let range = page_range(camera_visible_range(camera)).unwrap();
            assert!(
                selected_keys(ground, focus, range).len() > residency.capacity(),
                "the former rectangular page must overflow on this fixture"
            );
        }

        let terrain = residency.prepare_framed(ground, camera).unwrap();
        if let Some(map) = &mut scene_map {
            assert!(
                matches!(terrain.refresh(map, &[]).unwrap(), TerrainRefresh::Slots(_)),
                "camera moves keep the existing allocation"
            );
        } else {
            scene_map = Some(terrain.brick_map().unwrap());
        }
        assert!(
            first_hits.is_subset(&residency.current_keys),
            "{view:?} at {half_height} missed {:?}",
            first_hits
                .difference(&residency.current_keys)
                .collect::<Vec<_>>()
        );
        assert!(residency.resident_bytes() <= RESIDENT_BUDGET_BYTES);
        if view == View::Overview && half_height == 10.0 {
            match &near {
                None => near = Some(residency.current_keys.clone()),
                Some(expected) => assert_eq!(&residency.current_keys, expected),
            }
        }
        assert!(
            residency
                .prepare_framed(ground, camera)
                .unwrap()
                .changed
                .is_none(),
            "an unchanged view should produce no retarget"
        );
    }
}

#[test]
fn producer_refreshes_a_ground_edit_and_can_recover_a_skipped_publication() {
    let mut scene = ResidencyScene::grow();
    let mut stable = StableResidency::new(&scene).unwrap();
    let camera =
        isometer::SlabCamera::dimetric_2_1(scene.focus.map(|v| v as f32), 10.0, 1.6, 48.0).unwrap();
    let mut map = stable
        .prepare_framed(&scene.ground, camera)
        .unwrap()
        .brick_map()
        .unwrap();

    let floor = [scene.focus[0], scene.focus[1] - 1, scene.focus[2]];
    let original = scene.ground.material(floor).unwrap();
    let replacement = if original == 1 { 2 } else { 1 };
    assert_eq!(map.material_at(floor), original);
    assert_eq!(scene.ground.write([(floor, replacement)]).unwrap(), 1);

    let mut skipped = map.clone();
    let terrain = stable.prepare_framed(&scene.ground, camera).unwrap();
    assert!(
        matches!(terrain.refresh(&mut map, &[]).unwrap(), TerrainRefresh::Slots(ref slots) if !slots.is_empty())
    );
    assert_eq!(map.material_at(floor), replacement);

    let retry = stable.prepare_framed(&scene.ground, camera).unwrap();
    assert_eq!(
        retry.refresh(&mut skipped, &[]).unwrap(),
        TerrainRefresh::Full
    );
    assert_eq!(skipped.material_at(floor), replacement);
    assert_eq!(
        retry.refresh(&mut skipped, &[]).unwrap(),
        TerrainRefresh::Current
    );
}
