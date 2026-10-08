// Temporary: checkpoint 9's evidence, the probe's cells and asks.
use isocosm::{anatomy, probe::BodyFounding, rules::Measure, schema::*};

#[test]
#[ignore]
fn cells_and_asks() {
    let cap = |h: [i32; 3]| {
        anatomy::capacity(&Part {
            half_extent: h,
            ..Default::default()
        })
    };
    for (name, h) in [
        ("frond 3x3", [3, 3, 1]),
        ("frond 3x4", [3, 4, 1]),
        ("frond 4x4", [4, 4, 1]),
        ("lump", [2, 2, 2]),
        ("limb 3", [3, 1, 1]),
        ("limb 4", [4, 1, 1]),
        ("eye", [1, 1, 1]),
    ] {
        eprintln!("{name}: {} cells", cap(h));
    }
    let (mut income, mut mouthful, mut parts) = (vec![], vec![], vec![]);
    let b = isocosm::rules::BodyRules::default();
    for seed in 1..=300 {
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
            let ceiling: u64 = living().map(|p| anatomy::ceiling(p, b)).sum();
            match e.lineage.as_str() {
                "lineage:0" => {
                    let area: u128 = living()
                        .map(|p| anatomy::share_of(p, "function:fix", Measure::Area))
                        .sum();
                    income.push(((area * 11 / 144) as u64).max(1));
                },
                "lineage:1" => {
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
                    let volume: u128 = living()
                        .map(|p| anatomy::share_of(p, "function:intake", Measure::Volume))
                        .sum();
                    let priced = u128::from(ceiling + span * 100);
                    mouthful.push(
                        ((12 * volume * priced / (269 * u128::from(ceiling.max(1)))) as u64).max(1),
                    );
                },
                _ => continue,
            }
            parts.push(e.parts.len());
        }
    }
    let q = |v: &mut Vec<u64>| {
        v.sort();
        let n = v.len();
        (v[0], v[n / 2], v[n * 9 / 10], v[n - 1])
    };
    eprintln!(
        "income per tick (min, median, p90, max): {:?}",
        q(&mut income)
    );
    eprintln!(
        "mouthful per tick (min, median, p90, max): {:?}",
        q(&mut mouthful)
    );
    parts.sort();
    eprintln!(
        "parts per founded body: median {} max {}",
        parts[parts.len() / 2],
        parts[parts.len() - 1]
    );
}

/// The per-cell capacity an intact grazer needs for its mouthful to reach
/// every part by room from its root, part by part (562): the most any part
/// carries, what reaches it and the parts beyond it, over its cells.
#[test]
#[ignore]
fn capacity_needed() {
    let b = isocosm::rules::BodyRules::default();
    let mut need: Vec<(u64, u64, String)> = vec![];
    for seed in 1..=300 {
        let g = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap()
        .genesis;
        for c in g.population.groups.values() {
            let e = &c.entity;
            if e.lineage != "lineage:1" {
                continue;
            }
            let living: Vec<(&Id, &Part)> = e.parts.iter().filter(|(_, p)| !p.severed).collect();
            let ceiling: u64 = living.iter().map(|(_, p)| anatomy::ceiling(p, b)).sum();
            let span: u64 = living
                .iter()
                .filter(|(_, p)| p.functions.contains("function:contract"))
                .map(|(_, p)| {
                    p.half_extent
                        .iter()
                        .map(|h| h.unsigned_abs())
                        .max()
                        .unwrap() as u64
                })
                .sum();
            let volume: u128 = living
                .iter()
                .map(|(_, p)| anatomy::share_of(p, "function:intake", Measure::Volume))
                .sum();
            let ask = ((12 * volume * u128::from(ceiling + span * 100)
                / (269 * u128::from(ceiling.max(1)))) as u64)
                .max(1);
            // Each part's share by its adult mass, and what passes through it.
            let share = |p: &Part| ask * anatomy::ceiling(p, b) / ceiling.max(1);
            let beyond = |id: Id| -> u64 {
                let mut total = 0;
                let mut stack = vec![id];
                while let Some(x) = stack.pop() {
                    total += share(&e.parts[&x]);
                    stack.extend(
                        e.parts
                            .iter()
                            .filter(|(_, p)| p.parent == Some(x))
                            .map(|(k, _)| *k),
                    );
                }
                total
            };
            for (id, p) in &living {
                let cells: u64 = p.cells.values().map(|n| u64::from(*n)).sum();
                if cells == 0 {
                    continue;
                }
                let flow = if p.parent.is_none() {
                    ask
                } else {
                    beyond(**id)
                };
                need.push((
                    flow * 1000 / cells,
                    ask,
                    format!("{:?} {} cells", p.half_extent, cells),
                ));
            }
        }
    }
    need.sort();
    let n = need.len();
    eprintln!(
        "mg per cell a tick, x1000: median {:?} p90 {:?} p99 {:?} max {:?}",
        need[n / 2],
        need[n * 9 / 10],
        need[n * 99 / 100],
        need[n - 1]
    );
}

