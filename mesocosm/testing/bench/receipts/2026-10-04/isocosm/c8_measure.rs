// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

// Checkpoint 8's measurements, kept as a record of method: each was run as
// an ignored test from `shared/isocosm/tests/` (`cargo test --release --test
// <file> <name> -- --ignored --nocapture`) at the state of the branch when
// the ruling it informed was asked, and then removed. `measure_domain` and
// `measure_births` informed 551; `measure_buds_alone`, `measure_coverage`
// (with SOIL, TICKS and SEEDS set) and `measure_matter` informed 552 and 553
// and the run's 60 ticks (517); `measure_bonds` informed 554. Against the
// merged code their numbers differ, since those rulings changed the domain.


use isocosm::{Execution, Simulation, anatomy, probe::BodyFounding, rules::Measure, schema::*};
use std::collections::BTreeMap;

fn three_quarter(m: u64) -> u64 {
    let m = u128::from(m.max(1));
    (m * m.isqrt()).isqrt() as u64
}

/// For each founded body over many worlds: grazers' store bound against a
/// hundred ticks of rent at full tissue; producers' income, provision and
/// how many ticks of income their tissue gap and provision take.
#[test]
#[ignore]
fn measure_domain() {
    let b = isocosm::rules::BodyRules::default();
    let mut grazer_ratio = vec![];
    let mut producer = vec![];
    for seed in 0..300 {
        let g = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap()
        .genesis;
        for c in g.population.groups.values() {
            let e = &c.entity;
            let living = || e.parts.values().filter(|p| !p.severed);
            let cells = |p: &Part, f: &str| u64::from(p.cells.get(f).copied().unwrap_or(0));
            let ceiling: u64 = living().map(|p| anatomy::ceiling(p, b)).sum();
            let span: u64 = living()
                .filter(|p| p.functions.contains("function:contract"))
                .map(|p| {
                    p.half_extent
                        .iter()
                        .map(|h| h.unsigned_abs())
                        .max()
                        .unwrap() as u64
                })
                .sum();
            let upkeep =
                1 + three_quarter(ceiling) * (ceiling + span * 100) / (62 * ceiling.max(1));
            let store: u64 = living()
                .map(|p| cells(p, "function:store") * anatomy::cell_mass(p, b))
                .sum();
            let provision: u64 = living()
                .map(|p| cells(p, "function:reproduce") * anatomy::cell_mass(p, b))
                .sum();
            match e.lineage.as_str() {
                "lineage:1" => grazer_ratio.push((
                    store * 100 / (upkeep * 100),
                    store,
                    upkeep,
                    ceiling,
                    provision,
                )),
                "lineage:0" => {
                    let area: u128 = living()
                        .map(|p| anatomy::share_of(p, "function:fix", Measure::Area))
                        .sum();
                    let income = ((area * 11 / 144) as u64).max(1);
                    let tissue = anatomy::held(e, &g.rules, "tissue:0");
                    producer.push((income, upkeep, ceiling, provision, ceiling - tissue));
                },
                _ => {},
            }
        }
    }
    grazer_ratio.sort();
    let n = grazer_ratio.len();
    eprintln!(
        "grazers {n}: store as percent of 100 ticks' rent, min {:?} median {:?} max {:?}",
        grazer_ratio[0],
        grazer_ratio[n / 2],
        grazer_ratio[n - 1]
    );
    let fed = grazer_ratio.iter().filter(|r| r.1 >= r.2 * 100).count();
    eprintln!("grazers whose store can hold 100 ticks' rent: {fed} of {n}");
    let mut net: Vec<(i64, u64, u64, u64)> = producer
        .iter()
        .map(|p| (p.0 as i64 - p.1 as i64, p.3, p.4, p.2))
        .collect();
    net.sort();
    let m = net.len();
    eprintln!(
        "producers {m}: (income - rent, provision, gap, ceiling) min {:?} median {:?} max {:?}",
        net[0],
        net[m / 2],
        net[m - 1]
    );
    let ticks: Vec<i64> = net
        .iter()
        .filter(|x| x.0 > 0)
        .map(|x| ((x.1 + x.2) as i64) / x.0)
        .collect();
    let mut t = ticks.clone();
    t.sort();
    eprintln!(
        "producers paying their rent: {} of {m}; ticks to fill gap and provision alone: median {:?} max {:?}",
        t.len(),
        t.get(t.len() / 2),
        t.last()
    );
}

