// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::spatial::embodiment::tests::terrain;
use crate::{Command, Execution, Session};

fn input(sim: &Simulation) -> Input {
    search::turn_tests::control_input(sim)
}

#[test]
fn distant_movement_reuses_the_route_but_a_local_blocker_invalidates_it() {
    let sim = terrain();
    let cache = Cache::default();
    let initial = input(&sim);
    let reference = search::find(&initial).unwrap();
    assert!(reference.route.is_some());
    for x in [430, 431] {
        let mut far = initial.clone();
        far.blocked.push([[x, 0, 380], [x + 1, 64, 381]]);
        assert_eq!(search::find(&far).unwrap(), reference);
        assert_eq!(cache.find(far).unwrap(), reference);
        assert_eq!(
            cache.0.lock().unwrap().len(),
            1,
            "distant changes keep the exact local entry"
        );
    }
    let mut local = initial.clone();
    local.blocked.push([[360, 0, 360], [408, 64, 408]]);
    let blocked = cache.find(local.clone()).unwrap();
    assert_eq!(blocked, search::find(&local).unwrap());
    assert!(blocked.route.is_none());
    assert_eq!(cache.0.lock().unwrap().len(), 2);

    // A box crossing the window border must remain a blocker, not just boxes
    // with their origins inside. Keep the full box for the native checks.
    let expected = initial.blocked.len() + 1;
    let mut crossing = initial;
    crossing.blocked.push([[349, 0, 350], [419, 64, 418]]);
    assert!(cache.find(crossing).unwrap().route.is_none());
    assert_eq!(
        cache.0.lock().unwrap().last().unwrap().input.blocked.len(),
        expected
    );
}

#[test]
fn identical_rederived_terrain_reuses_a_route_but_window_changes_do_not() {
    let sim = terrain();
    let initial = input(&sim);
    let cache = Cache::default();
    let reference = cache.find(initial.clone()).unwrap();
    let fresh = Simulation::new(sim.genesis().clone(), Execution::Individuals).unwrap();
    let same = input(&fresh);
    assert!(!Arc::ptr_eq(&initial.terrain, &same.terrain));
    assert_eq!(cache.find(same).unwrap(), reference);
    assert_eq!(cache.0.lock().unwrap().len(), 1);
    let mut different = initial;
    different.terrain = sim
        .site_geometry(
            0,
            &Window {
                min: [359; 2],
                max: [409; 2],
                depth: 16,
            },
            Default::default(),
        )
        .unwrap();
    cache.find(different).unwrap();
    assert_eq!(cache.0.lock().unwrap().len(), 2);
}

#[test]
fn authenticated_prefixes_and_forks_share_the_existing_derivative_caches() {
    let sim = terrain();
    let mut session = Session::new(sim.genesis().clone(), Execution::Individuals).unwrap();
    let initial = input(&session.sim);
    let held = initial.terrain.clone();
    session.sim.foraging_routes.find(initial).unwrap();
    session.command(Command::Collect).unwrap();
    let key = &session.entries[0].id;
    let before = session.before_entry(key).unwrap();
    let prefix = session.prefix(1, 0).unwrap();
    let fork = session.fork_at(0, "branch:cache-control".into()).unwrap();
    for read in [before, prefix, fork] {
        assert!(Arc::ptr_eq(
            &session.sim.foraging_routes.0,
            &read.sim.foraging_routes.0
        ));
        assert!(Arc::ptr_eq(&held, &input(&read.sim).terrain));
    }
    let replay = Session::load(session.save(), Execution::Individuals).unwrap();
    assert_eq!(
        replay.save(),
        session.save(),
        "reuse changes no authenticated history"
    );
}
