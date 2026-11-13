// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: who takes the dead, and what happens to the
//! matter after. The played A1 founding of `rot_budget` (flat dormancy, the
//! default limits), over seeds and arms, a tick at a time with the flow
//! record handed over (which changes no outcome). Per arm it prints the dead
//! matter that left corpses, by decomposer meal and by rot, in units per site
//! per tick and as a share; the matter the decomposers' bodies passed on to
//! predators and omnivores; the same by span of 100 ticks; each lineage's
//! median count at the last tick; and the lines lost. Ignored:
//!
//! `cargo test --release --test corpse_share -- --ignored --nocapture`
//!
//! Knobs: `CORPSE_SEEDS` ("0-19" or "3,5,9"), `CORPSE_TICKS` (600),
//! `CORPSE_ARMS` (split by ";"; each arm is words joined by "+": `none` for
//! no rot, `rot` or `rot=n,d` for rot at that rate, `bites=k` for
//! `k` decomposer feedings where there was one, `diets` for a web of diets, whose
//! predator and omnivore eat decomposers, `dc=n,d` for the
//! decomposers' ceiling, `spare` for decomposers without limits). Default "none;rot".

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Limits, Rates, Rot, Web},
    flows::{Flow, MadeBy},
    rules::Dormancy,
};
use std::{
    collections::BTreeMap,
    sync::{Mutex, atomic::{AtomicUsize, Ordering}},
};

const SPAN: u64 = 100;
const DECOMPOSER: &str = "lineage:2";

fn seeds() -> Vec<u64> {
    let text = std::env::var("CORPSE_SEEDS").unwrap_or_else(|_| "0-19".into());
    text.split(',')
        .flat_map(|part| match part.trim().split_once('-') {
            Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
            None => vec![part.trim().parse().unwrap()],
        })
        .collect()
}

/// An arm's words, read once.
#[derive(Clone, Default)]
struct Arm {
    rot: Option<Rot>,
    diets: bool,
    ceiling: Option<[u64; 2]>,
    spare: bool,
}

fn pair(text: &str) -> [u64; 2] {
    let (n, d) = text.split_once(',').expect("n,d");
    [n.parse().unwrap(), d.parse().unwrap()]
}

fn arm(text: &str) -> Arm {
    let mut a = Arm::default();
    for word in text.split('+') {
        match word.split_once('=') {
            None if word == "none" => {},
            None if word == "rot" => a.rot = Some(a.rot.take().unwrap_or_default()),
            None if word == "spare" => a.spare = true,
            None if word == "diets" => a.diets = true,
            Some(("rot", rate)) => a.rot = Some(Rot { rate: pair(rate), ..a.rot.take().unwrap_or_default() }),
            Some(("bites", k)) => a.rot = Some(Rot { decomposer_bites: Some(k.parse().unwrap()), ..a.rot.take().unwrap_or_default() }),
            Some(("dc", rate)) => a.ceiling = Some(pair(rate)),
            _ => panic!("unknown arm word {word}"),
        }
    }
    a
}

fn founding(seed: u64, arm: &Arm) -> Founding {
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
        limits: Some(Limits { decomposer_ceiling: arm.ceiling.or(Limits::default().decomposer_ceiling), spare_decomposers: arm.spare, ..Limits::default() }),
        rot: arm.rot.clone(),
        web: if arm.diets { Web::Diets } else { Web::Levels },
        lineages: if arm.diets { 5 } else { 4 },
        ..Founding::default()
    }
}

fn counts(s: &Session, lineages: usize) -> Vec<u64> {
    let mut n = vec![0; lineages];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) {
            n[i] += g.count;
        }
    }
    n
}

/// Matter moved, by span and by what moved it.
#[derive(Default)]
struct Moves {
    /// Dead matter a decomposer's meal took.
    meals: u64,
    /// Dead matter rot gave to the soil.
    rot: u64,
    /// A decomposer's matter eaten by another lineage.
    eaten: u64,
    /// Every meal of the living by who ate and whose body, `feed-3 of lineage:1`.
    eats: BTreeMap<String, u64>,
    /// Matter lying in corpses when the span ended.
    dead: u128,
}

fn tally(span: &mut Moves, flows: &[Flow]) {
    for f in flows.iter().filter(|f| !f.internal()) {
        let MadeBy::Process(key) = &f.made_by else { continue };
        let n = f.amount * f.count;
        let lineage = f.from_kind.as_ref().map(|k| k.lineage.as_str());
        if key.starts_with("ecology:rot-") {
            span.rot += n;
        } else if let Some(who) = key.strip_prefix("ecology:feed-") {
            let who = who.split('-').next().unwrap_or(who);
            if who == "2" {
                span.meals += n;
            } else {
                if lineage == Some(DECOMPOSER) {
                    span.eaten += n;
                }
                *span.eats.entry(format!("feed-{who} of {}", lineage.unwrap_or("?"))).or_default() += n;
            }
        }
    }
}

