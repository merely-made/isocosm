// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: what the guard rule (`Founding.guard`: armour,
//! spines and glands against mouths' attacks and tolerances, lineage-rounds
//! D39 model 2) changes in the five-lineage diets web and the played four-line
//! world, with rot, 12 decomposer bites, spoilage, the repertoire, the
//! categories rule and inherit (D40), as `tests/categories_share.rs` runs them
//! (flat dormancy, default limits, a tick at a time with the flow record handed
//! over; every `SHARE_SPAN` ticks the unplayed lines weigh their candidates and
//! drift moves their templates, for `SHARE_BOUNDARIES` boundaries and one more
//! span after). Per arm it prints the lines alive and each lineage's median
//! count at the last tick, the catalog parts lines end carrying (defences, and
//! the counter-defences: attack mouths and detox organs), the harm events by
//! kind (per site per tick), the matter armour withheld, and the D39 test:
//! whether a defended line's eaters decline, as the median consumer count
//! over the seeds where some line ends defended and where some consumer ends
//! with a counter, against the seeds where none does. Ignored:
//!
//! Lines do not take a defence at a boundary (the boundary scoring weighs a
//! short span, over which a defence only costs), so the D39 question is
//! also asked by planting: a word `rev:<line>:<part>` in an arm revises that
//! line at tick 0 to the lowered catalog part `<part>` (as the played line's
//! review offers it), so the arm `guard+rev:0:cyanide-leaf` gives the plants a
//! gland, and `guard+rev:0:cyanide-leaf+rev:1:detox-gut` gives their grazer
//! its counter as well. Each is compared with the plain `guard` arm.
//!
//! `cargo test --release --test defence_share -- --ignored --nocapture`
//!
//! Knobs: `SHARE_SEEDS` ("0-19" or "3,5,9"), `SHARE_SPAN` (100),
//! `SHARE_BOUNDARIES` (4), `SHARE_SCORE` (15), `SHARE_ARMS` (split by ";";
//! words joined by "+": `guard` for the guard rule, `diets` for the
//! five-lineage web, otherwise the played four-line world; default "base;
//! guard;base+diets;guard+diets").

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    catalog::catalog,
    directing::Played,
    history::Command,
    ecology::{Categories, DECOMPOSER_BITES, Guard, Limits, Rates, Rot, Spoilage, Ward, Web},
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

fn founding(seed: u64, guard: bool, diets: bool) -> Founding {
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
        categories: Some(Categories::default()),
        guard: guard.then(Guard::default),
        inherit: true,
        web: if diets { Web::Diets } else { Web::Levels },
        lineages: if diets { 5 } else { 4 },
        ..Founding::default()
    }
}

#[derive(Default)]
struct Run {
    last: Vec<u64>,
    /// Harm events by kind: the categories' and spoilage's, and the wards'.
    harms: BTreeMap<&'static str, u64>,
    /// Matter armour withheld, and the bites it acted on.
    withheld: u64,
    resisted_bites: u64,
    /// Lines carrying a catalog part at the end, by part, and which kind of part.
    parts: BTreeMap<String, u64>,
    carrying: u64,
    /// A line of the flora kingdom ends carrying a defence; a line of the fauna kingdom does.
    defended_flora: bool,
    defended_fauna: bool,
    /// A consumer line ends carrying a counter: an attack mouth or a detox organ.
    countered: bool,
    /// Fauna lines alive at the end, and the matter they hold in all.
    consumers_alive: u64,
    consumers: u64,
    turns: u64,
    committed: u64,
    /// Planted revisions the world did not offer.
    refused: Vec<String>,
}

