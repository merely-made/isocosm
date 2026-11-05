// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
#[ignore = "explicit R3 actual-device immutable-query control"]
fn frozen_body_queries_keep_camera_geometry_parts_and_extent_across_the_next_prepare() {
    let (device, queue) = device().expect("R3 control requires an actual GPU adapter");
    let old = document(91);
    let next = document(92);
    let volumes = DeclaredExtentVolumes::from_documents([&old, &next], 9);
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &[body(SubjectKey(4), &old, [0.0; 3])], &volumes);
    let frozen = scene.query_snapshot().unwrap();
    let clone = frozen.clone();
    let captured = frozen.capture_stats();
    assert_eq!(captured.placed_bodies, 1);
    assert_eq!(clone.capture_stats(), captured);

    let hit = frozen.pick_at_pixel([SIZE / 2; 2]).unwrap().unwrap();
    let Pick::Body(hit) = hit else {
        panic!("fixture body must be picked");
    };
    let selected = frozen.select_part(SubjectKey(4), None, false).unwrap();
    assert!(frozen.validate_pick(hit));
    assert!(frozen.validates_address(selected));

    scene.resize(SIZE + 2, SIZE + 4);
    assert_eq!(scene.pick([0.0; 2]), Err(BodyPickError::NotReady));
    let camera_b = SlabCamera {
        centre: [100.0, 0.0, 0.0],
        ..camera()
    };
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera: camera_b,
                bodies: &[body(SubjectKey(9), &next, [100.0, 0.0, 0.0])],
                volumes: SceneVolumes::DeclaredSolid(&volumes),
                terrain: None,
                dirty: &[],
                grade: isometer_lens::Grade::retro(3),
                terrain_appearance: None,
                body_budget: 8,
                capsules: None,
            },
            &mut Host,
        )
        .unwrap();
    scene.queue.submit([encoder.finish()]);
    let changed = scene.query_snapshot().unwrap();
    assert_ne!(changed.generation(), frozen.generation());
    assert_eq!(changed.camera(), camera_b);
    assert_eq!(changed.extent(), [SIZE + 2, SIZE + 4]);
    assert_eq!(frozen.camera(), camera());
    assert_eq!(frozen.extent(), [SIZE; 2]);
    assert_eq!(
        frozen.capture_stats(),
        captured,
        "later prepare cannot relabel copied query records"
    );
    assert_eq!(
        frozen.pick_at_pixel([SIZE / 2; 2]).unwrap(),
        Some(Pick::Body(hit))
    );
    assert_eq!(clone.pick([0.0; 2]).unwrap(), Some(Pick::Body(hit)));
    assert_eq!(
        frozen.select_part(SubjectKey(4), None, false),
        Some(selected)
    );
    assert!(!changed.validate_pick(hit));
    assert!(!changed.validates_address(selected));
}

#[test]
#[ignore = "explicit R3 actual-device immutable-query terrain control"]
fn frozen_terrain_is_not_rebound_to_a_later_map_or_terrain_flag() {
    let (device, queue) = device().expect("R3 control requires an actual GPU adapter");
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();
    let volumes = DeclaredExtentVolumes::from_documents(std::iter::empty::<&BodyDocument>(), 9);
    iso_render(&mut scene, &[], &volumes, Some(&field()));
    let frozen = scene.query_snapshot().unwrap();
    let captured = frozen.capture_stats();
    assert!(captured.terrain_present);
    assert!(captured.terrain_pointer_words > 0 && captured.terrain_atlas_bytes > 0);
    let before = frozen.pick([0.0; 2]).unwrap();
    assert!(matches!(before, Some(Pick::Terrain(_))));
    let ridge = ridged();
    scene.set_terrain_map(isometer_lens::BrickMap::from_ground(&ridge).unwrap());
    iso_render(&mut scene, &[], &volumes, Some(&ridge));
    assert_ne!(
        scene.pick([0.0; 2]).unwrap(),
        before,
        "the later map really changes the queried surface"
    );
    iso_render(&mut scene, &[], &volumes, None);
    assert_eq!(scene.pick([0.0; 2]).unwrap(), None);
    assert_eq!(frozen.pick([0.0; 2]).unwrap(), before);
    assert_eq!(frozen.pick_at_pixel([ISO_SIZE / 2; 2]).unwrap(), before);
    assert_eq!(frozen.capture_stats(), captured);
}
