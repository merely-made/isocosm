// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Measurement, not a gate: edible matter beside headcount on the played A1
//! founding (see `limits_measure`), and the `stocked` limit that breeds only
//! on a stocked meadow. Ignored; prints lines. In release:
//!
//! `cargo test --release --test limits_stocked -- --ignored --nocapture stocked_trace`
//! `cargo test --release --test limits_stocked -- --ignored --nocapture stocked_arms`
//!
//! Knobs: `STOCK_SEEDS` ("3,5,9,11,17"), `STOCK_TICKS` (600), `STOCK_TRACE_SEED`
//! (9), `STOCK_TRACE_TICKS` (250), `STOCK_TRACE_ARM` (an arm name; "none"), `STOCK_ARMS` (a comma list of arm names to
//! run, default all), `STOCK_DORMANT` ("0"; "1" runs flat dormancy),
//! `STOCK_WORKERS` (8).

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Limits, Rates},
    rules::Dormancy,
};
use std::{collections::BTreeMap, sync::Mutex};

fn list(key: &str, default: &str) -> Vec<u64> {
    std::env::var(key).unwrap_or_else(|_| default.into()).split(',').filter_map(|v| v.trim().parse().ok()).collect()
}

fn founding(seed: u64, limits: Option<Limits>, dormant: bool) -> Founding {
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
        limits,
        ..Founding::default()
    }
}

/// The lineages the grazer's feed draws on, by index: its intake's admitted sources.
fn producers(g: &cosmograph::simulation::Genesis) -> Vec<usize> {
    use cosmograph::rules::FeedingSources;
    match g.rules.processes["ecology:feed-1"].port_feeding.as_ref().map(|f| &f.sources) {
        Some(FeedingSources::Only(lines)) => lines.iter().filter_map(|l| lineage(l)).collect(),
        _ => vec![0],
    }
}

fn lineage(key: &str) -> Option<usize> {
    key.strip_prefix("lineage:").and_then(|i| i.parse().ok())
}

/// The first tick after the peak at which `series` is under a quarter of it.
fn crash(series: &[u64]) -> Option<usize> {
    let (at, &peak) = series.iter().enumerate().max_by_key(|(i, v)| (**v, std::cmp::Reverse(*i)))?;
    (at..series.len()).find(|&t| series[t] * 4 < peak).map(|t| t + 1)
}

struct Sample {
    tick: u64,
    plants: u64,
    heads: f64,
    matter: f64,
    demand: u64,
    grazers: u64,
    births: u64,
}

fn sample(s: &Session, tick: u64, prey: &[usize]) -> Sample {
    let state = s.sim.state();
    let mut at_site: BTreeMap<u64, (u64, u64)> = BTreeMap::new(); // site -> (grazers, first grazer id)
    let (mut grazers, mut births) = (0, 0);
    for (&id, g) in state.population.groups.iter().filter(|(_, g)| g.entity.alive && lineage(&g.entity.lineage) == Some(1)) {
        grazers += g.count;
        if g.entity.born + 10 > tick && g.entity.born <= tick && g.entity.born > 0 {
            births += g.count;
        }
        if let Some(site) = g.entity.resident_site() {
            at_site.entry(site).or_insert((0, id)).0 += g.count;
        }
    }
    let (mut plants, mut matter, mut there) = (0, 0u128, 0);
    for (&site, &(n, id)) in &at_site {
        there += n;
        plants += state.population.groups.values()
            .filter(|g| g.entity.alive && lineage(&g.entity.lineage).is_some_and(|i| prey.contains(&i)) && g.entity.resident_site() == Some(site))
            .map(|g| g.count).sum::<u64>();
        matter += u128::from(s.sim.edible_matter(id, "ecology:feed-1").unwrap());
    }
    let per = |x: f64| if there == 0 { 0.0 } else { x / there as f64 };
    Sample {
        tick, plants, grazers, births,
        heads: per(plants as f64),
        matter: per(matter as f64),
        demand: s.sim.feed_demand("ecology:feed-1", &"lineage:1".to_owned()).unwrap(),
    }
}

