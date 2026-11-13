// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: what food categories (`Founding.categories`,
//! tough plant matter and dead matter in classes, lineage-rounds D37) change
//! in the five-lineage diets web and the played four-line world, with rot,
//! 12 decomposer bites, spoilage and the repertoire, as `tests/
//! repertoire_share.rs` runs them (flat dormancy, default limits, a tick at a
//! time with the flow record handed over; every `SHARE_SPAN` ticks the
//! unplayed lines weigh their candidates and drift moves their templates, for
//! `SHARE_BOUNDARIES` boundaries and one more span after). Per arm it prints
//! the lines alive and each lineage's median count at the last tick, how many
//! lines carry a catalog part at the end and which, the harm events by
//! category (per site per tick), the matter eaten by class (per site per
//! tick), and how much tough matter, living or dead, each lineage's mouths
//! ate. Ignored:
//!
//! `cargo test --release --test categories_share -- --ignored --nocapture`
//!
//! Knobs: `SHARE_SEEDS` ("0-19" or "3,5,9"), `SHARE_SPAN` (100),
//! `SHARE_BOUNDARIES` (4), `SHARE_SCORE` (15), `SHARE_ARMS` (split by ";";
//! words joined by "+": `base` for rot, bites, spoilage and the repertoire,
//! `cats` for the categories rule as well, `diets` for the five-lineage web,
//! otherwise the played four-line world; default "base;cats;base+diets;
//! cats+diets"), `SHARE_WOOD` (the rule's `wood_rot_every`, default 1).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Categories, DECOMPOSER_BITES, Limits, Rates, Rot, Spoilage, Web, tough},
    flows::{Holder, MadeBy},
    lineage::{boundary::{self, Candidate, Score, ScoringSource}, revise},
    repertoire::Repertoire,
    rules::{Directing, Dormancy},
    schema::Tick,
};
use std::{
    collections::BTreeMap,
    sync::{Mutex, atomic::{AtomicUsize, Ordering}},
};


fn knob(name: &str, default: u64) -> u64 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn seeds() -> Vec<u64> {
    let text = std::env::var("SHARE_SEEDS").unwrap_or_else(|_| "0-19".into());
    text.split(',')
        .flat_map(|part| match part.trim().split_once('-') {
            Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
            None => vec![part.trim().parse().unwrap()],
        })
        .collect()
}

fn founding(seed: u64, categories: bool, diets: bool) -> Founding {
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
        spoilage: Some(Spoilage::default()),
        repertoire: Some(Repertoire::default()),
        categories: categories.then(|| Categories { wood_rot_every: knob("SHARE_WOOD", 1), ..Categories::default() }),
        web: if diets { Web::Diets } else { Web::Levels },
        lineages: if diets { 5 } else { 4 },
        ..Founding::default()
    }
}

#[derive(Default)]
struct Run {
    last: Vec<u64>,
    /// Harm events by category.
    harms: BTreeMap<&'static str, u64>,
    /// Matter eaten, by class of what it was.
    classes: BTreeMap<&'static str, u64>,
    /// Tough matter (living or dead) eaten, by eating lineage.
    grinders: BTreeMap<String, u64>,
    /// Lines carrying a catalog part at the end, by part.
    parts: BTreeMap<String, u64>,
    carrying: u64,
    turns: u64,
    committed: u64,
}

