// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: is the matter budget closed, or a flow-through?
//! Runs the played A1 founding (see `limits_decomposers`) with flat
//! dormancy and the default limits, over seeds, a tick at a time with the
//! flow record handed over (which changes no outcome; checked on the first
//! seed against a plain run). Every `LEDGER_SAMPLE` ticks it reads
//! `Simulation::budget` and prints each reservoir, the world's total and,
//! for the span since the last sample, each reservoir's change as matter
//! flowed in, matter flowed out, and the rest: what changed class with no
//! flow (a death, a waking, a birth's new body). The rest summed over every
//! reservoir is matter that appeared or vanished with no flow; `issued`
//! counts the dev source, the only declared way in. Ignored:
//!
//! `cargo test --release --test ledger_budget -- --ignored --nocapture`
//!
//! Knobs: `LEDGER_SEEDS` ("0-9" or "3,5,9"), `LEDGER_TICKS` (600),
//! `LEDGER_SAMPLE` (50).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Budget, Limits, Rates},
    flows::{Flow, Holder},
    rules::Dormancy,
};
use std::collections::BTreeMap;

fn seeds() -> Vec<u64> {
    let text = std::env::var("LEDGER_SEEDS").unwrap_or_else(|_| "0-9".into());
    text.split(',')
        .flat_map(|part| match part.trim().split_once('-') {
            Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
            None => vec![part.trim().parse().unwrap()],
        })
        .collect()
}

fn knob(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn founding(seed: u64) -> Founding {
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
        ..Founding::default()
    }
}

fn session(seed: u64) -> Session {
    Session::new(founding(seed).generate().unwrap(), Execution::Individuals).unwrap()
}

/// A body's reservoir name, as `Budget::reservoirs` writes it.
fn class_of(s: &Session, id: u64) -> Option<String> {
    let e = s.sim.state().population.get(id)?;
    let seed = s.sim.genesis().rules.dormancy.as_ref().is_some_and(|d| e.traits.contains(&d.key));
    let kind = match (e.alive, seed) {
        (false, _) => "dead",
        (true, true) => "seeds",
        (true, false) => "living",
    };
    Some(format!("{kind} {}", e.lineage))
}

/// Where each body stood when the tick began, by first identity.
fn before(s: &Session) -> Vec<(u64, u64, String)> {
    let groups = &s.sim.state().population.groups;
    groups.iter().filter_map(|(&first, g)| Some((first, g.count, class_of(s, first)?))).collect()
}

fn holder(s: &Session, was: &[(u64, u64, String)], h: Holder, key: &str) -> String {
    let Some(id) = h.body() else {
        return match h {
            Holder::Site(_) => format!("site {key}"),
            Holder::Dev => "dev".into(),
            _ => "projects".into(),
        };
    };
    let at = was.partition_point(|(first, ..)| *first <= id);
    match at.checked_sub(1).map(|i| &was[i]) {
        Some((first, count, class)) if id < first + count => class.clone(),
        _ => class_of(s, id).map_or("gone".into(), |c| format!("{c} (born)")),
    }
}

#[derive(Default)]
struct Span {
    inflow: BTreeMap<String, i128>,
    outflow: BTreeMap<String, i128>,
}

fn tally(span: &mut Span, s: &Session, was: &[(u64, u64, String)], flows: &[Flow]) {
    for f in flows.iter().filter(|f| !f.internal()) {
        let n = i128::from(f.amount) * i128::from(f.count);
        let (from, to) = (holder(s, was, f.from.0, &f.from.1), holder(s, was, f.to.0, &f.to.1));
        // A newborn's matter is in its lineage's living reservoir at the tick's end.
        let (from, to) = (from.replace(" (born)", ""), to.replace(" (born)", ""));
        *span.outflow.entry(from).or_default() += n;
        *span.inflow.entry(to).or_default() += n;
    }
}

fn map(b: &Budget) -> BTreeMap<String, i128> {
    b.reservoirs().into_iter().filter(|(_, v)| *v != 0).collect()
}