#[test]
#[ignore = "a measurement; prints a trace"]
fn stocked_trace() {
    let seed = list("STOCK_TRACE_SEED", "9")[0];
    let upto = list("STOCK_TRACE_TICKS", "250")[0];
    let full = list("STOCK_TICKS", "600")[0];
    let arm = std::env::var("STOCK_TRACE_ARM").unwrap_or_else(|_| "none".into());
    let limits = arms().into_iter().find(|(n, _)| *n == arm).expect("a named arm").1;
    let g = founding(seed, limits, false).generate().unwrap();
    let prey = producers(&g);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let (mut rows, mut plants, mut grazers) = (vec![], vec![], vec![]);
    for t in 1..=full {
        s.advance(1).unwrap();
        let mut n = [0u64; 4];
        let mut eaten = 0;
        for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
            if let Some(i) = lineage(&g.entity.lineage) {
                if i < 4 { n[i] += g.count; }
                if prey.contains(&i) { eaten += g.count; }
            }
        }
        plants.push(eaten);
        grazers.push(n[1]);
        if t % 10 == 0 && t <= upto {
            rows.push(sample(&s, t, &prey));
        }
    }
    println!("TRACE seed={seed} arm={arm} tick plants@grazer-sites heads/grazer matter/grazer demand matter/demand grazers births(10)");
    for r in &rows {
        let periods = if r.demand == 0 { 0.0 } else { r.matter / r.demand as f64 };
        println!("TRACE t={:>3} plants={:>5} heads/g={:>7.2} matter/g={:>7.2} demand={} periods={:>6.2} grazers={:>4} births={}",
            r.tick, r.plants, r.heads, r.matter, r.demand, periods, r.grazers, r.births);
    }
    let (pc, gc) = (crash(&plants), crash(&grazers));
    println!("TRACE crash producers={pc:?} grazers={gc:?} peak producers {prey:?} = {} grazers={}",
        plants.iter().max().unwrap(), grazers.iter().max().unwrap());
    for k in [4.0, 2.0, 1.0] {
        // The first sample under `k` periods after one at or above it: the
        // founding's bare ground is not a decline.
        let mut risen = false;
        let trip = rows.iter().find(|r| {
            let periods = if r.demand == 0 { 0.0 } else { r.matter / r.demand as f64 };
            risen |= periods >= k;
            risen && r.grazers > 0 && periods < k
        }).map(|r| r.tick);
        let tripped = |r: &&Sample| trip.is_some_and(|t| r.tick >= t) && r.matter / (r.demand.max(1) as f64) < k;
        let well = |heads: f64| rows.iter().filter(tripped).filter(|r| r.heads >= heads).count();
        let lead = |c: Option<usize>| trip.zip(c).map(|(t, c)| c as i64 - t as i64);
        println!("TRACE below {k} periods: first tick {trip:?}; samples at or after it with >=2 holders per head: {}, >=4: {}; ticks before producer crash {:?}, before grazer crash {:?}",
            well(2.0), well(4.0), lead(pc), lead(gc));
    }
}

/// The arms: a name and its limits, none for no limits at all.
fn arms() -> Vec<(&'static str, Option<Limits>)> {
    let off = |k| Limits { full: u64::MAX, ceiling: [u64::MAX / 2, 1], stocked: Some(k) };
    let on = |k| Limits { stocked: Some(k), ..Limits::default() };
    vec![
        ("none", None),
        ("ceiling", Some(Limits::default())),
        ("stocked2", Some(on(2))),
        ("stocked4", Some(on(4))),
        ("stocked8", Some(on(8))),
        ("alone2", Some(off(2))),
        ("alone4", Some(off(4))),
        ("alone8", Some(off(8))),
    ]
}

