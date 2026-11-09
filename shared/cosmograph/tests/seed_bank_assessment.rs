// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SC3's measured assessment (sim plan S10.2, case 1189): where the
//! producers' matter goes on the two-year bar's founding, month by month,
//! read from every matter move the sim makes. Observation only: the run
//! advances a tick at a time with its moves handed over, which changes no
//! outcome. `SEED_BANK_SEED` (9), `SEED_BANK_MONTHS` (21) and an absolute
//! `SEED_BANK_REPORT` path for the JSON.

use cosmograph::{
    Execution, Founding, Session,
    anatomy::root_store,
    bodied::Bodies,
    ecology::{Rates, Web},
    flows::{Flow, Holder},
    schema::{Entity, Id, Key},
};
use serde::Serialize;
use std::collections::BTreeMap;

const DIETS: [&str; 5] = ["producer", "grazer", "decomposer", "predator", "omnivore"];

fn diet(lineage: &str) -> &'static str {
    lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()).map_or("other", |i| DIETS[i % 5])
}

fn env(key: &str, default: u64) -> u64 {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

#[derive(Default, Serialize)]
struct Month {
    month: u64,
    /// Matter moves touching a producer, by "process | from | to".
    moves: BTreeMap<String, u128>,
    producers: BTreeMap<Key, BTreeMap<&'static str, u128>>,
    /// Matter still held by dead bodies, by diet and part class.
    dead: BTreeMap<String, u128>,
    sites: BTreeMap<Id, BTreeMap<Key, u64>>,
    /// Per-process evaluations and acceptances this month, where summed.
    acts: BTreeMap<String, u128>,
    /// Deaths this month by process and diet.
    deaths: BTreeMap<String, u64>,
}

/// What a holder is to a producer move: its root, its exposed tissue,
/// another part, its ledger, a site account, or another body by diet.
fn class(s: &Session, h: Holder, account: &str, own: Option<Id>) -> String {
    let state = s.sim.state();
    let rules = &s.sim.genesis().rules;
    // A newborn landing at another site than its parent was scattered.
    let parent_site = own.and_then(|o| state.population.get(o)).map(|e| e.place);
    let other = |id: Id| {
        let e = state.population.get(id);
        let d = e.map_or("gone", |e| diet(&e.lineage));
        match e.filter(|e| e.born == state.tick) {
            Some(child) if Some(child.place) != parent_site => format!("scattered newborn {d}"),
            Some(_) => format!("newborn {d}"),
            None => format!("body {d}"),
        }
    };
    match h {
        Holder::Site(_) => format!("site {account}"),
        Holder::Part(id, _) | Holder::Entity(id) if Some(id) != own => other(id),
        Holder::Part(id, part) => match state.population.get(id) {
            Some(e) if e.alive => part_class(e, rules, account, part).into(),
            Some(e) => format!("dead {}", part_class(e, rules, account, part)),
            None => "part of gone body".into(),
        },
        Holder::Entity(id) if state.population.get(id).is_some_and(|e| !e.alive) => "dead ledger".into(),
        Holder::Entity(_) => "ledger".into(),
        _ => "other".into(),
    }
}

fn part_class(e: &Entity, rules: &cosmograph::rules::Rules, account: &str, part: cosmograph::schema::PartId) -> &'static str {
    match root_store::contract(e, rules, account) {
        Some((_, root)) if root == part => "root",
        Some(_) if root_store::exposed(e, rules, part) => "exposed",
        Some(_) => "other part",
        None => "part",
    }
}

fn record(s: &Session, flow: &Flow, month: &mut Month) {
    let producer = |k: &Option<cosmograph::flows::Kind>| k.as_ref().is_some_and(|k| diet(&k.lineage) == "producer");
    if !producer(&flow.from_kind) && !producer(&flow.to_kind) {
        return;
    }
    let process = match &flow.made_by {
        cosmograph::flows::MadeBy::Process(p) => p.split('-').next().unwrap_or(p).to_string(),
        cosmograph::flows::MadeBy::Command(c) => c.clone(),
    };
    let own = if producer(&flow.from_kind) { flow.from.0.body() } else { flow.to.0.body() };
    let from = class(s, flow.from.0, &flow.from.1, own);
    let to = class(s, flow.to.0, &flow.to.1, own);
    *month.moves.entry(format!("{process} | {from} | {to}")).or_default() += u128::from(flow.amount) * u128::from(flow.count);
}

