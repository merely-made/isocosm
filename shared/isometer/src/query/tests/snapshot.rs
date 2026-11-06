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
    assert!(captured.terrain_payload_copied);
    assert!(captured.terrain_pointer_words > 0 && captured.terrain_atlas_bytes > 0);
    let before = frozen.pick([0.0; 2]).unwrap();
    assert!(matches!(before, Some(Pick::Terrain(_))));
    iso_render(&mut scene, &[], &volumes, Some(&field()));
    let reused = scene.query_snapshot().unwrap();
    assert!(reused.capture_stats().terrain_present);
    assert!(!reused.capture_stats().terrain_payload_copied);
    assert_eq!(reused.pick([0.0; 2]).unwrap(), before);

    let ridge = ridged();
    scene.set_terrain_map(isometer_lens::BrickMap::from_ground(&ridge).unwrap());
    assert!(scene.query_snapshot().is_none());
    iso_render(&mut scene, &[], &volumes, Some(&ridge));
    let replacement = scene.query_snapshot().unwrap();
    assert!(replacement.capture_stats().terrain_payload_copied);
    assert_ne!(
        replacement.pick([0.0; 2]).unwrap(),
        before,
        "the later map really changes the queried surface"
    );
    iso_render(&mut scene, &[], &volumes, None);
    let terrain_free = scene.query_snapshot().unwrap();
    assert!(!terrain_free.capture_stats().terrain_present);
    assert_eq!(terrain_free.pick([0.0; 2]).unwrap(), None);
    iso_render(&mut scene, &[], &volumes, Some(&ridge));
    let restored = scene.query_snapshot().unwrap();
    assert!(restored.capture_stats().terrain_present);
    assert!(!restored.capture_stats().terrain_payload_copied);
    assert_eq!(restored.pick([0.0; 2]), replacement.pick([0.0; 2]));
    assert_eq!(frozen.pick([0.0; 2]).unwrap(), before);
    assert_eq!(frozen.pick_at_pixel([ISO_SIZE / 2; 2]).unwrap(), before);
    assert_eq!(frozen.capture_stats(), captured);
}

#[test]
#[ignore = "explicit R9 actual-device failed terrain refresh control"]
fn refused_terrain_refresh_keeps_frozen_queries_and_retries_the_upload() {
    use crate::scene::TerrainRefresh;
    use isometer_lens::BrickMap;

    struct RefusingRefresh<'a>(&'a Ground);

    impl TerrainSource for RefusingRefresh<'_> {
        fn revision(&self) -> u64 {
            self.0.revision()
        }

        fn brick_map(&self) -> Result<BrickMap, String> {
            GroundTerrain(self.0).brick_map()
        }

        fn refresh(
            &self,
            map: &mut BrickMap,
            dirty: &[[i16; 3]],
        ) -> Result<TerrainRefresh, String> {
            GroundTerrain(self.0).refresh(map, dirty)?;
            Err("fixture refuses after refreshing the brick".into())
        }
    }

    let (device, queue) = device().expect("R9 control requires an actual GPU adapter");
    let mut scene = Scene::new(device, queue, ISO_SIZE, ISO_SIZE).unwrap();
    let volumes = DeclaredExtentVolumes::from_documents(std::iter::empty::<&BodyDocument>(), 9);
    let mut ground = field();
    iso_render(&mut scene, &[], &volumes, Some(&ground));
    let frozen = scene.query_snapshot().unwrap();
    let Some(Pick::Terrain(before)) = frozen.pick([0.0; 2]).unwrap() else {
        panic!("fixture must pick terrain");
    };
    let material = if before.material == 7 { 9 } else { 7 };
    assert_eq!(ground.write([(before.voxel, material)]).unwrap(), 1);
    let original_projection = scene.terrain_map().unwrap().projection_revision();

    let mut encoder = scene.device.create_command_encoder(&Default::default());
    let result = scene.render(
        &mut encoder,
        SceneFrame {
            camera: iso_camera(),
            bodies: &[],
            volumes: SceneVolumes::DeclaredSolid(&volumes),
            terrain: Some(&RefusingRefresh(&ground)),
            dirty: &[before.brick],
            grade: isometer_lens::Grade::retro(3),
            terrain_appearance: None,
            body_budget: 8,
            capsules: None,
        },
        &mut Host,
    );
    assert_eq!(
        result.unwrap_err(),
        "fixture refuses after refreshing the brick"
    );
    assert!(scene.terrain_upload_pending());
    assert!(scene.query_snapshot().is_none());
    assert_eq!(frozen.pick([0.0; 2]).unwrap(), Some(Pick::Terrain(before)));

    iso_render(&mut scene, &[], &volumes, Some(&ground));
    assert!(!scene.terrain_upload_pending());
    assert_eq!(
        scene.terrain_map().unwrap().projection_revision(),
        original_projection
    );
    let uploaded = scene.terrain_diagnostics().unwrap();
    assert!(!uploaded.map_recreated);
    assert!(uploaded.full_map_upload && uploaded.brick_upload_bytes > 0);
    let retried = scene.query_snapshot().unwrap();
    assert!(retried.capture_stats().terrain_payload_copied);
    assert_eq!(
        retried.pick([0.0; 2]).unwrap(),
        Some(Pick::Terrain(TerrainHit { material, ..before }))
    );
    assert_eq!(frozen.pick([0.0; 2]).unwrap(), Some(Pick::Terrain(before)));
}