#[test]
#[ignore]
fn measure_births() {
    let watched = [
        "body:bud-0",
        "body:bud-once-0",
        "body:bear-1",
        "body:bear-once-1",
        "body:nurse-1",
        "body:graze-1",
        "body:starve-1",
    ];
    for seed in 0..12 {
        let w = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let g = w.genesis;
        let start = std::time::Instant::now();
        let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        for p in watched {
            sim.watch(p);
        }
        let mut first: BTreeMap<String, u64> = BTreeMap::new();
        let mut total: BTreeMap<String, u64> = BTreeMap::new();
        let founders = sim.state().population.count();
        let mut line = String::new();
        for t in 1..=400u64 {
            if let Err(e) = sim.advance(1) {
                line = format!("error at {t}: {e}");
                break;
            }
            for a in sim.take_watched() {
                first.entry(a.process.clone()).or_insert(a.tick);
                *total.entry(a.process).or_default() += a.count;
            }
            if [24, 48, 100, 200, 400].contains(&t) {
                let pop = sim.state().population.count();
                let alive: u64 = sim
                    .state()
                    .population
                    .groups
                    .values()
                    .filter(|c| c.entity.alive)
                    .map(|c| c.count)
                    .sum();
                eprintln!("seed {seed} t{t}: population {pop} alive {alive}");
            }
        }
        eprintln!(
            "seed {seed}: founders {founders} {:?} first {first:?} totals {total:?} {line}",
            start.elapsed()
        );
    }
}

#[test]
#[ignore]
fn measure_buds_alone() {
    for (grazers, label) in [([0u64, 0u64], "no grazers"), ([4, 16], "grazers")] {
        for seed in 0..6 {
            let w = BodyFounding {
                seed,
                grazers,
                ..Default::default()
            }
            .generate()
            .unwrap();
            let g = w.genesis;
            let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
            for p in [
                "body:bud-0",
                "body:bud-once-0",
                "body:starve-0",
                "body:fix-0",
            ] {
                sim.watch(p);
            }
            let mut first: BTreeMap<String, u64> = BTreeMap::new();
            let mut total: BTreeMap<String, u64> = BTreeMap::new();
            for t in 1..=200u64 {
                sim.advance(1).unwrap();
                for a in sim.take_watched() {
                    first.entry(a.process.clone()).or_insert(a.tick);
                    *total.entry(a.process).or_default() += a.count;
                }
                if t == 30 || t == 100 {
                    let rules = &sim.genesis().rules;
                    let b = rules.body();
                    let mut line = vec![];
                    for c in sim
                        .state()
                        .population
                        .groups
                        .values()
                        .filter(|c| c.entity.lineage == "lineage:0" && c.entity.alive)
                    {
                        let e = &c.entity;
                        let ceiling: u64 = e
                            .parts
                            .values()
                            .filter(|p| !p.severed)
                            .map(|p| anatomy::ceiling(p, b))
                            .sum();
                        let t0 = anatomy::held(e, rules, "tissue:0");
                        let p0 = anatomy::held(e, rules, "provision:0");
                        line.push(format!("{}x{t0}/{ceiling}+{p0}", c.count));
                    }
                    let soil: u64 = sim
                        .state()
                        .sites
                        .values()
                        .map(|s| s.accounts.get("world:soil").copied().unwrap_or(0))
                        .sum();
                    eprintln!("{label} seed {seed} t{t} soil {soil}: {}", line.join(" "));
                }
            }
            eprintln!("{label} seed {seed}: first {first:?} totals {total:?}");
        }
    }
}

