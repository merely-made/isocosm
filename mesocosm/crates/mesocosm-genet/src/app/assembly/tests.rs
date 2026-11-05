// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{HostConfig, State};
use std::rc::Rc;

#[test]
fn starting_paused_excludes_window_warmup_from_the_native_clock() {
    let mut state = State::new(HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        dev: true,
        start_paused: true,
        ..HostConfig::default()
    });
    state.refresh();
    let before = state.host.runtime.world().save().unwrap().native;
    state.host.last = Some(std::time::Instant::now() - std::time::Duration::from_secs(10));
    state.frame_step();
    assert!(state.host.dev_paused);
    assert_eq!(state.host.steps, 0);
    assert_eq!(state.host.runtime.tick(), 0);
    assert_eq!(state.host.runtime.world().save().unwrap().native, before);
}

#[test]
fn held_view_reuses_its_cut_but_refreshes_after_external_publication() {
    let mut state = State::new(HostConfig {
        bench_world: true,
        bench_epoch: 1,
        seed: 9,
        population: 256,
        ..HostConfig::default()
    });
    state.host.runtime.step(2);
    assert!(state.host.runtime.review().is_some() || state.host.runtime.checkpoint().is_some());
    state.refresh();
    let tick = state.host.runtime.tick();
    let revision = state.host.runtime.revision().unwrap();
    assert!(
        !state.frame_step(),
        "unchanged held world needs no DOM rebuild"
    );
    let stamp = super::presentation::tests::stamp(1);
    assert!(state.prepare_publication(stamp).publish(stamp));
    state.presented();
    assert_eq!(state.host.frames, 1);
    assert!(
        !state.frame_step(),
        "presentation bookkeeping needs no DOM rebuild"
    );
    assert_eq!(state.host.runtime.tick(), tick);
    assert_eq!(state.host.runtime.revision().unwrap(), revision);

    state
        .host
        .runtime
        .world()
        .put_section("held-view-control", vec![1])
        .unwrap();
    assert!(
        state.frame_step(),
        "another endpoint's publication must refresh"
    );
    assert_eq!(state.host.runtime.tick(), tick);
    assert!(!state.frame_step(), "the adopted held cut can be reused");
    state.refresh_required = true;
    assert!(
        state.frame_step(),
        "expired selection must still refresh its view"
    );
}

#[test]
fn coherent_dom_and_picture_retain_inputs_across_opaque_publication() {
    let mut state = State::new(HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        dev: true,
        ..HostConfig::default()
    });
    state.host.inspection.open = true;
    state.refresh();
    let first = state.picture.snapshot().unwrap();
    let native = state.committed.as_ref().unwrap().native.clone();
    assert_eq!(
        state.dev.inspection,
        state.committed.as_ref().unwrap().inspection
    );
    assert_eq!(state.tick_reading, state.host.runtime.tick());
    assert_eq!(state.dev.tick, state.tick_reading);
    let saved = state.host.runtime.world().save().unwrap();
    let extractions = state.host.scene_extractions;
    state
        .host
        .runtime
        .world()
        .put_section("coherent-reading-control", vec![1])
        .unwrap();
    state.refresh();
    let next = state.picture.snapshot().unwrap();
    let reading = state.committed.as_ref().unwrap();
    assert_eq!(reading.native, native);
    assert_eq!(reading.revision, state.host.runtime.revision().unwrap());
    assert_eq!(next.revision, Some(reading.revision));
    assert_eq!(
        next.presentation.as_ref().unwrap().selection,
        reading.selection
    );
    assert_eq!(state.dev.inspection, reading.inspection);
    assert_eq!(next.fingerprint, first.fingerprint);
    assert!(Rc::ptr_eq(&first.scene, &next.scene));
    assert_eq!(state.host.scene_extractions, extractions);
    state.host.pan.x += 1.0;
    state.refresh();
    let moved = state.picture.snapshot().unwrap();
    assert_ne!(moved.fingerprint, next.fingerprint);
    assert!(Rc::ptr_eq(&next.scene, &moved.scene));
    assert_eq!(
        state.host.runtime.world().save().unwrap().native,
        saved.native,
        "camera, inspector and section-only work leave native history intact"
    );
}

