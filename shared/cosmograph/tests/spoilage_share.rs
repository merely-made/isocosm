// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: what the spoilage rule (`Founding.spoilage`)
//! changes in the played A1 world with rot and 12 decomposer bites, as
//! `tests/corpse_share.rs` runs it: seeds 0 to 19, 600 ticks, flat
//! dormancy, default limits, a tick at a time with the flow record handed
//! over. Per arm it prints the lines alive, each lineage's median count at
//! the last tick, the harm events (bites that harmed their eater, from
//! `Simulation::harms`) per site per tick, and the decomposer matter eaten by
//! other lineages. The founded A1 bodies have bulk mouths only, so a world
//! of levels or diets founds no mouth that can bite what it does not admit:
//! harm is nought there unless a lineage is given a jaw. Ignored:
//!
//! `cargo test --release --test spoilage_share -- --ignored --nocapture`
//!
//! Knobs: `SPOIL_SEEDS` ("0-19" or "3,5,9"), `SPOIL_TICKS` (600),
//! `SPOIL_ARMS` (split by ";"; words joined by "+": `plain` for rot and
//! bites only, `spoil` for spoilage as well, `diets` for a web of diets).
//! Default "plain;spoil;plain+diets;spoil+diets".

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{DECOMPOSER_BITES, Limits, Rates, Rot, Spoilage, Web},
    flows::MadeBy,
    rules::Dormancy,
};
use std::sync::{Mutex, atomic::{AtomicUsize, Ordering}};

const DECOMPOSER: &str = "lineage:2";

fn seeds() -> Vec<u64> {
    let text = std::env::var("SPOIL_SEEDS").unwrap_or_else(|_| "0-19".into());
    text.split(',')
        .flat_map(|part| match part.trim().split_once('-') {
            Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
            None => vec![part.trim().parse().unwrap()],
        })
        .collect()
}

fn founding(seed: u64, spoil: bool, diets: bool) -> Founding {
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
        limits: Some(Limits::default()),
        rot: Some(Rot { decomposer_bites: Some(DECOMPOSER_BITES), ..Rot::default() }),
        spoilage: spoil.then(Spoilage::default),
        web: if diets { Web::Diets } else { Web::Levels },
        lineages: if diets { 5 } else { 4 },
        ..Founding::default()
    }
}

struct Run {
    last: Vec<u64>,
    harms: u64,
    eaten: u64,
}

fn run(seed: u64, name: &str, ticks: u64) -> Run {
    let words: Vec<&str> = name.split('+').collect();
    let f = founding(seed, words.contains(&"spoil"), words.contains(&"diets"));
    let lineages = f.lineages as usize;
    let mut s = Session::new(f.generate().unwrap(), Execution::Individuals).unwrap();
    let (mut harms, mut eaten) = (0, 0);
    for _ in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        harms += s.sim.harms().len() as u64;
        for f in r.flows.iter().filter(|f| !f.internal()) {
            let MadeBy::Process(key) = &f.made_by else { continue };
            let eater = key.strip_prefix("ecology:feed-").map(|w| w.split('-').next().unwrap_or(w));
            if eater.is_some_and(|w| w != "2") && f.from_kind.as_ref().is_some_and(|k| k.lineage == DECOMPOSER) {
                eaten += f.amount * f.count;
            }
        }
    }
    let mut last = vec![0; lineages];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) {
            last[i] += g.count;
        }
    }
    Run { last, harms, eaten }
}

fn median(mut v: Vec<u64>) -> f64 {
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] as f64 } else { (v[m - 1] + v[m]) as f64 / 2.0 }
}

#[test]
#[ignore = "a measurement; prints a table"]
fn what_spoilage_changes() {
    let ticks: u64 = std::env::var("SPOIL_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let seeds = seeds();
    let text = std::env::var("SPOIL_ARMS").unwrap_or_else(|_| "plain;spoil;plain+diets;spoil+diets".into());
    let names: Vec<String> = text.split(';').map(|a| a.trim().to_string()).collect();
    let jobs: Vec<(usize, u64)> = (0..names.len()).flat_map(|a| seeds.iter().map(move |&s| (a, s))).collect();
    let next = AtomicUsize::new(0);
    let done: Mutex<std::collections::BTreeMap<(usize, u64), Run>> = Mutex::new(Default::default());
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                let Some(&(a, seed)) = jobs.get(j) else { break };
                let r = run(seed, &names[a], ticks);
                done.lock().unwrap().insert((a, seed), r);
            });
        }
    });
    let done = done.into_inner().unwrap();
    for (a, name) in names.iter().enumerate() {
        let runs: Vec<&Run> = seeds.iter().map(|&s| &done[&(a, s)]).collect();
        let lineages = runs[0].last.len();
        let alive = runs.iter().flat_map(|r| r.last.iter()).filter(|&&n| n > 0).count();
        let med: Vec<f64> = (0..lineages).map(|i| median(runs.iter().map(|r| r.last[i]).collect())).collect();
        let site_ticks = (runs.len() as u64 * ticks * 6) as f64;
        let harms: u64 = runs.iter().map(|r| r.harms).sum();
        let eaten: u64 = runs.iter().map(|r| r.eaten).sum();
        println!("SPOIL arm={name} ticks={ticks} seeds={} lines_alive={alive} of {} median_last={med:?}", runs.len(), runs.len() * lineages);
        println!("SPOIL arm={name} harm_events_per_site_per_tick={:.4} (total {harms}) decomposer_matter_eaten_by_others={:.3} per site per tick (total {eaten})",
            harms as f64 / site_ticks, eaten as f64 / site_ticks);
    }
}