/// The kinds the share table groups reservoirs into.
fn kind(name: &str) -> &'static str {
    match name.split(' ').next().unwrap() {
        "living" => "living",
        "seeds" => "seeds",
        "dead" => "dead",
        "site" if name == "site world:soil" => "site soil",
        "site" => "site litter",
        _ => "other",
    }
}

fn shares(b: &Budget) -> String {
    let mut by: BTreeMap<&str, i128> = BTreeMap::new();
    for (name, v) in b.reservoirs() {
        *by.entry(kind(&name)).or_default() += v;
    }
    let total = b.total().max(1) as f64;
    by.iter().map(|(k, v)| format!("{k} {:.1}%", *v as f64 * 100.0 / total)).collect::<Vec<_>>().join(", ")
}

#[test]
#[ignore = "a measurement; prints the matter budget"]
fn matter_budget_over_seeds() {
    let (ticks, every) = (knob("LEDGER_TICKS", 600), knob("LEDGER_SAMPLE", 50));
    let mut grand = vec![];
    for seed in seeds() {
        let mut s = session(seed);
        let mut last = s.sim.budget();
        println!("LEDGER seed={seed} t=0 total={} {:?}", last.total(), map(&last));
        let mut span = Span::default();
        let (mut issued, mut leak) = (0i128, 0i128);
        let mut marks = vec![(0, last.total(), shares(&last))];
        for t in 1..=ticks {
            let was = before(&s);
            let r = s.advance_tick_with_flows().unwrap();
            issued += i128::from(cosmograph::flows::issued(&r.flows));
            tally(&mut span, &s, &was, &r.flows);
            if t % every != 0 && t != ticks {
                continue;
            }
            let now = s.sim.budget();
            let (a, b) = (map(&last), map(&now));
            println!("LEDGER seed={seed} t={t} total={} net={:+} issued={issued} {b:?}", now.total(), now.total() as i128 - last.total() as i128);
            let names: std::collections::BTreeSet<_> = a.keys().chain(b.keys()).chain(span.inflow.keys()).chain(span.outflow.keys()).collect();
            let mut rest_sum = 0;
            for name in names.into_iter().filter(|n| *n != "dev" && *n != "gone") {
                let (was, is) = (a.get(name).copied().unwrap_or(0), b.get(name).copied().unwrap_or(0));
                let (inn, out) = (span.inflow.get(name).copied().unwrap_or(0), span.outflow.get(name).copied().unwrap_or(0));
                let rest = is - was - inn + out;
                rest_sum += rest;
                println!("  span<={t} {name:<28} {was:>8} -> {is:>8}  flow in {inn:>8} out {out:>8}  no-flow {rest:>+8}");
            }
            let gone = span.inflow.get("gone").copied().unwrap_or(0) - span.outflow.get("gone").copied().unwrap_or(0);
            let dev = span.inflow.get("dev").copied().unwrap_or(0) - span.outflow.get("dev").copied().unwrap_or(0);
            leak += rest_sum + gone;
            println!("  span<={t} no-flow summed over reservoirs {rest_sum:+}, to or from bodies gone {gone:+}, dev {dev:+}");
            marks.push((t, now.total(), shares(&now)));
            last = now;
            span = Span::default();
        }
        println!("LEDGER seed={seed} summary: net over run {:+}, issued {issued}, unflowed leak {leak:+}", last.total() as i128 - marks[0].1 as i128);
        grand.push((seed, marks));
        if grand.len() == 1 {
            let mut plain = session(seed);
            plain.advance(ticks).unwrap();
            println!("LEDGER seed={seed} recording the flows changed nothing: {}", plain.sim.state_hash() == s.sim.state_hash());
        }
    }
    println!("TABLE seed total@0 total@300 total@600 | shares at 0 | at 300 | at 600");
    for (seed, marks) in grand {
        let at = |t| marks.iter().find(|m| m.0 == t).or(marks.last()).unwrap();
        let (a, b, c) = (at(0), at(300), at(600));
        println!("TABLE {seed} {} {} {} | {} | {} | {}", a.1, b.1, c.1, a.2, b.2, c.2);
    }
}