#[test]
fn a_fresh_held_owner_with_reused_numeric_revision_refreshes_and_ends_the_old_lifetime() {
    let config = HostConfig {
        bench_world: true,
        bench_epoch: 1,
        seed: 9,
        population: 256,
        ..HostConfig::default()
    };
    let mut first = State::new(config.clone());
    let mut replacement = State::new(config);
    first.host.runtime.step(2);
    replacement.host.runtime.step(2);
    first.refresh();
    let prior = first.committed.as_ref().unwrap().native.clone();
    let picture = first.picture.snapshot().unwrap();
    // Synthetic old route: this test checks owner revocation, not geometry.
    first.host.inspection.selected = Some(crate::section::BodySelection {
        organism: first.host.runtime.critter().unwrap(),
        part: cosmograph::schema::PartId(0),
        revision: isometer::mesh::BodyDependencyRevision(1),
    });
    assert_eq!(
        first.host.runtime.revision().unwrap(),
        replacement.host.runtime.revision().unwrap()
    );
    assert!(
        replacement.host.runtime.review().is_some()
            || replacement.host.runtime.checkpoint().is_some()
    );
    first.host.runtime = replacement.host.runtime;
    assert!(
        first.frame_step(),
        "same counter in a fresh owner must refresh the held view"
    );
    assert!(!prior.same_source(&first.committed.as_ref().unwrap().native));
    assert_eq!(first.source_lifetime, 1);
    assert!(first.host.inspection.selected.is_none());
    assert_ne!(
        first.picture.snapshot().unwrap().fingerprint,
        picture.fingerprint
    );
    assert!(!Rc::ptr_eq(
        &picture.scene,
        &first.picture.snapshot().unwrap().scene
    ));
    assert!(!first.frame_step());
}

#[test]
fn a_direct_camera_refresh_adopts_another_endpoints_relevant_section_first() {
    use mesocosm_runtime::{
        Runtime,
        spatial::{Setup, SightConfig, SightedSetup},
    };
    let mut state = State::new(HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        dev: true,
        ..HostConfig::default()
    });
    state.host.runtime = Runtime::bench_placed(
        mesocosm_runtime::Founded::generated(16_003_851_315_348_448_882, 60).unwrap(),
        10,
    )
    .unwrap();
    state.refresh();
    let before = state.picture.snapshot().unwrap();
    let native = state.committed.as_ref().unwrap().native.clone();
    let saved = state.host.runtime.save_world().unwrap();
    let mut peer = Runtime::resume_world(saved.clone(), 10, None).unwrap();
    let observer = *peer
        .spatial()
        .members
        .keys()
        .next()
        .expect("real physical observer");
    let sight = SightConfig {
        version: 1,
        ranges: [(observer, 1)].into(),
    };
    peer.setup_spatial(&Setup::Sighted(SightedSetup {
        version: 1,
        placements: vec![],
        profile: None,
        sight: sight.clone(),
    }))
    .unwrap();
    let peer_saved = peer.save_world().unwrap();
    assert_eq!(
        peer_saved.native, saved.native,
        "setup changes only product data"
    );
    assert_ne!(state.host.runtime.spatial().sight.as_ref(), Some(&sight));
    state
        .host
        .runtime
        .world()
        .put_section(
            mesocosm_runtime::runtime::MODE,
            peer_saved.sections[mesocosm_runtime::runtime::MODE].clone(),
        )
        .unwrap();
    state.host.pan.x += 1.0;
    state.refresh();
    assert!(state.error.is_none(), "{:?}", state.error);
    assert_eq!(state.host.runtime.spatial().sight, Some(sight));
    let next = state.picture.snapshot().unwrap();
    assert!(!Rc::ptr_eq(&before.scene, &next.scene));
    assert_eq!(state.committed.as_ref().unwrap().native, native);
    assert_eq!(next.revision, state.host.runtime.revision().ok());
}

#[test]
fn a_malformed_current_product_cannot_retag_the_previous_picture() {
    let mut state = State::new(HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        ..HostConfig::default()
    });
    state.refresh();
    assert!(state.picture.snapshot().is_some());
    state
        .host
        .runtime
        .world()
        .put_section(
            mesocosm_runtime::runtime::MODE,
            b"not a saved product".to_vec(),
        )
        .unwrap();
    state.refresh();
    assert!(state.picture.snapshot().is_none());
    assert!(
        state
            .error
            .as_ref()
            .unwrap()
            .contains("could not be adopted")
    );
    assert!(state.refresh_required);
}
