// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: what the nutrient rule (`Founding.nutrients`: the
//! ground holds several nutrients, a producer needs them in a fixed proportion
//! that differs by line and fixes only as far as its scarcest allows;
//! lineage-rounds D39 model 1, competition for limiting resources) changes in
//! the played four-line world (producers 0 and 3, the played grazer 1, the
//! decomposer 2) with the stacked rules `tests/mutualism_share.rs` uses (rot,
//! 12 decomposer bites, spoilage, the repertoire, categories, inherit, flat
//! dormancy, default limits; every `SHARE_SPAN` ticks the unplayed lines weigh
//! candidates and drift moves their templates, for `SHARE_BOUNDARIES`
//! boundaries and one more span after).
//!
//! Two questions. D39's test for competition: two producer lines limited by
//! different nutrients should coexist more often than two limited by the same.
//! Line 0 leads on the first nutrient and line 3 on the second, so their
//! needs can differ: arm `same` gives both the same need (one of each), arm
//! `diff` gives each only its own nutrient (two of one, none of the other), arm
//! `rule` the default two to one and one to two; each act makes as much tissue
//! as it takes, two in `same` and `diff`, three in `rule`. The number that
//! answers it is the seeds in which both producer lines are alive at the end.
//! The second, the reason the rule is built now: under scarcity, does
//! mutualism's plant side gain? The mutualism rule is set on the arms with
//! `mut`; a gain is the plant line with a working partner (`nboth`) doing
//! better than the plants with the rules set and nothing planted (`nmut`),
//! and the same under a scarcer ground (`s` arms, the second nutrient between a
//! fifth and three fifths of the soil).
//!
//! The planted organs are those of `mutualism_share.rs`: a word `rev:<line>:<part>`
//! revises a line at tick 0 to a lowered catalog part; `organ:<line>` gives a
//! line's bodies the exchange organ at tick 0 (a decomposer line has no recipe
//! to revise); `cheat` strips the planted plant organ's giving. Words for the
//! rule: `nut`, `tilt:<n>`, `base:<n>`, `spread:<lo>-<hi>`, `mut`.
//!
//! `cargo test --release --test nutrient_share -- --ignored --nocapture`
//!
//! Knobs: `SHARE_SEEDS` ("0-19" or "3,5,9"), `SHARE_SPAN` (100),
//! `SHARE_BOUNDARIES` (4), `SHARE_SCORE` (15), `SHARE_ARMS` (split by ";";
//! an arm is `name=word+word`).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Categories, DECOMPOSER_BITES, Limits, Mutualism, Nutrients, Rates, Rot, Spoilage, axis},
    flows::MadeBy,
    history::Command,
    lineage::{boundary::{self, Candidate, Score, ScoringSource}, revise},
    process::{IntakePort, Trade},
    repertoire::Repertoire,
    rules::{Directing, Dormancy},
    schema::Tick,
};
use std::{
    collections::BTreeMap,
    sync::{Mutex, atomic::{AtomicUsize, Ordering}},
};

const PLANT: &str = "rev:0:flora-mycorrhizal-sheath-nutrient-for-sugar-swap";
const FUNGUS: &str = "organ:2";

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

/// The nutrient rule the words set, if `nut` is among them.
fn nutrients(words: &[String]) -> Option<Nutrients> {
    let mut n = Nutrients::default();
    for w in words {
        let (key, value) = w.split_once(':').unwrap_or((w, ""));
        match key {
            "tilt" => n.tilt = value.parse().unwrap(),
            "base" => n.base = value.parse().unwrap(),
            "spread" => {
                let (lo, hi) = value.split_once('-').unwrap();
                n.spread = [lo.parse().unwrap(), hi.parse().unwrap()];
            },
            _ => {},
        }
    }
    words.iter().any(|w| w == "nut").then_some(n)
}

fn founding(seed: u64, words: &[String]) -> Founding {
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
        mutualism: words.iter().any(|w| w == "mut").then(Mutualism::default),
        nutrients: nutrients(words),
        inherit: true,
        lineages: 4,
        ..Founding::default()
    }
}

#[derive(Default)]
struct Run {
    last: Vec<u64>,
    matter: Vec<u64>,
    /// Soil carried to plants (the decomposers' turns) and tissue paid (the plants').
    carried: u64,
    paid: u64,
    /// What the producers drew of the second nutrient, what the sorting returned, and what tuned organs drew.
    drawn: u64,
    sorted: u64,
    tuned: u64,
    /// At the last tick, the sites' second nutrient over their soil, in thousandths, and the sites short of it (below an act's need of the soil).
    ratio: u64,
    short: u64,
    refused: Vec<String>,
}