fn run(seed: u64, name: &str, span: Tick, boundaries: u64, scoring: Tick) -> Run {
    let words: Vec<&str> = name.split('+').collect();
    let f = founding(seed, words.contains(&"guard"), words.contains(&"diets"));
    let lineages = f.lineages as usize;
    let mut g = f.generate().unwrap();
    g.rules.directing.get_or_insert_with(Directing::default).drift = true;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let mut out = Run::default();
    for word in words.iter().filter_map(|w| w.strip_prefix("rev:")) {
        let (line, part) = word.split_once(':').expect("rev:<line>:<part>");
        let line = format!("lineage:{line}");
        let d = s.sim.state().lineages[&line].development.clone().unwrap();
        let want = format!("kind:catalog-{part}");
        let offer = revise::offered(&s, &line, &d, usize::MAX).into_iter().find(|(n, _)| n.ends_with(&want));
        match offer {
            Some((_, development)) => {
                s.command(Command::Revise { lineage: line, development }).unwrap();
            },
            None => out.refused.push(word.to_owned()),
        }
    }
    let mut source: Option<ScoringSource> = None;
    for b in 0..=boundaries {
        for _ in 0..span {
            let _ = s.advance_tick_with_flows().unwrap();
            for h in s.sim.harms() {
                let category = match (h.ward, h.tough, h.flesh) {
                    (Some(Ward::Armour), ..) => {
                        out.withheld += h.withheld;
                        out.resisted_bites += 1;
                        continue;
                    },
                    (Some(Ward::Spines), ..) => "spines",
                    (Some(Ward::Toxin), ..) => "toxin",
                    (None, true, _) => "tough",
                    (None, _, cosmograph::ecology::Flesh::Fresh) => "living-decomposer",
                    (None, _, cosmograph::ecology::Flesh::Aged) => "aged-flesh",
                    (None, _, cosmograph::ecology::Flesh::Toxic) => "toxic-flesh",
                };
                *out.harms.entry(category).or_default() += 1;
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
                println!("SHARE seed={seed} b={b} {} committed={:?}", t.lineage, t.committed);
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
    let rules = &s.sim.genesis().rules;
    for (key, l) in &s.sim.state().lineages {
        let Some(d) = &l.development else { continue };
        let named = d.recipe.kinds();
        let mut any = false;
        for (kind, e) in lowered.iter().flat_map(|r| r.kinds.iter()).filter(|(k, _)| named.contains(*k)) {
            let part = catalog().part_by_key(&e.part).unwrap();
            let port = rules.kinds[kind].port;
            let defence = port.is_some_and(|p| !p.is_mouth() && !p.strength().is_none());
            let counter = !defence && port.is_some_and(|p| (p.is_mouth() && !p.strength().is_none()) || p.tolerates(cosmograph::process::Tolerance::Toxin));
            any = true;
            *out.parts.entry(format!("{}{}", if defence { "defence:" } else if counter { "counter:" } else { "other:" }, e.part)).or_default() += 1;
            match (defence, l.kingdom.as_str()) {
                (true, "kingdom:flora") => out.defended_flora = true,
                (true, _) => out.defended_fauna = true,
                _ => {},
            }
            out.countered |= counter && l.kingdom == "kingdom:fauna";
        }
        out.carrying += u64::from(any);
        if l.kingdom == "kingdom:fauna" {
            let i = key.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok());
            let n = i.and_then(|i| out.last.get(i)).copied().unwrap_or(0);
            out.consumers += n;
            out.consumers_alive += u64::from(n > 0);
        }
    }
    out
}

fn median(mut v: Vec<u64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_unstable();
    let m = v.len() / 2;
    if v.len() % 2 == 1 { v[m] as f64 } else { (v[m - 1] + v[m]) as f64 / 2.0 }
}

#[test]
#[ignore = "a measurement; prints a table"]
fn what_the_guard_changes() {
    let (span, boundaries, scoring) = (knob("SHARE_SPAN", 100), knob("SHARE_BOUNDARIES", 4), knob("SHARE_SCORE", 15));
    let ticks = span * (boundaries + 1);
    let seeds = seeds();
    let text = std::env::var("SHARE_ARMS").unwrap_or_else(|_| "base;guard;base+diets;guard+diets".into());
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
        let mut harms: BTreeMap<&str, f64> = BTreeMap::new();
        for r in &runs {
            for (k, n) in &r.harms {
                *harms.entry(k).or_default() += *n as f64 / site_ticks;
            }
        }
        let harms = harms.into_iter().map(|(k, n)| format!("{k}={n:.4}")).collect::<Vec<_>>().join(" ");
        let (withheld, bites): (u64, u64) = (runs.iter().map(|r| r.withheld).sum(), runs.iter().map(|r| r.resisted_bites).sum());
        let carrying: u64 = runs.iter().map(|r| r.carrying).sum();
        let mut parts: BTreeMap<&str, u64> = BTreeMap::new();
        for r in &runs {
            for (p, n) in &r.parts {
                *parts.entry(p).or_default() += n;
            }
        }
        let (turns, committed): (u64, u64) = (runs.iter().map(|r| r.turns).sum(), runs.iter().map(|r| r.committed).sum());
        println!("SHARE arm={name} ticks={ticks} boundaries={boundaries} span={span} scoring={scoring} seeds={} lines_alive={alive} of {} median_last={med:?}", runs.len(), runs.len() * lineages);
        let refused: Vec<_> = runs.iter().flat_map(|r| r.refused.iter()).collect();
        if !refused.is_empty() {
            println!("SHARE arm={name} planted revisions not offered: {refused:?}");
        }
        println!("SHARE arm={name} lines_carrying_a_catalog_part={carrying} boundary_turns={turns} turns_committing_one={committed}");
        println!("SHARE arm={name} parts_carried_by_lines_at_the_end={parts:?}");
        println!("SHARE arm={name} harm_events_per_site_per_tick: {harms}");
        println!("SHARE arm={name} matter_withheld_by_armour={withheld} over {bites} resisted_bites (per site per tick {:.4})", withheld as f64 / site_ticks);
        // The D39 test: with the defence and its counter both available both lines persist; with the defence only, the eater declines.
        for (label, pick) in [
            ("defended_flora_and_a_counter", (|r: &Run| r.defended_flora && r.countered) as fn(&Run) -> bool),
            ("defended_flora_no_counter", |r| r.defended_flora && !r.countered),
            ("no_defended_flora_a_counter", |r| !r.defended_flora && r.countered),
            ("no_defended_flora_no_counter", |r| !r.defended_flora && !r.countered),
        ] {
            let cell: Vec<&&Run> = runs.iter().filter(|r| pick(r)).collect();
            let counts = cell.iter().map(|r| r.consumers).collect::<Vec<_>>();
            let alive = cell.iter().map(|r| r.consumers_alive).sum::<u64>();
            println!("SHARE arm={name} d39 {label}: seeds={} median_consumers={:.1} consumer_lines_alive={alive} defended_fauna_seeds={}", cell.len(), median(counts), cell.iter().filter(|r| r.defended_fauna).count());
        }
    }
}
