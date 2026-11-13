// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: what the detritus rule (`Founding.detritus`: an
//! animal's meal leaves a share of what it took at its site as dung, which
//! dung mouths and rot return to the soil; lineage-rounds D39 model 4, the
//! catalog's mechanic `dung-stock`) changes in the played four-line world
//! (producers 0 and 3, the played grazer 1, the decomposer 2) and, with the
//! word `diets`, the five-lineage web (a predator 3 and an omnivore 4), with
//! the stacked rules `tests/mutualism_share.rs` and `tests/defence_share.rs`
//! use (rot, 12 decomposer bites, spoilage, the repertoire, categories,
//! inherit, flat dormancy, default limits; every `SHARE_SPAN` ticks the
//! unplayed lines weigh candidates and drift moves their templates, for
//! `SHARE_BOUNDARIES` boundaries and one more span after). The dung class
//! leans on the categories rule, so every arm with the rule sets it.
//!
//! Per arm it prints the median count of each line at the last tick, the
//! lines alive, the dung deposited and eaten per site per tick, what rot
//! returned, the decomposers' median matter and the median soil over the
//! seeds at the last tick, then the D39 test of the detritus web: with dung
//! and a dung feeder, does the feeder persist on dung, and is the producers'
//! soil supply at least as large.
//!
//! The founded decomposer has no body parts, so the played world's only
//! dung eater is its founded root; arms plant a feeder by words: `rev:<line>:
//! <part>` revises that line at tick 0 to the lowered catalog part (the
//! omnivore of the diets web, `rev:4:dung-mouth`, a mouth for dung alone);
//! `dung:<line>` gives that line's bodies a dung-only mouth organ at tick 0
//! (a decomposer line has no recipe to revise, so it is planted by editing
//! bodies, as `organ:` is in `mutualism_share`); `noeat` removes the dung
//! turns, so only rot returns dung; `diets` is the five-lineage web; `rule`
//! sets the rule. Default arms: `base`, `rule`, `noeat` (rule, only rot),
//! `feeder` (rule with `dung:2`), and the same four in the diets web with
//! `rev:4:dung-mouth` for its feeder.
//!
//! `cargo test --release --test detritus_share -- --ignored --nocapture`
//!
//! Knobs: `SHARE_SEEDS` ("0-19" or "3,5,9"), `SHARE_SPAN` (100),
//! `SHARE_BOUNDARIES` (4), `SHARE_SCORE` (15), `SHARE_ARMS` (split by ";";
//! an arm is a name, or `name=word+word`).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Categories, DECOMPOSER_BITES, DUNG, Detritus, Limits, Rates, Rot, Spoilage},
    flows::{Holder, MadeBy},
    history::Command,
    lineage::{boundary::{self, Candidate, Score, ScoringSource}, revise},
    process::{DeadClass, IntakePort},
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

fn founding(seed: u64, rule: bool, diets: bool) -> Founding {
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
        detritus: rule.then(Detritus::default),
        inherit: true,
        lineages: if diets { 5 } else { 4 },
        web: if diets { cosmograph::ecology::Web::Diets } else { Default::default() },
        ..Founding::default()
    }
}

#[derive(Default)]
struct Run {
    last: Vec<u64>,
    matter: Vec<u64>,
    /// Dung deposited at the sites by meals, taken by dung mouths, and rotted to soil.
    deposited: u64,
    eaten: u64,
    rotted: u64,
    /// The soil the sites hold at the last tick.
    soil: u64,
    refused: Vec<String>,
}