fn arm_run(name: &str, f: Founding, ticks: u64) -> String {
    let g = f.generate().unwrap();
    let prey = producers(&g);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let (mut at, mut grazers, mut born, mut tissue, mut heads) = (vec![], vec![], vec![], vec![], vec![]);
    let marks = [100, 200, 300, 450, 600];
    for t in 1..=ticks {
        s.advance(1).unwrap();
        let mut n = [0u64; 4];
        let (mut fresh, mut matter, mut standing) = (0u64, 0u128, 0u64);
        for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
            let Some(i) = lineage(&g.entity.lineage).filter(|i| *i < 4) else { continue };
            n[i] += g.count;
            if prey.contains(&i) {
                standing += g.count;
            }
            if i == 1 && g.entity.born == t {
                fresh += g.count;
            }
            if prey.contains(&i) && t % 10 == 0 {
                matter += u128::from(cosmograph::anatomy::books(&g.entity).get(&format!("matter:{i}-0")).copied().unwrap_or(0)) * u128::from(g.count);
            }
        }
        if marks.contains(&t) || t == ticks {
            at.push(n);
        }
        grazers.push(n[1]);
        heads.push(standing);
        born.push(fresh);
        if t % 10 == 0 {
            tissue.push((t, matter));
        }
    }
    let peak = grazers.iter().max().copied().unwrap_or(0);
    let gc = crash(&grazers);
    let horizon = gc.unwrap_or(ticks as usize);
    let last_birth = born[..horizon.min(born.len())].iter().rposition(|b| *b > 0).map(|i| i + 1);
    let top = tissue.iter().max_by_key(|(t, m)| (*m, std::cmp::Reverse(*t))).map(|(t, _)| *t).unwrap_or(0);
    let bottom = tissue.iter().filter(|(t, _)| *t >= top).min_by_key(|(t, m)| (*m, *t)).map(|(t, m)| (*t, *m));
    let stops = match (last_birth, bottom) {
        (Some(b), Some((t, _))) => format!("{} (breeding ends {b}, producer tissue bottoms {t})", if (b as u64) < t { "before" } else { "not before" }),
        _ => "n/a".into(),
    };
    format!("{name} peak={peak} plant_crash={:?} grazer_crash={gc:?} last_birth={last_birth:?} counts@100/200/300/450/end={at:?} stops_breeding={stops}", crash(&heads))
}

#[test]
#[ignore = "a measurement; prints a table"]
fn stocked_arms() {
    let ticks = list("STOCK_TICKS", "600")[0];
    let seeds = list("STOCK_SEEDS", "3,5,9,11,17");
    let only = std::env::var("STOCK_ARMS").ok().map(|v| v.split(',').map(str::to_owned).collect::<Vec<_>>());
    let mut jobs = vec![];
    for dormant in list("STOCK_DORMANT", "0") {
        for (name, limits) in arms().into_iter().filter(|(n, _)| only.as_ref().is_none_or(|o| o.iter().any(|x| x == n))) {
            for &seed in &seeds {
                jobs.push((dormant == 1, name, limits.clone(), seed));
            }
        }
    }
    let queue = Mutex::new(jobs);
    let lines = Mutex::new(vec![]);
    std::thread::scope(|scope| {
        for _ in 0..list("STOCK_WORKERS", "8")[0] {
            scope.spawn(|| {
                loop {
                    let job = queue.lock().unwrap().pop();
                    let Some((dormant, name, limits, seed)) = job else { break };
                    let line = arm_run(name, founding(seed, limits, dormant), ticks);
                    lines.lock().unwrap().push((dormant, seed, name, format!("ARM dorm={} seed={seed} {line}", u8::from(dormant))));
                }
            });
        }
    });
    let mut lines = lines.into_inner().unwrap();
    let order: Vec<_> = arms().into_iter().map(|(n, _)| n).collect();
    lines.sort_by_key(|(d, s, n, _)| (*d, *s, order.iter().position(|o| o == n)));
    for (.., line) in lines {
        println!("{line}");
    }
}
