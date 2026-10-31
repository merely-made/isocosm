// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::founding::{Purpose, found};
use crate::save::WorldSave;
use cosmograph::Founding;

/// The dev bench's own founding inputs (`mesocosm-genet` `app/bench/sim.rs`).
fn bench_genesis() -> cosmograph::simulation::Genesis {
    Founding { seed: 10975940, population: 256, sites: 6, lineages: 4, ecology: false, ..Default::default() }
        .generate().unwrap()
}

#[test]
fn a_head_refuses_a_generated_world_without_its_admitted_witness() {
    let refused = found(bench_genesis(), Execution::Grouped, Purpose::Head, None);
    assert!(refused.err().unwrap().contains("875"), "885: a head waits for the joint witness");
}

#[test]
fn the_bench_founds_a_generated_world_as_its_direct_path_does() {
    let genesis = bench_genesis();
    let mut direct = Session::new(genesis.clone(), Execution::Grouped).unwrap();
    let world = found(genesis, Execution::Grouped, Purpose::Bench, None).unwrap();
    let cut = world.cut().unwrap();
    assert_eq!((cut.tick, cut.state_hash), (direct.sim.state().tick, direct.sim.state_hash()));
    world.advance(5).unwrap();
    direct.advance(5).unwrap();
    assert_eq!(world.cut().unwrap().state_hash, direct.sim.state_hash());
}

#[test]
fn a_genesis_naming_no_generator_needs_no_witness() {
    let mut genesis = bench_genesis();
    genesis.founding = None;
    let world = found(genesis, Execution::Individuals, Purpose::Head, None).unwrap();
    assert_eq!(world.cut().unwrap().tick, 0);
}

#[test]
fn a_founded_world_round_trips_through_its_save() {
    let world = found(bench_genesis(), Execution::Grouped, Purpose::Bench, None).unwrap();
    world.advance(3).unwrap();
    let saved = world.save().unwrap();
    let resumed = World::resume(WorldSave::decode(&saved.encode().unwrap()).unwrap(), Execution::Grouped).unwrap();
    assert_eq!(resumed.cut().unwrap(), world.cut().unwrap());
    assert_eq!(resumed.save().unwrap(), saved);
}