fn run(seed: u64, words: &[String], span: Tick, boundaries: u64, scoring: Tick) -> Run {
    let f = founding(seed, words);
    let lineages = f.lineages as usize;
    let mut g = f.generate().unwrap();
    g.rules.directing.get_or_insert_with(Directing::default).drift = true;
    if words.iter().any(|w| w == "cheat") {
        // The plant's organ takes and does not give.
        let kind = g.rules.kinds.get_mut("kind:catalog-flora-mycorrhizal-sheath-nutrient-for-sugar-swap").unwrap();
        let old = kind.port.unwrap();
        kind.port = old.support().map(|s| IntakePort::none().trading(Trade::Takes).supported_by(s));
    }
    // A decomposer line carries no recipe in the played world (its lineage has no
    // development), so it cannot be revised to a catalog part; its bodies are given
    // the exchange organ at tick 0 instead, as the unit tests do, and its young copy them.
    for line in words.iter().filter_map(|w| w.strip_prefix("organ:")) {
        for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == format!("lineage:{line}")) {
            let e = group.entity_mut();
            let id = *e.parts.keys().next().unwrap();
            let held = cosmograph::process::Registry::native()
                .get(&cosmograph::process::ProcessId::new("mesocosm", "adhesion")).unwrap().reference();
            let part = e.part_mut(&id).unwrap();
            part.cells.insert("function:adhesion".into(), 1);
            part.functions.insert("function:adhesion".into());
            let port = part.port.unwrap_or_else(|| IntakePort::none().supported_by(held));
            part.port = Some(port.trading(Trade::Gives).trading(Trade::Takes));
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
                match &f.made_by {
                    MadeBy::Process(p) if p.starts_with("mutualism:nutrient-") && !f.internal() => out.carried += f.amount * f.count,
                    MadeBy::Process(p) if p.starts_with("mutualism:sugar-") && !f.internal() => out.paid += f.amount * f.count,
                    MadeBy::Process(p) if p.starts_with("ecology:produce-") && f.from.1 == axis(1) => out.drawn += f.amount * f.count,
                    MadeBy::Process(p) if p.starts_with("nutrients:sort-") && f.to.1 == axis(1) => out.sorted += f.amount * f.count,
                    MadeBy::Process(p) if p.starts_with("nutrients:tuned-") => out.tuned += f.amount * f.count,
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
    let sites = s.sim.state().sites.values();
    let (soil, other): (u64, u64) = sites.fold((0, 0), |(a, b), site| (a + site.accounts.get("world:soil").copied().unwrap_or(0), b + site.accounts.get(&axis(1)).copied().unwrap_or(0)));
    out.ratio = if soil == 0 { 0 } else { other * 1000 / soil };
    out.short = s.sim.state().sites.values().filter(|site| site.accounts.get(&axis(1)).copied().unwrap_or(0) < 2).count() as u64;
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
        let n = "nut+mut";
        let scarce = "spread:20-60";
        [
            "base=".to_string(),
            "rule=nut".into(),
            "same=nut+tilt:0".into(),
            "diff=nut+base:0+tilt:2".into(),
            format!("nmut={n}"),
            format!("nplant={n}+{PLANT}"),
            format!("nfungus={n}+{FUNGUS}"),
            format!("nboth={n}+{PLANT}+{FUNGUS}"),
            format!("ncheat={n}+{PLANT}+{FUNGUS}+cheat"),
            format!("srule=nut+{scarce}"),
            format!("smut={n}+{scarce}"),
            format!("splant={n}+{scarce}+{PLANT}"),
            format!("sboth={n}+{scarce}+{PLANT}+{FUNGUS}"),
        ]
        .join(";")
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
fn what_the_nutrients_change() {
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
    println!("SHARE seeds={} span={span} boundaries={boundaries} scoring={scoring}", seeds.len());
    println!("SHARE arm | line0 plant n/matter | line2 fungus n/matter | line3 plant n/matter | line1 grazer n | both plants alive | lines alive of 4x seeds | 2nd nutrient drawn / sorted / tuned | 2nd over soil (permille) | sites short of it | soil carried | tissue paid");
    for (a, (name, _)) in arms.iter().enumerate() {
        let runs: Vec<&Run> = seeds.iter().map(|&s| &done[&(a, s)]).collect();
        let n = |i: usize| median(runs.iter().map(|r| r.last[i]).collect());
        let m = |i: usize| median(runs.iter().map(|r| r.matter[i]).collect());
        let alive = runs.iter().flat_map(|r| r.last.iter()).filter(|&&n| n > 0).count();
        let both = runs.iter().filter(|r| r.last[0] > 0 && r.last[3] > 0).count();
        let (carried, paid): (u64, u64) = (runs.iter().map(|r| r.carried).sum(), runs.iter().map(|r| r.paid).sum());
        let (drawn, sorted, tuned): (u64, u64, u64) = (runs.iter().map(|r| r.drawn).sum(), runs.iter().map(|r| r.sorted).sum(), runs.iter().map(|r| r.tuned).sum());
        let (ratio, short) = (median(runs.iter().map(|r| r.ratio).collect()), runs.iter().map(|r| r.short).sum::<u64>());
        println!(
            "SHARE {name:8} | {:6.1}/{:6.1} | {:6.1}/{:6.1} | {:6.1}/{:6.1} | {:6.1} | {both} of {} | {alive} of {} | {drawn}/{sorted}/{tuned} | {ratio:.0} | {short} | {carried} | {paid}",
            n(0), m(0), n(2), m(2), n(3), m(3), n(1), runs.len(), runs.len() * 4
        );
        let refused: Vec<_> = runs.iter().flat_map(|r| r.refused.iter()).collect();
        if !refused.is_empty() {
            println!("SHARE {name} planted revisions not offered: {} of {}", refused.len(), runs.len());
        }
    }
}
