// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Diagnosis, not a gate: why the second producer (lineage 3) of the played A1
//! world goes extinct by about tick 200 without dormancy. All ignored; run in
//! release:
//!
//! `cargo test --release --test producer_pair_diagnosis -- --ignored --nocapture ARMS`
//!
//! Tests: `tally` (per-10-tick table of L0 and L3: counts, births, deaths by
//! process, bites, light/soil captured, reserves, sites, ages; knobs
//! `PP_SEED`, `PP_TICKS`, `PP_ARM`), `arms` (counts of all four lineages at
//! ticks 50/100/150/200/300 for every arm over `PP_SEEDS`).
//! Arms (`PP_ARM`): base, swap_lifespans, same_lifespan, no_grazers,
//! l3_as_l0 (founded 32 like L0), l0_as_l3 (founded 64), stagger (founders
//! born spread over a lifespan), no_hazard, no_age.

use cosmograph::{
    Execution, Founding, Session,
    anatomy::books,
    bodied::Bodies,
    directing::Played,
    ecology::Rates,
    flows::MadeBy,
    rules::{Dormancy, Effect, Query},
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

fn founding(seed: u64, dormant: bool) -> Founding {
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
        dormancy: dormant.then(Dormancy::flat),
        ..Founding::default()
    }
}

fn lin(i: u32) -> String { format!("lineage:{i}") }

/// The age ceiling of lineage `i`, in ticks.
fn lifespan_ticks(g: &Genesis, i: u32) -> u64 {
    g.rules.processes[&format!("ecology:age-{i}")].requires.iter()
        .find_map(|q| if let Query::Age { at_least } = q { Some(*at_least) } else { None }).unwrap()
}

fn set_lifespan(g: &mut Genesis, i: u32, ticks: u64) {
    for id in [format!("ecology:age-{i}")] {
        for q in &mut g.rules.processes.get_mut(&id).unwrap().requires {
            if let Query::Age { at_least } = q { *at_least = ticks; }
        }
    }
}

fn arm(mut g: Genesis, name: &str) -> Genesis {
    let (l0, l3) = (lifespan_ticks(&g, 0), lifespan_ticks(&g, 3));
    match name {
        "base" => {}
        "swap_lifespans" => { set_lifespan(&mut g, 0, l3); set_lifespan(&mut g, 3, l0); }
        "same_lifespan" => { set_lifespan(&mut g, 3, l0); }
        "no_grazers" => { g.population.groups.retain(|_, c| c.entity.lineage != lin(1)); }
        "l3_as_l0" | "l0_as_l3" => {
            let (keep, thin) = if name == "l3_as_l0" { (3, 3) } else { (0, 0) };
            let _ = keep;
            // Halve L3 (64 -> 32) or double-up L0 by cloning its groups (32 -> 64).
            if name == "l3_as_l0" {
                let mut n = 0;
                g.population.groups.retain(|_, c| { if c.entity.lineage != lin(thin) { return true; } n += 1; n % 2 == 1 });
            } else {
                let extra: Vec<_> = g.population.groups.iter().filter(|(_, c)| c.entity.lineage == lin(0)).map(|(k, c)| (k.clone(), c.clone())).collect();
                let mut top = g.population.groups.iter().map(|(k, c)| k + c.count).max().unwrap_or(0);
                for (_, c) in extra { let w = c.count; g.population.groups.insert(top, c); top += w; }
            }
        }
        "stagger" => {
            // Founders of both producers get born spread over the older half of a lifespan.
            let mut n = 0u64;
            for c in g.population.groups.values_mut() {
                let e = c.entity_mut();
                if e.lineage == lin(0) || e.lineage == lin(3) { n += 1; e.born = e.born.saturating_sub((n * 7) % 90); }
            }
        }
        "l3_first" => { g.rules.processes.get_mut("ecology:produce-3").unwrap().priority = -1; }
        "l3_first_all" => {
            for id in ["produce", "upkeep", "birth"] { g.rules.processes.get_mut(&format!("ecology:{id}-3")).unwrap().priority -= 1; }
        }
        "no_l0" => { g.population.groups.retain(|_, c| c.entity.lineage != lin(0)); }
        "no_l3" => { g.population.groups.retain(|_, c| c.entity.lineage != lin(3)); }
        "no_hazard" => { g.rules.processes.retain(|id, _| !id.starts_with("ecology:hazard-")); }
        "no_age" => { g.rules.processes.retain(|id, _| !id.starts_with("ecology:age-")); }
        "no_hazard_no_age" => { g.rules.processes.retain(|id, _| !id.starts_with("ecology:hazard-") && !id.starts_with("ecology:age-")); }
        "l3_hazard_off" => { g.rules.processes.remove("ecology:hazard-3"); set_lifespan(&mut g, 3, 1_000_000); }
        other => panic!("arm {other}"),
    }
    g
}

