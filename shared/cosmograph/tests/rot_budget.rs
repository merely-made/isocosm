// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: does rot, set on a founding, keep lines alive?
//! The played A1 founding of `ledger_budget` with flat dormancy (the crowding
//! brake) and the default limits, over seeds and rot arms. Per arm it prints
//! the seeds that lose a lineage by the last tick and which, each lineage's
//! median peak and median count at the last tick, the mean share of the
//! world's matter in soil, the dead, the living and dormant seeds at ticks
//! 300 and 600, whether the grand total never moved, and each seed's lineages
//! alive at the end. Ignored:
//!
//! `cargo test --release --test rot_budget -- --ignored --nocapture`
//!
//! Knobs: `ROT_SEEDS` ("0-19" or "3,5,9"), `ROT_TICKS` (600), `ROT_RATES`
//! ("none;1,200;1,50;1,20;1,5": arms split by ";", "none" or "n,d").

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Limits, Rates, Rot},
    rules::Dormancy,
};
use std::{
    collections::BTreeMap,
    sync::{Mutex, atomic::{AtomicUsize, Ordering}},
};

const LINEAGES: usize = 4;
const MARKS: [u64; 2] = [300, 600];

fn seeds() -> Vec<u64> {
    let text = std::env::var("ROT_SEEDS").unwrap_or_else(|_| "0-19".into());
    text.split(',')
        .flat_map(|part| match part.trim().split_once('-') {
            Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
            None => vec![part.trim().parse().unwrap()],
        })
        .collect()
}

fn founding(seed: u64, rot: Option<Rot>) -> Founding {
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
        rot,
        ..Founding::default()
    }
}

fn arm(text: &str) -> Option<Rot> {
    if text == "none" {
        return None;
    }
    let (n, d) = text.split_once(',').expect("none or n,d");
    Some(Rot { rate: [n.parse().unwrap(), d.parse().unwrap()], ..Rot::default() })
}

/// Members alive of each lineage, seeds included.
fn counts(s: &Session) -> [u64; LINEAGES] {
    let mut n = [0; LINEAGES];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) {
            n[i] += g.count;
        }
    }
    n
}

/// Matter in soil, dead, living, dormant seeds and anything else, in tenths of a percent.
fn shares(s: &Session) -> [f64; 5] {
    let b = s.sim.budget();
    let total = b.total().max(1) as f64;
    let sum = |m: &BTreeMap<String, u128>| m.values().sum::<u128>() as f64;
    let soil = b.sites.get("world:soil").copied().unwrap_or(0) as f64;
    let (dead, living, seeds) = (sum(&b.dead), sum(&b.living), sum(&b.seeds));
    [soil, dead, living, seeds, total - soil - dead - living - seeds].map(|v| v * 100.0 / total)
}

struct Run {
    peak: [u64; LINEAGES],
    last: [u64; LINEAGES],
    shares: Vec<(u64, [f64; 5])>,
    constant: bool,
}

fn run(seed: u64, rot: Option<Rot>, ticks: u64) -> Run {
    let mut s = Session::new(founding(seed, rot).generate().unwrap(), Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let (mut peak, mut constant, mut marks) = ([0; LINEAGES], true, vec![]);
    for t in 1..=ticks {
        s.advance(1).unwrap();
        let n = counts(&s);
        for i in 0..LINEAGES {
            peak[i] = peak[i].max(n[i]);
        }
        if t % 50 == 0 || t == ticks {
            constant &= s.sim.budget().total() == total && s.sim.matter() == total;
        }
        if MARKS.contains(&t) || t == ticks {
            marks.push((t, shares(&s)));
        }
    }
    Run { peak, last: counts(&s), shares: marks, constant }
}

fn median(mut v: Vec<u64>) -> f64 {
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] as f64 } else { (v[m - 1] + v[m]) as f64 / 2.0 }
}

#[test]
#[ignore = "a measurement; prints a table"]
fn rot_over_seeds_and_rates() {
    let ticks: u64 = std::env::var("ROT_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let seeds = seeds();
    let rates = std::env::var("ROT_RATES").unwrap_or_else(|_| "none;1,200;1,50;1,20;1,5".into());
    let arms: Vec<String> = rates.split(';').map(|a| a.trim().to_string()).collect();
    let jobs: Vec<(usize, u64)> = (0..arms.len()).flat_map(|a| seeds.iter().map(move |&s| (a, s))).collect();
    let next = AtomicUsize::new(0);
    let done: Mutex<BTreeMap<(usize, u64), Run>> = Mutex::new(BTreeMap::new());
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                let Some(&(a, seed)) = jobs.get(j) else { break };
                let r = run(seed, arm(&arms[a]), ticks);
                done.lock().unwrap().insert((a, seed), r);
            });
        }
    });
    let done = done.into_inner().unwrap();
    for (a, name) in arms.iter().enumerate() {
        let runs: Vec<(u64, &Run)> = seeds.iter().map(|&s| (s, &done[&(a, s)])).collect();
        let lost: Vec<String> = runs.iter()
            .flat_map(|(s, r)| (0..LINEAGES).filter(|&i| r.last[i] == 0).map(move |i| format!("seed {s} lineage {i}")))
            .collect();
        let lost_seeds = runs.iter().filter(|(_, r)| r.last.contains(&0)).count();
        println!("ROT arm={name} ticks={ticks} seeds={} lost_seeds={lost_seeds} lost=[{}]", seeds.len(), lost.join(", "));
        let med = |f: &dyn Fn(&Run) -> [u64; LINEAGES]| -> Vec<f64> {
            (0..LINEAGES).map(|i| median(runs.iter().map(|(_, r)| f(r)[i]).collect())).collect()
        };
        println!("ROT arm={name} median_peak={:?} median_last={:?}", med(&|r| r.peak), med(&|r| r.last));
        let at: Vec<u64> = runs[0].1.shares.iter().map(|m| m.0).collect();
        for (k, t) in at.iter().enumerate() {
            let mean: Vec<f64> = (0..5).map(|c| runs.iter().map(|(_, r)| r.shares[k].1[c]).sum::<f64>() / runs.len() as f64).collect();
            println!("ROT arm={name} t={t} mean_share% soil={:.1} dead={:.1} living={:.1} dormant={:.1} other={:.1}",
                mean[0], mean[1], mean[2], mean[3], mean[4]);
        }
        println!("ROT arm={name} grand_total_constant={}", runs.iter().all(|(_, r)| r.constant));
        let alive: Vec<String> = runs.iter()
            .map(|(s, r)| format!("{s}:{}", r.last.iter().map(|n| if *n > 0 { 'y' } else { '-' }).collect::<String>()))
            .collect();
        println!("ROT arm={name} alive_at_{ticks} (lineages 0-3, y alive) {}", alive.join(" "));
    }
}
