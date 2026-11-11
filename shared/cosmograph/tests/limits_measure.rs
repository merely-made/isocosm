// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: the played A1 founding (see `played_world_decay`)
//! with the consumers' limits, over seeds and settings, with and without
//! flat dormancy. Ignored; prints one line per run. In release:
//!
//! `cargo test --release --test limits_measure -- --ignored --nocapture`
//!
//! Knobs: `LIMITS_SEEDS` ("3,5,9,11,17"), `LIMITS_TICKS` (600), `LIMITS_FULLS`
//! ("0,10,12,15"; 0 leaves the fullness off), `LIMITS_CEILINGS` ("0,2,3,4";
//! 0 leaves the ceiling off, n is the ratio n/1), `LIMITS_WORKERS` (8).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Limits, Rates},
    rules::Dormancy,
};
use std::{collections::BTreeMap, sync::Mutex};

fn list(key: &str, default: &str) -> Vec<u64> {
    std::env::var(key).unwrap_or_else(|_| default.into()).split(',').filter_map(|v| v.trim().parse().ok()).collect()
}

fn founding(seed: u64, limits: Option<Limits>, dormant: bool) -> Founding {
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
        dormancy: dormant.then(Dormancy::flat),
        limits,
        ..Founding::default()
    }
}

/// Counts of lineages 0 to 3 at ticks 100, 300 and the end, and the grazer peak.
fn run(f: Founding, ticks: u64) -> (Vec<[u64; 4]>, u64) {
    let mut s = Session::new(f.generate().unwrap(), Execution::Individuals).unwrap();
    let (mut seen, mut peak) = (vec![], 0);
    for t in 1..=ticks {
        s.advance(1).unwrap();
        let mut n = BTreeMap::<u32, u64>::new();
        for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
            if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse().ok()) {
                *n.entry(i).or_default() += g.count;
            }
        }
        let row = [0, 1, 2, 3].map(|i| n.get(&i).copied().unwrap_or(0));
        peak = peak.max(row[1]);
        if t == 100 || t == 300 || t == ticks {
            seen.push(row);
        }
    }
    (seen, peak)
}

#[test]
#[ignore = "a measurement; prints a table"]
fn limits_over_seeds_and_settings() {
    let ticks = list("LIMITS_TICKS", "600")[0];
    let seeds = list("LIMITS_SEEDS", "3,5,9,11,17");
    let mut jobs = vec![];
    for dormant in [false, true] {
        for &full in &list("LIMITS_FULLS", "0,10,12,15") {
            for &ceil in &list("LIMITS_CEILINGS", "0,2,3,4") {
                for &seed in &seeds {
                    jobs.push((dormant, full, ceil, seed));
                }
            }
        }
    }
    let queue = Mutex::new(jobs);
    let workers = list("LIMITS_WORKERS", "8")[0];
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                while let Some((dormant, full, ceil, seed)) = queue.lock().unwrap().pop() {
                    let limits = (full > 0 || ceil > 0).then_some(Limits {
                        full: if full > 0 { full } else { u64::MAX },
                        ceiling: [if ceil > 0 { ceil } else { u64::MAX / 2 }, 1],
                        stocked: None,
                    });
                    let (seen, peak) = run(founding(seed, limits, dormant), ticks);
                    println!("LIMITS dorm={} full={full} ceil={ceil} seed={seed} peak={peak} at100/300/end={seen:?}",
                        u8::from(dormant));
                }
            });
        }
    });
}
