// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate (ruling D27): the played A1 founding of
//! `limits_measure` with flat dormancy and the default limits, the
//! decomposers spared, limited alike, or limited by a ceiling of their own.
//! Prints per seed the four lineages' counts at ticks 100, 200, 300, 450 and
//! 600, and each lineage's peak and minimum after tick 100. Ignored:
//!
//! `cargo test --release --test limits_decomposers -- --ignored --nocapture`
//!
//! Knobs: `LIMITS_SEEDS` ("3,5,9,11,17"), `LIMITS_TICKS` (600),
//! `LIMITS_DECOMPOSERS` ("none", "spare", "same" or "n,d", split by ";"; spare and same by default).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Limits, Rates},
    rules::Dormancy,
};
use std::collections::BTreeMap;

fn founding(seed: u64, limits: Option<Limits>) -> Founding {
    let grid = cosmograph::map::Grid { width: 3, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    Founding {
        seed,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies {
            hazard: cosmograph::harm::hazard::RATE,
            wound: cosmograph::harm::hazard::CELLS,
            ..Bodies::default()
        }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        dormancy: Some(Dormancy::flat()),
        limits,
        ..Founding::default()
    }
}

fn limits(mode: &str) -> Option<Limits> {
    Some(match mode {
        "none" => return None,
        "spare" => Limits { spare_decomposers: true, ..Limits::default() },
        "same" => Limits::default(),
        pair => {
            let (n, d) = pair.split_once(',').expect("none, spare, same or n,d");
            Limits { decomposer_ceiling: Some([n.parse().unwrap(), d.parse().unwrap()]), ..Limits::default() }
        },
    })
}

#[test]
#[ignore = "a measurement; prints a table"]
fn decomposer_limits_over_seeds() {
    let ticks: u64 = std::env::var("LIMITS_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let seeds: Vec<u64> = std::env::var("LIMITS_SEEDS").unwrap_or_else(|_| "3,5,9,11,17".into())
        .split(',').filter_map(|v| v.trim().parse().ok()).collect();
    let modes = std::env::var("LIMITS_DECOMPOSERS").unwrap_or_else(|_| "spare;same".into());
    let jobs: Vec<(String, u64)> = modes.split(';').flat_map(|m| seeds.iter().map(move |&s| (m.to_string(), s))).collect();
    std::thread::scope(|scope| {
        for (mode, seed) in jobs {
            scope.spawn(move || {
                let mut s = Session::new(founding(seed, limits(&mode)).generate().unwrap(), Execution::Individuals).unwrap();
                let (mut at, mut peak, mut low) = (vec![], [0u64; 4], [u64::MAX; 4]);
                for t in 1..=ticks {
                    s.advance(1).unwrap();
                    let mut n = BTreeMap::<u32, u64>::new();
                    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
                        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse().ok()) {
                            *n.entry(i).or_default() += g.count;
                        }
                    }
                    let row = [0, 1, 2, 3].map(|i| n.get(&i).copied().unwrap_or(0));
                    if t > 100 {
                        for k in 0..4 {
                            peak[k] = peak[k].max(row[k]);
                            low[k] = low[k].min(row[k]);
                        }
                    }
                    if [100, 200, 300, 450, 600].contains(&t) {
                        at.push((t, row));
                    }
                }
                println!("DECOMP mode={mode} seed={seed} at={at:?} peak={peak:?} min={low:?}");
            });
        }
    });
}