fn counts(s: &Session) -> [u64; 4] {
    let mut c = [0u64; 4];
    for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
        if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|v| v.parse::<usize>().ok()) {
            if i < 4 { c[i] += g.count; }
        }
    }
    c
}

fn envs(k: &str, d: &str) -> String { std::env::var(k).unwrap_or(d.into()) }

#[derive(Default, Clone)]
struct W {
    births: u64,
    deaths: BTreeMap<String, u64>,
    eaten_deaths: u64,
    bites: u64,
    bitten: u64,
    light: u64,
    soil: u64,
    acts: u64,
}

#[test]
#[ignore = "diagnostic table"]
fn tally() {
    let seed: u64 = envs("PP_SEED", "9").parse().unwrap();
    let ticks: u64 = envs("PP_TICKS", "200").parse().unwrap();
    let dormant = envs("PP_DORMANT", "0") == "1";
    let g = arm(founding(seed, dormant).generate().unwrap(), &envs("PP_ARM", "base"));
    for i in [0, 3] {
        let p = &g.rules.processes[&format!("ecology:produce-{i}")];
        eprintln!("L{i}: lifespan ticks {}, produce period {:?}, birth {:?}, upkeep {:?}", lifespan_ticks(&g, i), p.period,
            g.rules.processes[&format!("ecology:birth-{i}")].period, g.rules.processes[&format!("ecology:upkeep-{i}")].period);
    }
    let mut start: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    let mut per_site0: BTreeMap<_, [u64; 4]> = BTreeMap::new();
    for c in g.population.groups.values() {
        let e = &c.entity;
        let sl = start.entry(e.lineage.clone()).or_default();
        sl.0 += c.count;
        sl.1 = sl.1.max(e.born);
        if let Some(i) = e.lineage.strip_prefix("lineage:").and_then(|v| v.parse::<usize>().ok()) { if i < 4 { per_site0.entry(e.place).or_default()[i] += c.count; } }
    }
    eprintln!("founded {start:?}\nfounded per site [L0,L1,L2,L3] {per_site0:?}");
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let mut w: [W; 2] = Default::default();
    let mut last_bite: BTreeMap<u64, u64> = BTreeMap::new();
    let mut seen_events = 0usize;
    let idx = |l: &str| match l { "lineage:0" => Some(0), "lineage:3" => Some(1), _ => None };
    for n in 1..=ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let MadeBy::Process(p) = &f.made_by else { continue };
            if p == "ecology:feed-1" {
                if let (Some(k), Some(b)) = (&f.from_kind, f.from.0.body()) {
                    if let Some(j) = idx(&k.lineage) { w[j].bites += f.count; w[j].bitten += f.amount * f.count; last_bite.insert(b.into(), n); }
                }
            }
            for (j, i) in [(0, 0), (1, 3)] {
                if p == &format!("ecology:produce-{i}") && f.to_kind.as_ref().is_some_and(|k| k.lineage == lin(i)) {
                    if f.from.1 == "sim:energy" { w[j].light += f.amount * f.count; w[j].acts += f.count; }
                    if f.from.1 == "world:soil" { w[j].soil += f.amount * f.count; }
                }
            }
        }
        if n % 10 == 0 {
            let st = s.sim.state();
            let _ = &mut seen_events;
            for e in st.events.values().filter(|e| e.tick > n - 10 && e.tick <= n) {
                let Some(b) = st.population.get(e.subject) else { continue };
                let Some(j) = idx(&b.lineage) else { continue };
                let i = if j == 0 { 0 } else { 3 };
                if e.process == format!("ecology:birth-{i}") { w[j].births += 1; }
                else if e.process.starts_with("ecology:death-") || e.process.starts_with("ecology:age-") || e.process.starts_with("ecology:hazard-") {
                    let key = e.process.rsplit_once('-').unwrap().0.to_string();
                    *w[j].deaths.entry(key.clone()).or_default() += 1;
                    if key == "ecology:death" && last_bite.get(&u64::from(e.subject)).is_some_and(|t| e.tick - t <= 3) { w[j].eaten_deaths += 1; }
                }
            }
            let c = counts(&s);
            // reserve histogram, age stats, sites
            let mut out = String::new();
            for (j, i) in [(0usize, 0u32), (1, 3)] {
                let mut hist = [0u64; 14];
                let mut ages = vec![];
                let mut sites: BTreeMap<_, u64> = BTreeMap::new();
                for g in st.population.groups.values().filter(|g| g.entity.alive && g.entity.lineage == lin(i)) {
                    let t = books(&g.entity).get(&format!("matter:{i}-0")).copied().unwrap_or(0);
                    hist[(t as usize).min(13)] += g.count;
                    ages.push(n - g.entity.born);
                    *sites.entry(g.entity.place).or_default() += g.count;
                }
                ages.sort();
                let q = |f: f64| ages.get(((ages.len() as f64 - 1.0) * f) as usize).copied().unwrap_or(0);
                let wj = &w[j];
                out += &format!(
                    "  L{i}: n={:>5} births={:>4} deaths={:?} eatenDeaths={} bites={} bitten={} light={} soil={} acts={} age[q0,q50,q100]=[{},{},{}] sites={:?} reserve={:?}\n",
                    c[i as usize], wj.births, wj.deaths, wj.eaten_deaths, wj.bites, wj.bitten, wj.light, wj.soil, wj.acts, q(0.0), q(0.5), q(1.0),
                    sites.values().collect::<Vec<_>>(), hist);
            }
            let soil: Vec<_> = st.sites.iter().map(|(k, x)| (*k, x.accounts.get("world:soil").copied().unwrap_or(0), x.accounts.get("sim:energy").copied().unwrap_or(0) / 1000)).collect();
            eprintln!("t={n} counts {c:?} site (id,soil,light/1000) {soil:?}\n{out}");
            w = Default::default();
        }
    }
}