#[test]
#[ignore]
fn measure_coverage() {
    let ticks: u64 = std::env::var("TICKS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(60);
    let mut worlds: BTreeMap<String, u64> = BTreeMap::new();
    let mut first: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut times = vec![];
    let env = |k: &str, d: u64| {
        std::env::var(k)
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or(d)
    };
    let (seeds, grazers, soil) = (env("SEEDS", 100), env("GRAZERS", 16), env("SOIL", 2400));
    for seed in 0..seeds {
        let w = BodyFounding {
            seed,
            grazers: [grazers.min(4), grazers],
            soil: [200, soil],
            ..Default::default()
        }
        .generate()
        .unwrap();
        let g = w.genesis;
        let start = std::time::Instant::now();
        let mut sim = Simulation::new(g.clone(), Execution::Individuals).unwrap();
        for p in g.rules.processes.keys() {
            sim.watch(p);
        }
        sim.advance(ticks).unwrap();
        times.push(start.elapsed().as_millis());
        let mut seen: BTreeMap<String, u64> = BTreeMap::new();
        for a in sim.take_watched() {
            seen.entry(a.process).or_insert(a.tick);
        }
        for c in sim.state().population.groups.values() {
            let e = &c.entity;
            if e.born > 0 {
                let kind = match (e.lineage.as_str(), e.soma.len()) {
                    ("lineage:0", _) => "born:bud",
                    (_, _) if e.parts.len() == 1 => "born:egg",
                    _ => "born:brood",
                };
                seen.entry(kind.into()).or_insert(e.born);
            }
            if e.lineage == "lineage:1"
                && e.parts.values().any(|p| p.matter.contains_key("tissue:0"))
            {
                seen.entry("whole".into()).or_insert(0);
            }
        }
        for (k, t) in seen {
            *worlds.entry(k.clone()).or_default() += 1;
            first.entry(k).or_default().push(t);
        }
    }
    times.sort();
    let n = times.len();
    eprintln!(
        "ticks {ticks}: ms per draw median {} max {}",
        times[n / 2],
        times[n - 1]
    );
    for (k, n) in &worlds {
        let mut f = first[k].clone();
        f.sort();
        eprintln!(
            "{k}: {n} worlds of {seeds}, first tick median {} max {}",
            f[f.len() / 2],
            f[f.len() - 1]
        );
    }
}

#[test]
#[ignore]
fn measure_matter() {
    let b = isocosm::rules::BodyRules::default();
    let mut ratios = vec![];
    for seed in 0..300 {
        let g = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap()
        .genesis;
        let soil: u64 = g
            .sites
            .values()
            .map(|s| s.accounts.get("world:soil").copied().unwrap_or(0))
            .sum();
        let (mut held, mut capacity) = (0u64, 0u64);
        for c in g.population.groups.values() {
            let e = &c.entity;
            if e.lineage == "world:ground" {
                continue;
            }
            let m: u64 = e.parts.values().flat_map(|p| p.matter.values()).sum();
            held += m * c.count;
            if e.lineage == "lineage:0" {
                let ceiling: u64 = e.parts.values().map(|p| anatomy::ceiling(p, b)).sum();
                capacity += ceiling * c.count;
            }
        }
        ratios.push(((soil + held) * 100 / capacity.max(1), soil, held, capacity));
    }
    ratios.sort();
    let n = ratios.len();
    let full = ratios.iter().filter(|r| r.0 >= 100).count();
    eprintln!(
        "matter as percent of producers' ceilings: min {:?} q1 {:?} median {:?} q3 {:?} max {:?}; at least 100%: {full} of {n}",
        ratios[0],
        ratios[n / 4],
        ratios[n / 2],
        ratios[3 * n / 4],
        ratios[n - 1]
    );
}

/// Ruling 554's evidence: of living grazers at the end of the run, the
/// share in a bond with a still-hungry young, and the milk that flowed.
#[test]
#[ignore]
fn measure_bonds() {
    let mut fractions = vec![];
    let (mut nursed, mut grazers_end) = (0u64, 0u64);
    for seed in 0..100 {
        let w = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut sim = Simulation::new(w.genesis.clone(), Execution::Individuals).unwrap();
        sim.watch("body:nurse-1");
        sim.advance(w.ticks).unwrap();
        nursed += sim.take_watched().iter().map(|a| a.count).sum::<u64>();
        let state = sim.state();
        let rules = &sim.genesis().rules;
        let alive = |id: Id| state.population.get(id).is_some_and(|e| e.alive);
        let hungry = |id: Id| {
            state.population.get(id).is_some_and(|e| e.alive && anatomy::room(e, rules, "reserve:1") > 0)
        };
        let mut grazers = vec![];
        for (&first, c) in &state.population.groups {
            if c.entity.lineage == "lineage:1" && c.entity.alive {
                grazers.extend(first..first + c.count);
            }
        }
        let bonded = grazers
            .iter()
            .filter(|&&g| {
                state.relations.iter().any(|r| {
                    (r.kind == "sim:child" && r.subject == g && hungry(r.object))
                        || (r.kind == "sim:parent" && r.subject == g && alive(r.object) && hungry(g))
                })
            })
            .count();
        grazers_end += grazers.len() as u64;
        if !grazers.is_empty() {
            fractions.push(bonded * 100 / grazers.len());
        }
    }
    fractions.sort();
    let n = fractions.len();
    eprintln!("alive grazers {grazers_end}; percent in a bond with a hungry young: q1 {} median {} q3 {}; milk acts {nursed}", fractions[n / 4], fractions[n / 2], fractions[3 * n / 4]);
}