/// Births a world sees in its run, by kind.
#[test]
#[ignore]
fn births_per_world() {
    let mut per = vec![];
    for seed in 0..20 {
        let w = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap();
        let mut sim =
            isocosm::Simulation::new(w.genesis.clone(), isocosm::Execution::Individuals).unwrap();
        sim.advance(w.ticks).unwrap();
        let born: u64 = sim
            .state()
            .population
            .groups
            .values()
            .filter(|c| c.entity.born > 0)
            .map(|c| c.count)
            .sum();
        per.push(born);
    }
    per.sort();
    eprintln!(
        "births per 60-tick world: min {} median {} max {} total {}",
        per[0],
        per[per.len() / 2],
        per[per.len() - 1],
        per.iter().sum::<u64>()
    );
}

/// Cells and parts per founded body, by lineage.
#[test]
#[ignore]
fn cells_per_body() {
    let mut by: std::collections::BTreeMap<String, (Vec<u64>, Vec<u64>)> = Default::default();
    for seed in 1..=300 {
        let g = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap()
        .genesis;
        for c in g.population.groups.values() {
            let e = &c.entity;
            let cells: u64 = e
                .parts
                .values()
                .flat_map(|p| p.cells.values())
                .map(|n| u64::from(*n))
                .sum();
            let v = by.entry(e.lineage.clone()).or_default();
            v.0.push(cells);
            v.1.push(e.parts.len() as u64);
        }
    }
    for (l, (mut c, mut p)) in by {
        c.sort();
        p.sort();
        let n = c.len();
        eprintln!(
            "{l}: n {n} cells min {} median {} p90 {} max {}; parts min {} median {} max {}",
            c[0],
            c[n / 2],
            c[n * 9 / 10],
            c[n - 1],
            p[0],
            p[n / 2],
            p[n - 1]
        );
    }
}

/// Founded grazers with no contracting part, and producers with a gland.
#[test]
#[ignore]
fn limbless_and_glands() {
    let (mut grazers, mut limbless, mut worlds, mut worlds_limbless) = (0, 0, 0, 0);
    for seed in 1..=300 {
        let g = BodyFounding {
            seed,
            ..Default::default()
        }
        .generate()
        .unwrap()
        .genesis;
        let mut any = false;
        worlds += 1;
        for c in g.population.groups.values() {
            let e = &c.entity;
            if e.lineage != "lineage:1" {
                continue;
            }
            grazers += c.count;
            if !e
                .parts
                .values()
                .any(|p| p.functions.contains("function:contract"))
            {
                limbless += c.count;
                any = true;
            }
        }
        worlds_limbless += any as u32;
    }
    eprintln!(
        "founded grazers {grazers}, limbless {limbless}; worlds {worlds}, with a limbless grazer {worlds_limbless}"
    );
}
