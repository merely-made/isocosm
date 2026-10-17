// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{HostConfig, State};

#[test]
fn held_view_reuses_its_cut_but_refreshes_after_external_publication() {
    let mut state = State::new(HostConfig {
        bench_world: true, bench_epoch: 1, seed: 9, population: 256,
        ..HostConfig::default()
    });
    state.host.runtime.step(2);
    assert!(state.host.runtime.review().is_some()
        || state.host.runtime.checkpoint().is_some());
    state.refresh();
    let tick = state.host.runtime.tick();
    let revision = state.host.runtime.revision().unwrap();
    assert!(!state.frame_step(), "unchanged held world needs no DOM rebuild");
    state.presented();
    assert_eq!(state.host.frames, 1);
    assert!(!state.frame_step(), "presentation bookkeeping needs no DOM rebuild");
    assert_eq!(state.host.runtime.tick(), tick);
    assert_eq!(state.host.runtime.revision().unwrap(), revision);

    state.host.runtime.world().put_section("held-view-control", vec![1]).unwrap();
    assert!(state.frame_step(), "another endpoint's publication must refresh");
    assert_eq!(state.host.runtime.tick(), tick);
    assert!(!state.frame_step(), "the adopted held cut can be reused");
    state.refresh_required = true;
    assert!(state.frame_step(), "expired selection must still refresh its view");
}