fn snapshot(s: &Session, month: &mut Month) {
    let state = s.sim.state();
    let rules = &s.sim.genesis().rules;
    for g in state.population.groups.values() {
        let e = &g.entity;
        let n = u128::from(g.count);
        let d = diet(&e.lineage);
        for (&id, part) in &e.parts {
            for (account, &v) in &part.matter {
                let class = part_class(e, rules, account, id);
                if e.alive && d == "producer" {
                    *month.producers.entry(e.lineage.clone()).or_default().entry(class).or_default() += u128::from(v) * n;
                } else if !e.alive {
                    *month.dead.entry(format!("{d} {class}")).or_default() += u128::from(v) * n;
                }
            }
        }
        if e.alive && d == "producer" {
            *month.producers.entry(e.lineage.clone()).or_default().entry("members").or_default() += n;
        }
        if !e.alive {
            *month.dead.entry(format!("{d} bodies")).or_default() += n;
        }
    }
    month.sites = state.sites.iter().map(|(&id, site)| (id, site.accounts.clone())).collect();
}

#[test]
#[ignore = "two native years a tick at a time with every matter move; run in release"]
fn where_the_producers_matter_goes() {
    let seed = env("SEED_BANK_SEED", 9);
    let months = env("SEED_BANK_MONTHS", 21);
    let grid = cosmograph::map::Grid { width: 2, height: 2, ..cosmograph::map::Grid::drawn(seed) };
    let genesis = Founding {
        seed,
        sites: 4,
        population: 240,
        cohort_size: 1,
        lineages: 10,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::WorldTime,
        experimental_food_contract: true,
        bodies: Some(Bodies { clutch: [1, 1], sensing: true, grazing: true, hazard: [0, 0], wound: [0, 0], ..Bodies::default() }),
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut s = Session::new(genesis, Execution::Grouped).unwrap();
    let month_ticks = cosmograph::rules::year_ticks(s.sim.genesis().rules.tick_microseconds()) / 12;
    let start = s.sim.state().tick;
    let mut out = vec![];
    let mut first = Month::default();
    snapshot(&s, &mut first);
    out.push(first);
    let mut acts_before: BTreeMap<String, u128> = BTreeMap::new();
    for m in 1..=months {
        let mut month = Month { month: m, ..Default::default() };
        while s.sim.state().tick < start + m * month_ticks {
            let result = s.advance_tick_with_flows().expect("a native tick");
            for flow in &result.flows {
                record(&s, flow, &mut month);
            }
        }
        snapshot(&s, &mut month);
        let since = start + (m - 1) * month_ticks;
        let state = s.sim.state();
        for e in state.events.values().filter(|e| e.tick > since && (e.process.starts_with("ecology:death") || e.process.starts_with("ecology:hazard"))) {
            let d = state.population.get(e.subject).map_or("gone", |b| diet(&b.lineage));
            let p = e.process.split('-').next().unwrap_or(&e.process);
            *month.deaths.entry(format!("{p} {d}")).or_default() += 1;
        }
        for (label, [_, _, count]) in cosmograph::profile_totals() {
            if label.contains('#') && (label.contains("produce") || label.contains("regrow") || label.contains("feed") || label.contains("birth")) {
                let was = acts_before.insert(label.clone(), count).unwrap_or(0);
                month.acts.insert(label, count - was);
            }
        }
        let members: u128 = month.producers.values().map(|p| p.get("members").copied().unwrap_or(0)).sum();
        eprintln!("seed {seed} month {m}: producers {members}");
        out.push(month);
    }
    if let Some(path) = std::env::var_os("SEED_BANK_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&out).unwrap()).unwrap();
    }
}
