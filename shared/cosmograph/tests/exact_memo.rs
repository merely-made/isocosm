// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The pass-scoped plant readings and the per-tick sun intensity are an exact
//! CPU optimisation: no state hash moves. Each run folds the state hash after
//! every tick into one history hash, in both executions. The pins were taken
//! from the code before the memo existed (the memo was switched off to take them).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Rates, Web},
    map::{Grid, Layout},
    rules::Dormancy,
};

fn a1(seed: u64) -> cosmograph::simulation::Genesis {
    let grid = Grid { width: 3, height: 2, ..Grid::drawn(seed) };
    Founding {
        seed,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies {
            hazard: cosmograph::harm::hazard::RATE,
            wound: cosmograph::harm::hazard::CELLS,
            ..Bodies::default()
        }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        dormancy: Some(Dormancy::flat()),
        ..Founding::default()
    }
    .generate()
    .unwrap()
}

fn world_time(seed: u64) -> cosmograph::simulation::Genesis {
    let grid = Grid { width: 2, height: 2, ..Grid::drawn(seed) };
    Founding {
        seed,
        sites: 4,
        population: 240,
        cohort_size: 1,
        lineages: 10,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::WorldTime,
        experimental_food_contract: true,
        bodies: Some(Bodies {
            clutch: [1, 1],
            sensing: true,
            grazing: true,
            hazard: [0, 0],
            wound: [0, 0],
            ..Bodies::default()
        }),
        map: Some(Layout::Grid(grid)),
        ..Default::default()
    }
    .generate()
    .unwrap()
}

/// FNV-1a over the state hash after every tick, then the final state hash.
fn run(genesis: cosmograph::simulation::Genesis, mode: Execution, ticks: u64) -> (u64, u64) {
    let mut s = Session::new(genesis, mode).unwrap();
    let mut history = 0xcbf2_9ce4_8422_2325u64;
    for _ in 0..ticks {
        s.advance(1).unwrap();
        for b in s.sim.state_hash().to_le_bytes() {
            history = (history ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
        }
    }
    (s.sim.state_hash(), history)
}

fn check(name: &str, got: (u64, u64), pinned: (u64, u64)) {
    eprintln!("[exact-memo] {name}: state {:#018x} history {:#018x}", got.0, got.1);
    assert_eq!(got, pinned, "{name}");
}

// 2026-11-11, taken with the memo disabled (before the change).
const A1_INDIVIDUALS: (u64, u64) = (0xa855c4f2dabe794c, 0x2c902ffb9edef484);
const A1_GROUPED: (u64, u64) = (0xa855c4f2dabe794c, 0x2c902ffb9edef484);
const WT_INDIVIDUALS: (u64, u64) = (0xe98044de7e465e60, 0xf2afc37f672101ed);
const WT_GROUPED: (u64, u64) = (0xe98044de7e465e60, 0xf2afc37f672101ed);

#[test]
fn the_a1_played_world_is_exact_individuals() {
    check("a1 individuals", run(a1(9), Execution::Individuals, 300), A1_INDIVIDUALS);
}
#[test]
fn the_a1_played_world_is_exact_grouped() {
    check("a1 grouped", run(a1(9), Execution::Grouped, 300), A1_GROUPED);
}
#[test]
fn the_world_time_sensing_world_is_exact_individuals() {
    check("wt individuals", run(world_time(9), Execution::Individuals, 5000), WT_INDIVIDUALS);
}
#[test]
fn the_world_time_sensing_world_is_exact_grouped() {
    check("wt grouped", run(world_time(9), Execution::Grouped, 5000), WT_GROUPED);
}
