// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

#[test]
#[ignore = "explicit R3 actual-device staged scene publication"]
fn pending_scene_keeps_accepted_queries_and_promotes_only_its_frozen_input() {
    let (mut session, participant, _, window, site) = fixture();
    let other = *session
        .sim
        .state()
        .sites
        .keys()
        .find(|id| **id != site)
        .unwrap();
    let second = session
        .attach_window(
            participant,
            "mesocosm".into(),
            cosmograph::attention::WindowAttention {
                examined: Some(cosmograph::attention::Region::Site(other)),
                ..Default::default()
            },
        )
        .unwrap();
    let handle_b = SceneHandle::default();
    handle_b
        .update(
            &session,
            participant,
            second.id(),
            cosmograph::directing::readings::Mode::Creative,
            other,
            0,
        )
        .unwrap();
    let scene_b = handle_b.snapshot().unwrap().scene.clone();
    let saved = session.save();
    let handle = SceneHandle::default();
    handle
        .update(
            &session,
            participant,
            window,
            cosmograph::directing::readings::Mode::Survival,
            site,
            0,
        )
        .unwrap();
    let native = handle.snapshot().unwrap();
    let size = [192, 144];
    let camera = native.camera(size[0] as f32 / size[1] as f32);
    let view_a = Presentation {
        mode: section::CameraMode::default(),
        depth: None,
        forward: camera.forward,
        centre: camera.centre,
        half_height: camera.half_height,
        bodies: BodyMode::Voxels,
        budget: section::DEFAULT_BODY_BUDGET,
        light: 1.0,
        sky: section::SkyBlock::fixed(),
        terrain: section::TerrainStyle::default(),
        terrain_y: camera.centre[1],
        looking: section::perspective::Looking::overview(),
        focus: None,
        followed: None,
        selection: None,
    };
    handle.present(&native.scene, 7, view_a.clone()).unwrap();
    let core = genet_render_host::RenderCore::boot(netrender::NetrenderOptions::default()).unwrap();
    let host = section::host_device::from_renderer(core.renderer());
    let source = Source::new(host, handle.clone());
    let costs = source.costs.clone();
    let mut leaf = SectionProducer {
        handle: handle.clone(),
        render_scale: 1,
        inner: Some(SceneProducer::new(source)),
        costs,
    };
    let request = FrameRequest {
        device: core.device(),
        queue: core.queue(),
        size,
        aspect: camera.aspect,
        color: None,
        needs_frame: true,
        render_scale: 1,
    };
    assert!(
        leaf.inner
            .as_mut()
            .unwrap()
            .render_scene_staged(&request)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        leaf.renders(),
        0,
        "encoding alone advances no accepted generation"
    );
    let prepared_a = leaf.preparation_costs();
    assert!(prepared_a.cost_epoch.is_some());
    assert_eq!(prepared_a.counters["native_stage_calls"], 1);
    assert_eq!(prepared_a.counters["staged_candidates_ready"], 1);
    assert!(
        prepared_a
            .counters
            .get("staged_candidates_published")
            .is_none()
    );
    assert_eq!(prepared_a.counters["fresh_query_snapshots"], 1);
    assert_eq!(prepared_a.counters["target_textures_created"], 1);
    assert_eq!(prepared_a.counters["target_views_created"], 2);
    assert_eq!(prepared_a.counters["target_texels_created"], 192 * 144);
    assert_eq!(prepared_a.last_target_extent, Some(size));
    assert_eq!(prepared_a.stages["resource_setup"].calls, 2);
    assert_eq!(prepared_a.stages["draw_work"].calls, 1);
    assert_eq!(prepared_a.stages["queue_submit"].calls, 1);
    assert_eq!(prepared_a.stages["accepted_query_capture"].calls, 1);

    assert!(leaf.pick((96.0, 72.0), (192.0, 144.0)).is_err());
    leaf.inner.as_mut().unwrap().publish_staged();
    let copied = leaf
        .inner
        .as_ref()
        .unwrap()
        .source()
        .accepted
        .as_ref()
        .unwrap()
        .queries
        .as_ref()
        .unwrap()
        .capture_stats();
    assert!(copied.terrain_payload_copied);
    assert_eq!(prepared_a.counters["query_terrain_maps_copied"], 1);
    assert_eq!(prepared_a.counters["query_terrain_maps_reused"], 0);
    assert_eq!(
        prepared_a.counters["query_placed_records_copied"],
        copied.placed_bodies as u64
    );
    assert_eq!(
        prepared_a.counters["query_surface_records_copied"],
        copied.surfaces as u64
    );
    assert_eq!(
        prepared_a.counters["query_terrain_dense_payload_bytes_copied"],
        (copied.terrain_pointer_words * std::mem::size_of::<u32>() + copied.terrain_atlas_bytes)
            as u64
    );

    assert_eq!(leaf.renders(), 1);
    let a = leaf.inner.as_ref().unwrap().signature().clone();
    let camera_a = leaf.inner.as_ref().unwrap().presented_camera();
    let pick_a = leaf.pick((96.0, 72.0), (192.0, 144.0)).unwrap();
    assert_eq!(pick_a.1, 7);
    let selected = leaf
        .select_part(native.played().unwrap(), None, false)
        .unwrap();
    assert!(leaf.validate_selection(selected));
    let stats_a = leaf.body_stats().unwrap();
    assert_eq!(leaf.cost_status(), (true, true));
    assert_eq!(
        leaf.pick_picture((96.0, 72.0), (192.0, 144.0)).unwrap().2,
        site
    );

    let mut view_b = view_a.clone();
    view_b.half_height *= 1.1;
    view_b.centre[0] += 1.0;
    view_b.focus = Some(selected.organism);
    view_b.selection = Some(selected);
    handle.present(&scene_b, 8, view_b.clone()).unwrap();
    let request_b = FrameRequest {
        size: [128, 96],
        ..request
    };
    assert!(
        leaf.inner
            .as_mut()
            .unwrap()
            .render_scene_staged(&request_b)
            .unwrap()
            .is_some()
    );
    assert_eq!(leaf.renders(), 1);
    assert_eq!(leaf.inner.as_ref().unwrap().signature(), &a);
    assert_eq!(leaf.inner.as_ref().unwrap().presented_camera(), camera_a);
    assert_eq!(
        leaf.pick((96.0, 72.0), (192.0, 144.0)).unwrap(),
        pick_a,
        "B's camera and resize do not mutate A's query geometry or extent"
    );
    assert!(leaf.validate_selection(selected));
    assert_eq!(
        leaf.cost_status(),
        (true, true),
        "accepted A remains eligible while desired B is a different site"
    );
    assert_eq!(
        leaf.pick_picture((96.0, 72.0), (192.0, 144.0)).unwrap().2,
        site,
        "B cannot retarget terrain from the still-displayed A"
    );
    assert_eq!(leaf.body_stats().unwrap().draw_parts, stats_a.draw_parts);
    // A later admission-only publication must not relabel the already prepared B.
    handle.present(&scene_b, 9, view_b.clone()).unwrap();
    assert!(leaf.staged_is_live());
    leaf.inner.as_mut().unwrap().publish_staged();
    assert_eq!(leaf.renders(), 2);
    assert_eq!(
        leaf.pick_picture((64.0, 48.0), (128.0, 96.0)).unwrap().2,
        other
    );
    assert_eq!(leaf.pick((64.0, 48.0), (128.0, 96.0)).unwrap().1, 8);
    let query_b = leaf
        .inner
        .as_ref()
        .unwrap()
        .source()
        .accepted
        .as_ref()
        .unwrap()
        .queries
        .as_ref()
        .unwrap()
        .generation();
    let retained = FrameRequest {
        needs_frame: false,
        ..request_b
    };
    let before_retained = leaf.preparation_costs();
    assert!(
        leaf.inner
            .as_mut()
            .unwrap()
            .render_scene_staged(&retained)
            .unwrap()
            .is_none()
    );
    let retained_work = leaf.preparation_costs();
    assert_eq!(retained_work.cost_epoch, prepared_a.cost_epoch);
    assert_eq!(
        retained_work.counters["fresh_query_snapshots"],
        before_retained.counters["fresh_query_snapshots"]
    );
    assert_eq!(
        retained_work.counters["target_textures_created"],
        before_retained.counters["target_textures_created"]
    );
    assert_eq!(
        retained_work.stages["draw_work"].calls,
        before_retained.stages["draw_work"].calls
    );
    assert_eq!(
        retained_work.stages["accepted_query_capture"].calls,
        before_retained.stages["accepted_query_capture"].calls
    );
    assert_eq!(retained_work.counters["retained_query_arc_clones"], 1);
    assert_eq!(retained_work.stages["retained_query_clone"].calls, 1);
    handle.present(&scene_b, 10, view_b.clone()).unwrap();
    assert_eq!(leaf.pick((64.0, 48.0), (128.0, 96.0)).unwrap().1, 8);
    leaf.inner.as_mut().unwrap().publish_staged();
    assert_eq!(
        leaf.renders(),
        2,
        "retained metadata publication advances no image generation"
    );
    assert_eq!(leaf.pick((64.0, 48.0), (128.0, 96.0)).unwrap().1, 9);
    assert_eq!(
        leaf.inner
            .as_ref()
            .unwrap()
            .source()
            .accepted
            .as_ref()
            .unwrap()
            .queries
            .as_ref()
            .unwrap()
            .generation(),
        query_b
    );

    let mut view_c = view_b.clone();
    view_c.light *= 0.5;
    handle.present(&scene_b, 11, view_c).unwrap();
    assert!(
        leaf.inner
            .as_mut()
            .unwrap()
            .render_scene_staged(&retained)
            .unwrap()
            .is_some()
    );
    let fresh_same_terrain = leaf.preparation_costs();
    assert_eq!(
        fresh_same_terrain.counters["fresh_query_snapshots"],
        retained_work.counters["fresh_query_snapshots"] + 1
    );
    assert_eq!(
        fresh_same_terrain.counters["query_terrain_maps_reused"],
        retained_work.counters["query_terrain_maps_reused"] + 1
    );
    assert_eq!(
        fresh_same_terrain.counters["query_terrain_dense_payload_bytes_copied"],
        retained_work.counters["query_terrain_dense_payload_bytes_copied"]
    );
    leaf.discard_staged();
    assert_eq!(leaf.renders(), 2);
    assert_eq!(
        leaf.pick((64.0, 48.0), (128.0, 96.0)).unwrap().1,
        9,
        "discard keeps accepted B"
    );

    handle.present(&scene_b, 12, view_b.clone()).unwrap();
    leaf.inner
        .as_mut()
        .unwrap()
        .render_scene_staged(&retained)
        .unwrap();
    handle.revoke_source();
    handle
        .present(&Rc::new((*native.scene).clone()), 7, view_b)
        .unwrap();
    assert!(
        !leaf.staged_is_live(),
        "a new owner cannot reuse the pending source lifetime"
    );
    leaf.inner.as_mut().unwrap().publish_staged();
    assert_eq!(leaf.renders(), 2);
    assert!(leaf.pick((64.0, 48.0), (128.0, 96.0)).is_err());
    assert_eq!(
        session.save(),
        saved,
        "derived GPU/query preparation changes no native save or history"
    );
    let replay = cosmograph::Session::load(saved, cosmograph::Execution::Individuals).unwrap();
    assert_eq!(replay.sim.state_hash(), session.sim.state_hash());
}