#[test]
#[ignore = "diagnostic table"]
fn arms() {
    let seeds: Vec<u64> = envs("PP_SEEDS", "9").split(',').filter_map(|v| v.parse().ok()).collect();
    let names: Vec<String> = envs("PP_ARMS", "base,swap_lifespans,same_lifespan,no_grazers,l3_as_l0,stagger,no_hazard,no_age,no_hazard_no_age").split(',').map(String::from).collect();
    let dormant = envs("PP_DORMANT", "0") == "1";
    let ticks: u64 = envs("PP_TICKS", "300").parse().unwrap();
    let at: BTreeSet<u64> = [0, 50, 100, 150, 200, 300, 450].into_iter().collect();
    let jobs: Vec<(u64, String)> = seeds.iter().flat_map(|&s| names.iter().map(move |n| (s, n.clone()))).collect();
    let out = std::sync::Mutex::new(BTreeMap::new());
    std::thread::scope(|sc| {
        for chunk in jobs.chunks(jobs.len().div_ceil(8)) {
            let out = &out; let at = &at;
            sc.spawn(move || {
                for (seed, name) in chunk {
                    let g = arm(founding(*seed, dormant).generate().unwrap(), name);
                    let mut s = Session::new(g, Execution::Individuals).unwrap();
                    let mut rows = vec![];
                    for n in 0..=ticks {
                        if at.contains(&n) { rows.push(format!("{n}:{:?}", counts(&s))); }
                        if n < ticks { s.advance(1).unwrap(); }
                    }
                    eprintln!("ROW seed {seed:>2} {name:<18} {}", rows.join(" "));
                    out.lock().unwrap().insert((*seed, name.clone()), rows.join(" "));
                }
            });
        }
    });
    for ((seed, name), row) in out.lock().unwrap().iter() { eprintln!("seed {seed:>2} dormant={dormant} {name:<18} {row}"); }
    let _ = Effect::Death;
}
