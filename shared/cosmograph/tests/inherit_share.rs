// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: what young growing from the line's recipe
//! (`Founding.inherit`, lineage-rounds D40) changes in the played A1 world
//! with the Mesocosm generated rules (flat dormancy, default limits, rot with
//! 12 decomposer bites), run as `tests/repertoire_share.rs` runs it: every
//! `SHARE_SPAN` ticks the unplayed lines weigh their candidates by growing a
//! copy `SHARE_SCORE` ticks (the played line takes no turn) and drift moves
//! their templates, for `SHARE_BOUNDARIES` boundaries and one more span after.
//! Per arm it prints the lines alive, each lineage's median count, how many
//! lines took a revision (a committed turn that was not the status quo), and
//! how many living members at the end carry their line's current recipe body
//! against an older one (a body is current when its parts, with their cells,
//! are those the recipe's own segments grow). Ignored:
//!
//! `cargo test --release --test inherit_share -- --ignored --nocapture`
//!
//! Knobs: `SHARE_SEEDS` ("0-19" or "3,5,9"), `SHARE_SPAN` (100),
//! `SHARE_BOUNDARIES` (4), `SHARE_SCORE` (15), `SHARE_ARMS` (split by ";";
//! `copy` for the world as it was, `inherit` for the rule; words joined by "+":
//! `diets` for a web of diets). Default "copy;inherit;copy+diets;inherit+diets".

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    development::{Soma, develop},
    directing::Played,
    ecology::{DECOMPOSER_BITES, Limits, Rates, Rot, Web},
    lineage::{boundary::{self, Candidate, Score, ScoringSource}, revise},
    rules::{Directing, Dormancy},
    schema::Tick,
};
use std::{
    collections::{BTreeMap, BTreeSet},
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

fn founding(seed: u64, inherit: bool, diets: bool) -> Founding {
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
        inherit,
        web: if diets { Web::Diets } else { Web::Levels },
        lineages: if diets { 5 } else { 4 },
        ..Founding::default()
    }
}

#[derive(Default)]
struct Run {
    last: Vec<u64>,
    turns: u64,
    /// Lines that committed a revision at some boundary.
    revised: BTreeSet<String>,
    /// Lines whose recipe at the end is not the one they were founded with (a revision or drift).
    moved: u64,
    current: u64,
    older: u64,
    /// Segment counts of the members at the end, first tagma, by member.
    spread: Vec<u8>,
}

fn run(seed: u64, name: &str, span: Tick, boundaries: u64, scoring: Tick) -> Run {
    let words: Vec<&str> = name.split('+').collect();
    let f = founding(seed, words.contains(&"inherit"), words.contains(&"diets"));
    let lineages = f.lineages as usize;
    let mut g = f.generate().unwrap();
    let founded: BTreeMap<String, _> = g.lineages.iter().filter_map(|(k, l)| Some((k.clone(), l.development.as_ref()?.recipe.clone()))).collect();
    g.rules.directing.get_or_insert_with(Directing::default).drift = true;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let mut out = Run::default();
    let mut source: Option<ScoringSource> = None;
    for b in 0..=boundaries {
        s.advance(span).unwrap();
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
        out.turns += turns.len() as u64;
        for t in &turns {
            if t.committed.as_ref().is_some_and(|n| n != "candidate:stay") {
                out.revised.insert(t.lineage.clone());
            }
        }
    }
    let rules = s.sim.genesis().rules.clone();
    out.moved = s.sim.state().lineages.iter()
        .filter(|(k, l)| l.development.as_ref().is_some_and(|d| founded.get(*k).is_some_and(|r| *r != d.recipe))).count() as u64;
    // What each line's recipe grows, as its parts and their cells.
    let current: BTreeMap<String, Vec<BTreeMap<String, u32>>> = s.sim.state().lineages.iter()
        .filter_map(|(k, l)| {
            let d = l.development.as_ref()?;
            let soma = Soma { segments: d.recipe.tagmata.iter().map(|t| t.segments).collect(), absent: vec![], seed: 0 };
            let body = develop(&rules, d, &soma).ok()?;
            Some((k.clone(), body.parts.iter().map(|(_, p)| p.cells.clone()).collect()))
        }).collect();
    let mut last = vec![0; lineages];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) {
            last[i] += g.count;
        }
        for _ in 0..g.count.min(50) { out.spread.extend(g.entity.soma.first().copied()); }
        if let Some(want) = current.get(&g.entity.lineage) {
            let have: Vec<BTreeMap<String, u32>> = g.entity.parts.values().map(|p| p.cells.clone()).collect();
            if &have == want { out.current += g.count } else { out.older += g.count }
        }
    }
    out.last = last;
    out
}

fn median(mut v: Vec<u64>) -> f64 {
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] as f64 } else { (v[m - 1] + v[m]) as f64 / 2.0 }
}

#[test]
#[ignore = "a measurement; prints a table"]
fn what_inheriting_the_recipe_changes() {
    let (span, boundaries, scoring) = (knob("SHARE_SPAN", 100), knob("SHARE_BOUNDARIES", 4), knob("SHARE_SCORE", 15));
    let ticks = span * (boundaries + 1);
    let seeds = seeds();
    let text = std::env::var("SHARE_ARMS").unwrap_or_else(|_| "copy;inherit;copy+diets;inherit+diets".into());
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
        let (turns, revised): (u64, usize) = (runs.iter().map(|r| r.turns).sum(), runs.iter().map(|r| r.revised.len()).sum());
        let moved: u64 = runs.iter().map(|r| r.moved).sum();
        let (current, older): (u64, u64) = (runs.iter().map(|r| r.current).sum(), runs.iter().map(|r| r.older).sum());
        let mut seg: Vec<u8> = runs.iter().flat_map(|r| r.spread.iter().copied()).collect();
        seg.sort_unstable();
        let mean = seg.iter().map(|&n| f64::from(n)).sum::<f64>() / seg.len().max(1) as f64;
        let sd = (seg.iter().map(|&n| (f64::from(n) - mean).powi(2)).sum::<f64>() / seg.len().max(1) as f64).sqrt();
        let distinct = seg.iter().collect::<BTreeSet<_>>().len();
        println!("SHARE arm={name} first_tagma_segments: members_sampled={} distinct_counts={distinct} mean={mean:.2} sd={sd:.3}", seg.len());
        println!("SHARE arm={name} ticks={ticks} boundaries={boundaries} span={span} scoring={scoring} seeds={} lines_alive={alive} of {} median_last={med:?}", runs.len(), runs.len() * lineages);
        println!("SHARE arm={name} boundary_turns={turns} lines_that_took_a_revision={revised} lines_whose_recipe_moved={moved} (of {}) members_at_the_end: current_recipe={current} older_body={older}", runs.len() * lineages);
    }
}
