// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 113's check of ruling 115's competing instance (feeding when food
//! is short): seeded draws of probe worlds, each run four ways, compared
//! reading by reading. Clocks live here, in the host; the sim reads none.
//! Worlds and dynamics seeds follow from one master seed, chosen from the
//! system clock before the run unless `--seed` is given. `--density` runs
//! only the exact and crowd arms, to measure savings without a verdict;
//! `--water` has every world contest water as well as food; `--predators`
//! adds a lineage hunting the others by ruling 287's weighted draw;
//! `--approximate` adds a fifth arm, the crowd with ruling 220's approximate
//! pairing draw, checked against the exact runner and against the exact
//! crowd. `--bodies` draws checkpoint 6's worlds of bodies instead (ruling
//! 262), where the draw control runs as it does with hunters.

mod report;

use isocosm::probe::{
    BodyFounding, Crowd, PredatorFounding, ProbeFounding, ProbeWorld, REFUSED, Variant,
    check::{self, Settings},
    readings::{self, Reading},
    run_exact,
};
use report::*;
use std::time::Instant;

fn main() {
    if let Err(why) = run() {
        eprintln!("{why}");
        std::process::exit(1);
    }
}

fn arm(
    world: &ProbeWorld,
    derived: &[Reading],
    index: usize,
    dynamics: u64,
) -> Result<Arm, String> {
    let start = Instant::now();
    let inspect = isocosm::draw(dynamics, "probe-inspect", &[]);
    let finish = |micros,
                  members: &[(&isocosm::schema::Entity, u64)],
                  work: &isocosm::simulation::Work,
                  stored,
                  shortfalls,
                  values| {
        let (alive, alive_states) = readings::alive_states(members);
        Arm {
            arm: ARMS[index],
            dynamics,
            micros,
            evaluations: work.evaluations,
            represented: work.represented,
            accepted: work.accepted,
            blocked: work.blocked,
            stored,
            alive,
            alive_states,
            shortfalls,
            refused: None,
            readings: values,
        }
    };
    if index < 2 {
        let run = run_exact(world, dynamics, true)?;
        let micros = start.elapsed().as_micros() as u64;
        let members = readings::exact_members(&run);
        let s = run.sim.state();
        let inspected = readings::inspect(&members, inspect);
        let values = readings::evaluate(
            derived, world, &members, &s.sites, s.tick, inspected, &run.meals,
        )?;
        let stored = s.population.groups.len() - s.sites.len();
        Ok(finish(micros, &members, &run.work, stored, 0, values))
    } else {
        let variant = match index {
            2 => Variant::Histogram,
            3 => Variant::Averaged,
            4 => Variant::Approximate,
            _ => Variant::Unweighted,
        };
        let crowd = Crowd::new(world, dynamics, variant)?.run()?;
        let micros = start.elapsed().as_micros() as u64;
        let members = readings::crowd_members(&crowd);
        let inspected = readings::inspect(&members, inspect);
        let values = readings::evaluate(
            derived,
            world,
            &members,
            &crowd.sites,
            crowd.tick,
            inspected,
            &crowd.meals,
        )?;
        let shortfalls = crowd.shortfalls;
        Ok(finish(
            micros,
            &members,
            &crowd.work,
            members.len(),
            shortfalls,
            values,
        ))
    }
}

struct Options {
    master: u64,
    draws: u64,
    permutations: u64,
    output: Option<String>,
    project: Option<u64>,
    members: Option<[u64; 2]>,
    density: bool,
    crowds: bool,
    water: bool,
    predators: bool,
    approximate: bool,
    bodies: bool,
}