struct Run {
    last: Vec<u64>,
    spans: Vec<Moves>,
}

fn run(seed: u64, arm: &Arm, ticks: u64) -> Run {
    let f = founding(seed, arm);
    let lineages = f.lineages as usize;
    let mut s = Session::new(f.generate().unwrap(), Execution::Individuals).unwrap();
    let mut spans = vec![];
    let mut span = Moves::default();
    for t in 1..=ticks {
        let flows = s.advance_tick_with_flows().unwrap().flows;
        tally(&mut span, &flows);
        if t % SPAN == 0 || t == ticks {
            span.dead = s.sim.budget().dead.values().sum();
            spans.push(std::mem::take(&mut span));
        }
    }
    Run { last: counts(&s, lineages), spans }
}

fn median(mut v: Vec<u64>) -> f64 {
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] as f64 } else { (v[m - 1] + v[m]) as f64 / 2.0 }
}

#[test]
#[ignore = "a measurement; prints a table"]
fn who_takes_the_dead() {
    let ticks: u64 = std::env::var("CORPSE_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(600);
    let seeds = seeds();
    let text = std::env::var("CORPSE_ARMS").unwrap_or_else(|_| "none;rot".into());
    let names: Vec<String> = text.split(';').map(|a| a.trim().to_string()).collect();
    let jobs: Vec<(usize, u64)> = (0..names.len()).flat_map(|a| seeds.iter().map(move |&s| (a, s))).collect();
    let next = AtomicUsize::new(0);
    let done: Mutex<BTreeMap<(usize, u64), Run>> = Mutex::new(BTreeMap::new());
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                let Some(&(a, seed)) = jobs.get(j) else { break };
                let r = run(seed, &arm(&names[a]), ticks);
                done.lock().unwrap().insert((a, seed), r);
            });
        }
    });
    let done = done.into_inner().unwrap();
    for (a, name) in names.iter().enumerate() {
        let runs: Vec<(u64, &Run)> = seeds.iter().map(|&s| (s, &done[&(a, s)])).collect();
        let lineages = runs[0].1.last.len();
        let lost: Vec<String> = runs.iter()
            .flat_map(|(s, r)| (0..lineages).filter(|&i| r.last[i] == 0).map(move |i| format!("{s}:{i}")))
            .collect();
        let lost_seeds = runs.iter().filter(|(_, r)| r.last.contains(&0)).count();
        let alive = runs.len() * lineages - lost.len();
        let med: Vec<f64> = (0..lineages).map(|i| median(runs.iter().map(|(_, r)| r.last[i]).collect())).collect();
        println!("CORPSE arm={name} ticks={ticks} seeds={} lines_alive={alive} of {} seeds_losing_a_line={lost_seeds} lost=[{}]",
            runs.len(), runs.len() * lineages, lost.join(", "));
        println!("CORPSE arm={name} median_last={med:?}");
        let sum = |f: &dyn Fn(&Moves) -> u64| -> u64 { runs.iter().flat_map(|(_, r)| r.spans.iter()).map(f).sum() };
        let (meals, rot, eaten) = (sum(&|m| m.meals), sum(&|m| m.rot), sum(&|m| m.eaten));
        let site_ticks = (runs.len() as u64 * ticks * 6) as f64;
        let share = meals as f64 * 100.0 / (meals + rot).max(1) as f64;
        println!("CORPSE arm={name} dead_matter_per_site_per_tick meals={:.3} rot={:.3} decomposer_share={share:.1}% decomposer_matter_eaten={:.3} (units, mean over seeds)",
            meals as f64 / site_ticks, rot as f64 / site_ticks, eaten as f64 / site_ticks);
        let mut by: BTreeMap<String, u64> = BTreeMap::new();
        for m in runs.iter().flat_map(|(_, r)| r.spans.iter()) {
            for (k, v) in &m.eats {
                *by.entry(k.clone()).or_default() += v;
            }
        }
        if !by.is_empty() {
            let per: Vec<String> = by.iter().map(|(k, v)| format!("{k} {:.3}", *v as f64 / site_ticks)).collect();
            println!("CORPSE arm={name} meals_of_the_living (units per site per tick) {}", per.join(", "));
        }
        for k in 0..runs[0].1.spans.len() {
            let mean = |f: &dyn Fn(&Moves) -> u64| runs.iter().map(|(_, r)| f(&r.spans[k]) as f64).sum::<f64>() / runs.len() as f64;
            println!("CORPSE arm={name} span<={} per world: meals {:.0} rot {:.0} corpse->decomposer->eater {:.0}, matter in corpses at span end {:.0}",
                ((k as u64 + 1) * SPAN).min(ticks), mean(&|m| m.meals), mean(&|m| m.rot), mean(&|m| m.eaten), mean(&|m| m.dead as u64));
        }
    }
}
