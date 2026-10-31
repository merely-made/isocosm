// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_runtime::{Founded, Runtime};

fn founded(seed: u64) -> Founded {
    let mut founded = Founded::generated(seed, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = cosmograph::ecology::Rates::A1;
    founded.genesis = domain.generate().unwrap();
    founded.genesis.rules.epoch_ticks = 60;
    founded
}

#[test]
fn placed_play_binds_generator_poses_and_keeps_them_through_save_and_clock() {
    let mut runtime = Runtime::bench_placed(founded(7), 2).unwrap();
    assert!(!runtime.spatial().members.is_empty());
    let templates = runtime
        .founded()
        .genesis
        .rules
        .spatial_templates
        .as_ref()
        .unwrap();
    for at in runtime.spatial().members.values() {
        assert!(
            templates.stands[&at.site.0]
                .iter()
                .any(|stand| stand.at == at.pose.at && stand.yaw == at.pose.yaw)
        );
        at.realized(&runtime.session().sim).unwrap();
    }
    let saved = runtime.save_world().unwrap();
    let mut restored = Runtime::resume_world(saved.clone(), 2, None).unwrap();
    assert_eq!(restored.save_world().unwrap(), saved);
    assert_eq!(restored.spatial(), runtime.spatial());
    assert_eq!(restored.step(1), runtime.step(1));
    assert_eq!(restored.state_hash(), runtime.state_hash());
    assert_eq!(restored.spatial(), runtime.spatial());
    runtime
        .spatial()
        .validate_replay(&runtime.session())
        .unwrap();
}

#[test]
fn placed_play_refuses_missing_templates_and_does_not_invent_water_stands() {
    let mut missing = founded(7);
    missing.genesis.rules.spatial_templates = None;
    assert!(
        Runtime::bench_placed(missing, 2)
            .err()
            .unwrap()
            .contains("physical templates")
    );
    let mut water = founded(2_814_468_213_915_169_548);
    for site in water.genesis.sites.values_mut() {
        for (key, value) in [("terrain:elevation", 0), ("terrain:relief", 0), ("terrain:water", 8)] {
            site.conditions.insert(key.into(), value);
        }
    }
    let flooded = Runtime::bench_placed(water, 2).unwrap();
    let played = flooded.critter().unwrap();
    assert!(!flooded.spatial().members.contains_key(&played));
    assert!(flooded.spatial().driver.is_some());
}

#[test]
fn authored_roster_enters_through_native_development() {
    let mut founded = founded(7);
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.bodies.as_mut().unwrap().roster = true;
    founded.genesis = domain.generate().unwrap();
    let runtime = Runtime::bench_placed(founded, 2).unwrap();
    let sim = runtime.sim();
    assert!(
        sim.genesis()
            .founding
            .as_ref()
            .unwrap()
            .bodies
            .as_ref()
            .unwrap()
            .roster
    );
    assert!(
        sim.state()
            .population
            .groups
            .values()
            .filter_map(|group| group.entity.body.as_ref())
            .any(|body| body.parts.len() > 20)
    );
}