fn options() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let mut o = Options {
        master: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos() as u64,
        draws: 64,
        permutations: 1999,
        output: None,
        project: None,
        members: None,
        density: false,
        crowds: false,
        water: false,
        predators: false,
        approximate: false,
        bodies: false,
    };
    while let Some(arg) = args.next() {
        let mut number = || -> Result<u64, String> {
            let v = args.next().ok_or(format!("{arg} needs a value"))?;
            v.parse().map_err(|_| format!("{arg} needs an integer"))
        };
        match arg.as_str() {
            "--seed" => o.master = number()?,
            "--draws" => o.draws = number()?,
            "--permutations" => o.permutations = number()?,
            // Print the time this run's pace projects for that many draws.
            "--project" => o.project = Some(number()?),
            "--members" => o.members = Some([number()?, number()?]),
            "--density" => o.density = true,
            // The crowd arms alone, exact and approximate, for their times
            // at densities the exact runner is too slow to reach often.
            "--crowds" => o.crowds = true,
            // Contest water as well as food: two competitions a tick.
            "--water" => o.water = true,
            // A lineage that hunts the others: ruling 287's weighted draw.
            "--predators" => o.predators = true,
            "--approximate" => o.approximate = true,
            // Checkpoint 6's worlds of bodies (ruling 262).
            "--bodies" => o.bodies = true,
            "--output" => o.output = Some(args.next().ok_or("--output needs a path")?),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    Ok(o)
}

fn run() -> Result<(), String> {
    let o = options()?;
    let mut probe = ProbeFounding {
        water: o.water,
        predators: o.predators.then(PredatorFounding::default),
        ..ProbeFounding::default()
    };
    if let Some(members) = o.members {
        probe.members = members;
    }
    // With bodies, `--members` sets the producers per site and a quarter
    // as many grazers.
    let mut bodies = BodyFounding::default();
    if let Some([low, high]) = o.members {
        bodies.producers = [low, high];
        bodies.grazers = [(low / 4).max(1), (high / 4).max(1)];
    }
    let domain = match o.bodies {
        true => Domain::Bodies(Box::new(bodies)),
        false => Domain::Probe(Box::new(probe)),
    };
    let mut arms: Vec<usize> = if o.crowds {
        vec![2]
    } else if o.density {
        vec![0, 2]
    } else {
        vec![0, 1, 2, 3]
    };
    if o.approximate || o.crowds {
        arms.push(4);
    }
    // The draw's control runs beside a verdict, wherever there are hunters.
    let draw_control = (o.predators || o.bodies) && !o.density && !o.crowds;
    if draw_control {
        arms.push(5);
    }
    eprintln!(
        "Probe receipt: master seed {}, {} draws, arms {arms:?}",
        o.master, o.draws
    );
    let settings = Settings {
        alpha: 0.05,
        permutations: o.permutations,
        seed: isocosm::draw(o.master, "probe-check", &[]),
    };
    let began = Instant::now();
    // A draw an arm refused has no row there.
    let mut rows: [Vec<Option<Vec<u64>>>; 6] = Default::default();
    let mut infos: Option<Vec<ReadingInfo>> = None;
    let mut read_set = None;
    let mut out: Vec<Draw> = Vec::new();
    let mut checks = Checks {
        exact_rerun_identical: false,
        collect_changes_no_outcome: false,
    };
    for k in 0..o.draws {
        let seed = isocosm::draw(o.master, "probe-world", &[k]);
        let world = domain.generate(seed)?;
        let derived = readings::derive(&world);
        let similitude = world.similitude()?;
        let these: Vec<ReadingInfo> = derived
            .iter()
            .map(|r| ReadingInfo::new(r, similitude.bound(&r.key)))
            .collect();
        let keys = |v: &[ReadingInfo]| {
            v.iter()
                .map(|r| (r.key.clone(), r.bound_per_mille))
                .collect::<Vec<_>>()
        };
        match &infos {
            None => infos = Some(these),
            Some(first) if keys(first) != keys(&these) => {
                let (a, b) = (keys(first), keys(&these));
                let only = |x: &[(String, u32)], y: &[(String, u32)]| {
                    x.iter()
                        .filter(|r| !y.contains(r))
                        .map(|r| r.0.clone())
                        .collect::<Vec<_>>()
                };
                return Err(format!(
                    "draw {k} reads differently from draw 0: only in draw 0 {:?}, only in draw {k} {:?}",
                    only(&a, &b),
                    only(&b, &a)
                ));
            },
            Some(_) => {},
        }
        read_set.get_or_insert_with(|| readings::read_set(&world));
        let mut draw = Draw::new(k, seed, &world)?;
        for &index in &arms {
            let dynamics = isocosm::draw(o.master, "probe-dynamics", &[k, index as u64]);
            let a = match arm(&world, &derived, index, dynamics) {
                Ok(a) => a,
                Err(why) if why.contains(REFUSED) => Arm::refused(ARMS[index], dynamics, why),
                Err(why) => return Err(format!("draw {k}, {}: {why}", ARMS[index])),
            };
            rows[index].push(a.refused.is_none().then(|| a.readings.clone()));
            draw.arms.push(a);
        }
        if k == 0 && !o.crowds {
            let dynamics = draw.arms[0].dynamics;
            let reference = run_exact(&world, dynamics, true)?.sim.state_hash();
            checks.exact_rerun_identical =
                reference == run_exact(&world, dynamics, true)?.sim.state_hash();
            checks.collect_changes_no_outcome =
                reference == run_exact(&world, dynamics, false)?.sim.state_hash();
        }
        out.push(draw);
        if (k + 1) % 25 == 0 || k + 1 == o.draws {
            let spent = began.elapsed().as_secs_f64();
            eprintln!(
                "{} draws in {spent:.0} s; {:.2} s per draw",
                k + 1,
                spent / (k + 1) as f64
            );
        }
    }
    if let Some(n) = o.project {
        let per_draw = began.elapsed().as_secs_f64() / o.draws.max(1) as f64;
        let seconds = per_draw * n as f64;
        eprintln!(
            "At this pace {n} draws take {seconds:.0} s ({:.2} h)",
            seconds / 3600.0
        );
    }
    let infos = infos.ok_or("no draws")?;
    let bounds: Vec<(String, u32)> = infos
        .iter()
        .map(|r| (r.key.clone(), r.bound_per_mille))
        .collect();
    let starvation: Vec<String> = infos
        .iter()
        .filter(|r| r.starvation)
        .map(|r| r.key.clone())
        .collect();
    let [exact, rerun, crowd, averaged, approximate, unweighted] = rows;
    // Each comparison reads the draws both its arms ran to the end. A crowd
    // under certification may refuse at most one draw in a hundred; the
    // controls' refusals are only recorded.
    let compare = |name: &str, a: &[Option<Vec<u64>>], b: &[Option<Vec<u64>>], bounded| {
        check::compare_runs(name, &bounds, a, b, settings, bounded)
    };
    let (certified, control) = (true, false);
    let mut comparisons = Vec::new();
    let mut verdicts = None;
    if !o.density && !o.crowds {
        comparisons = vec![
            compare("exact against crowd", &exact, &crowd, certified),
            compare(
                "exact against exact (positive control)",
                &exact,
                &rerun,
                control,
            ),
            compare(
                "exact against averaged crowd (negative control)",
                &exact,
                &averaged,
                control,
            ),
        ];
        if draw_control {
            comparisons.push(compare(
                "exact against unweighted crowd (draw control)",
                &exact,
                &unweighted,
                control,
            ));
        }
        verdicts = Some(Verdicts::new(&comparisons, starvation));
    }
    if o.approximate && !o.crowds {
        comparisons.push(compare(
            "exact against approximate crowd",
            &exact,
            &approximate,
            certified,
        ));
    }
    if o.approximate || o.crowds {
        comparisons.push(compare(
            "crowd against approximate crowd",
            &crowd,
            &approximate,
            certified,
        ));
    }
    // Savings and density come from the draws no arm refused.
    let whole: Vec<&Draw> = out
        .iter()
        .filter(|d| d.arms.iter().all(|a| a.refused.is_none()))
        .collect();
    let savings = Savings::new(&whole);
    let density = Density::new(&whole);
    let mut refused: std::collections::BTreeMap<&'static str, Vec<u64>> = Default::default();
    for d in &out {
        for a in d.arms.iter().filter(|a| a.refused.is_some()) {
            refused.entry(a.arm).or_default().push(d.k);
        }
    }
    if !refused.is_empty() {
        eprintln!("refused draws by arm: {refused:?}");
    }
    eprintln!(
        "{}; evaluations {:.1}x fewer",
        verdicts
            .as_ref()
            .map_or("no verdict".into(), |v| v.summary()),
        savings.evaluation_ratio
    );
    let receipt = Receipt {
        version: isocosm::VERSION,
        kind: if o.crowds {
            "isocosm-probe-crowds"
        } else if o.density {
            "isocosm-probe-density"
        } else {
            "isocosm-probe"
        },
        master_seed: o.master,
        arms: arms.iter().map(|&i| ARMS[i]).collect(),
        domain,
        settings,
        read_set: read_set.unwrap_or_default(),
        readings: infos,
        verdicts,
        comparisons,
        savings,
        density,
        checks,
        refused,
        note: NOTE,
        draws: out,
    };
    let json = serde_json::to_string_pretty(&receipt).map_err(|e| e.to_string())?;
    match o.output {
        Some(path) => std::fs::write(path, json).map_err(|e| e.to_string())?,
        None => println!("{json}"),
    }
    Ok(())
}