fn run(seed: u64, name: &str, span: Tick, boundaries: u64, scoring: Tick) -> Run {
    let words: Vec<&str> = name.split('+').collect();
    let f = founding(seed, words.contains(&"cats"), words.contains(&"diets"));
    let lineages = f.lineages as usize;
    let mut g = f.generate().unwrap();
    g.rules.directing.get_or_insert_with(Directing::default).drift = true;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let mut out = Run::default();
    let mut source: Option<ScoringSource> = None;
    for b in 0..=boundaries {
        for _ in 0..span {
            let r = s.advance_tick_with_flows().unwrap();
            for h in s.sim.harms() {
                let category = match (h.tough, h.flesh) {
                    (true, _) => "tough",
                    (_, cosmograph::ecology::Flesh::Fresh) => "living-decomposer",
                    (_, cosmograph::ecology::Flesh::Aged) => "aged-flesh",
                    (_, cosmograph::ecology::Flesh::Toxic) => "toxic-flesh",
                };
                *out.harms.entry(category).or_default() += 1;
            }
            for f in r.flows.iter().filter(|f| !f.internal()) {
                let MadeBy::Process(key) = &f.made_by else { continue };
                let Some(eater) = key.strip_prefix("ecology:feed-").map(|w| w.split('-').next().unwrap_or(w)) else { continue };
                if !matches!(f.to.0, Holder::Entity(_) | Holder::Part(..)) || !f.from.1.starts_with("matter:") {
                    continue;
                }
                let (Some(kind), Some(body)) = (&f.from_kind, f.from.0.body()) else { continue };
                let Some(prey) = s.sim.state().population.get(body) else { continue };
                let hard = matches!(f.from.0, Holder::Part(_, part) if tough(prey, part));
                let class = match (kind.kingdom.as_str(), prey.alive, hard) {
                    ("kingdom:flora", true, false) => "live-soft-plant",
                    ("kingdom:flora", true, true) => "live-tough-plant",
                    ("kingdom:flora", false, false) => "dead-litter",
                    ("kingdom:flora", false, true) => "dead-wood",
                    (_, true, _) => "live-flesh",
                    (_, false, _) => "dead-carrion",
                };
                let n = f.amount * f.count;
                *out.classes.entry(class).or_default() += n;
                if matches!(class, "live-tough-plant" | "dead-wood") {
                    *out.grinders.entry(format!("lineage:{eater}")).or_default() += n;
                }
            }
        }
        if b == boundaries {
            break;
        }
        let since = s.sim.state().tick.saturating_sub(span);
        let mut score = |at: &Session, lineage: &str, c: &Candidate, ticks: Tick| -> Result<Score, String> {
            if !source.as_ref().is_some_and(|x| x.matches(at)) {
                source = Some(ScoringSource::new(at)?);
            }
            source.as_ref().unwrap().score_using(lineage, c, ticks, &mut |copy, n| copy.advance(n).map(|_| ()))
        };
        let turns = boundary::adapt_since_using(&mut s, Some("lineage:1"), since, &revise::revisions, scoring, &mut score).unwrap();
        if std::env::var_os("SHARE_VERBOSE").is_some() {
            for t in &turns {
                println!("SHARE seed={seed} b={b} {} committed={:?} {:?}", t.lineage, t.committed,
                    t.considered.iter().map(|(n, c)| (n.replace("revision:", ""), c.held, c.members)).collect::<Vec<_>>());
            }
        }
        out.turns += turns.len() as u64;
        out.committed += turns.iter().filter(|t| t.committed.as_ref().is_some_and(|n| n.contains("repertoire-"))).count() as u64;
    }
    let mut last = vec![0; lineages];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) {
            last[i] += g.count;
        }
    }
    out.last = last;
    let lowered = s.sim.genesis().rules.repertoire.clone();
    for l in s.sim.state().lineages.values() {
        let Some(d) = &l.development else { continue };
        let named = d.recipe.kinds();
        let parts: Vec<_> = lowered.iter().flat_map(|r| r.kinds.iter())
            .filter(|(k, _)| named.contains(*k)).map(|(_, e)| e.part.clone()).collect();
        out.carrying += u64::from(!parts.is_empty());
        for p in parts {
            *out.parts.entry(p).or_default() += 1;
        }
    }
    out
}

fn median(mut v: Vec<u64>) -> f64 {
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] as f64 } else { (v[m - 1] + v[m]) as f64 / 2.0 }
}

#[test]
#[ignore = "a measurement; prints a table"]
fn what_the_categories_change() {
    let (span, boundaries, scoring) = (knob("SHARE_SPAN", 100), knob("SHARE_BOUNDARIES", 4), knob("SHARE_SCORE", 15));
    let ticks = span * (boundaries + 1);
    let seeds = seeds();
    let text = std::env::var("SHARE_ARMS").unwrap_or_else(|_| "base;cats;base+diets;cats+diets".into());
    let names: Vec<String> = text.split(';').map(|a| a.trim().to_string()).collect();
    let jobs: Vec<(usize, u64)> = (0..names.len()).flat_map(|a| seeds.iter().map(move |&s| (a, s))).collect();
    let next = AtomicUsize::new(0);
    let done: Mutex<BTreeMap<(usize, u64), Run>> = Mutex::new(Default::default());
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                let Some(&(a, seed)) = jobs.get(j) else { break };
                let r = run(seed, &names[a], span, boundaries, scoring);
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
        let merged = |pick: fn(&Run) -> &BTreeMap<&'static str, u64>| {
            let mut all: BTreeMap<&str, f64> = BTreeMap::new();
            for r in &runs {
                for (k, n) in pick(r) {
                    *all.entry(k).or_default() += *n as f64 / site_ticks;
                }
            }
            all.into_iter().map(|(k, n)| format!("{k}={n:.4}")).collect::<Vec<_>>().join(" ")
        };
        let mut grinders: BTreeMap<&str, f64> = BTreeMap::new();
        for r in &runs {
            for (k, n) in &r.grinders {
                *grinders.entry(k).or_default() += *n as f64 / site_ticks;
            }
        }
        let carrying: u64 = runs.iter().map(|r| r.carrying).sum();
        let mut parts: BTreeMap<&str, u64> = BTreeMap::new();
        for r in &runs {
            for (p, n) in &r.parts {
                *parts.entry(p).or_default() += n;
            }
        }
        let (turns, committed): (u64, u64) = (runs.iter().map(|r| r.turns).sum(), runs.iter().map(|r| r.committed).sum());
        println!("SHARE arm={name} ticks={ticks} boundaries={boundaries} span={span} scoring={scoring} seeds={} lines_alive={alive} of {} median_last={med:?}", runs.len(), runs.len() * lineages);
        println!("SHARE arm={name} lines_carrying_a_catalog_part={carrying} boundary_turns={turns} turns_committing_one={committed} parts={parts:?}");
        println!("SHARE arm={name} harm_events_per_site_per_tick: {}", merged(|r| &r.harms));
        println!("SHARE arm={name} matter_eaten_by_class_per_site_per_tick: {}", merged(|r| &r.classes));
        println!("SHARE arm={name} tough_matter_eaten_by_lineage_per_site_per_tick: {grinders:?}");
    }
}