fn run(seed: u64, words: &[String], span: Tick, boundaries: u64, scoring: Tick) -> Run {
    let f = founding(seed, words.iter().any(|w| w == "rule"), words.iter().any(|w| w == "diets"));
    let lineages = f.lineages as usize;
    let mut g = f.generate().unwrap();
    g.rules.directing.get_or_insert_with(Directing::default).drift = true;
    if words.iter().any(|w| w == "noeat") {
        g.rules.processes.retain(|id, _| !id.starts_with("detritus:eat-"));
    }
    // A decomposer line carries no recipe in the played world, so a dung mouth is planted by
    // editing its bodies at tick 0 (a dung-only organ beside whatever it has); its young copy them.
    for line in words.iter().filter_map(|w| w.strip_prefix("dung:")) {
        for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == format!("lineage:{line}")) {
            let e = group.entity_mut();
            let id = *e.parts.keys().next().unwrap();
            let held = cosmograph::process::Registry::native()
                .get(&cosmograph::process::ProcessId::new("mesocosm", "adhesion")).unwrap().reference();
            let part = e.part_mut(&id).unwrap();
            part.cells.insert("function:adhesion".into(), 1);
            part.functions.insert("function:adhesion".into());
            part.port = Some(IntakePort::none().supported_by(held).with_deadstock().taking(DeadClass::Dung));
            e.lay_out();
        }
    }
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let mut out = Run::default();
    for word in words.iter().filter_map(|w| w.strip_prefix("rev:")) {
        let (line, part) = word.split_once(':').expect("rev:<line>:<part>");
        let line = format!("lineage:{line}");
        let d = s.sim.state().lineages[&line].development.clone().unwrap_or_else(|| panic!("{line} has no development in arm {words:?}"));
        let want = format!("kind:catalog-{part}");
        match revise::offered(&s, &line, &d, usize::MAX).into_iter().find(|(n, _)| n.ends_with(&want)) {
            Some((_, development)) => {
                s.command(Command::Revise { lineage: line, development }).unwrap();
            },
            None => out.refused.push(word.to_owned()),
        }
    }
    let mut source: Option<ScoringSource> = None;
    for b in 0..=boundaries {
        for _ in 0..span {
            let r = s.advance_tick_with_flows().unwrap();
            for f in &r.flows {
                let moved = f.amount * f.count;
                if matches!(f.to.0, Holder::Site(_)) && f.to.1 == DUNG {
                    out.deposited += moved;
                }
                match &f.made_by {
                    MadeBy::Process(p) if p.starts_with("detritus:eat-") && matches!(f.from.0, Holder::Site(_)) => out.eaten += moved,
                    MadeBy::Process(p) if p == "detritus:rot" && matches!(f.from.0, Holder::Site(_)) => out.rotted += moved,
                    _ => {},
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
        boundary::adapt_since_using(&mut s, Some("lineage:1"), since, &revise::revisions, scoring, &mut score).unwrap();
    }
    out.soil = s.sim.state().sites.values().map(|site| site.accounts.get("world:soil").copied().unwrap_or(0)).sum();
    out.last = vec![0; lineages];
    out.matter = vec![0; lineages];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) {
            out.last[i] += g.count;
            out.matter[i] += g.count * cosmograph::anatomy::books(&g.entity).get(&format!("matter:{i}-0")).copied().unwrap_or(0);
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

fn arms() -> Vec<(String, Vec<String>)> {
    let text = std::env::var("SHARE_ARMS").unwrap_or_else(|_| {
        "base=;rule=rule;noeat=rule+noeat;feeder=rule+dung:2;\
         d-base=diets;d-rule=rule+diets;d-noeat=rule+diets+noeat;d-feeder=rule+diets+rev:4:dung-mouth"
            .into()
    });
    text.split(';')
        .map(|a| {
            let (name, words) = a.split_once('=').unwrap_or((a, a));
            (name.trim().to_string(), words.split('+').filter(|w| !w.is_empty()).map(str::to_string).collect())
        })
        .collect()
}

#[test]
#[ignore = "a measurement; prints a table"]
fn what_the_detritus_web_changes() {
    let (span, boundaries, scoring) = (knob("SHARE_SPAN", 100), knob("SHARE_BOUNDARIES", 4), knob("SHARE_SCORE", 15));
    let seeds = seeds();
    let arms = arms();
    let jobs: Vec<(usize, u64)> = (0..arms.len()).flat_map(|a| seeds.iter().map(move |&s| (a, s))).collect();
    let next = AtomicUsize::new(0);
    let done: Mutex<BTreeMap<(usize, u64), Run>> = Mutex::new(Default::default());
    let workers = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| loop {
                let j = next.fetch_add(1, Ordering::SeqCst);
                let Some(&(a, seed)) = jobs.get(j) else { break };
                let r = run(seed, &arms[a].1, span, boundaries, scoring);
                done.lock().unwrap().insert((a, seed), r);
            });
        }
    });
    let done = done.into_inner().unwrap();
    let ticks = span * (boundaries + 1);
    println!("SHARE seeds={} span={span} boundaries={boundaries} scoring={scoring} ticks={ticks} sites=6", seeds.len());
    println!("SHARE arm | median count per line (0..) | lines alive | dung deposited, eaten, rotted per site per tick | decomposer (line 2) median matter | median soil");
    for (a, (name, _)) in arms.iter().enumerate() {
        let runs: Vec<&Run> = seeds.iter().map(|&s| &done[&(a, s)]).collect();
        let lines = runs[0].last.len();
        let counts: Vec<String> = (0..lines).map(|i| format!("{:.1}", median(runs.iter().map(|r| r.last[i]).collect()))).collect();
        let alive = runs.iter().flat_map(|r| r.last.iter()).filter(|&&n| n > 0).count();
        let per = |f: fn(&Run) -> u64| runs.iter().map(|r| f(r)).sum::<u64>() as f64 / (runs.len() as f64 * 6.0 * ticks as f64);
        println!(
            "SHARE {name:9} | {} | {alive} of {} | {:.3}, {:.3}, {:.3} | {:.1} | {:.1}",
            counts.join(" "), runs.len() * lines,
            per(|r| r.deposited), per(|r| r.eaten), per(|r| r.rotted),
            median(runs.iter().map(|r| r.matter[2]).collect()), median(runs.iter().map(|r| r.soil).collect()),
        );
        let refused: Vec<_> = runs.iter().flat_map(|r| r.refused.iter()).collect();
        if !refused.is_empty() {
            println!("SHARE {name} planted revisions not offered: {} of {}", refused.len(), runs.len());
        }
    }
}
