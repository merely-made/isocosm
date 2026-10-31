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

mod conservation;
mod probe_draw_execution;
mod probe_parallel;
mod report;

use cosmograph::probe::{
    BodyFounding, Crowd, HarmFounding, PredatorFounding, ProbeFounding, ProbeWorld,
    Variant,
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
    audit_matter: bool,
) -> Result<Arm, String> {
    let start = Instant::now();
    let inspect = cosmograph::draw(dynamics, "probe-inspect", &[]);
    let finish = |micros,
                  members: &[(&cosmograph::schema::Entity, u64)],
                  work: &cosmograph::simulation::Work,
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
            matter: None,
            readings: values,
        }
    };
    if index < 2 {
        let (run, matter) = if audit_matter {
            let (run, matter) = conservation::run(world, dynamics)?;
            (run, Some(matter))
        } else {
            (run_exact(world, dynamics, true)?, None)
        };
        let micros = start.elapsed().as_micros() as u64;
        let members = readings::exact_members(&run);
        let s = run.sim.state();
        let inspected = readings::inspect(&members, inspect);
        let values = readings::evaluate(
            derived, world, &members, &s.sites, s.tick, inspected, &run.meals,
        )?;
        let stored = s.population.groups.len() - s.sites.len();
        let mut out = finish(micros, &members, &run.work, stored, 0, values);
        out.matter = matter;
        Ok(out)
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
    workers: usize,
    output: Option<String>,
    trace: Option<String>,
    trace_entries: Option<String>,
    project: Option<u64>,
    members: Option<[u64; 2]>,
    density: bool,
    crowds: bool,
    water: bool,
    predators: bool,
    approximate: bool,
    bodies: bool,
    harm: bool,
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
        workers: 1,
        output: None,
        trace: None,
        trace_entries: None,
        project: None,
        members: None,
        density: false,
        crowds: false,
        water: false,
        predators: false,
        approximate: false,
        bodies: false,
        harm: false,
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
            "--workers" => o.workers = probe_parallel::requested(number()?)?,
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
            "--harm" => {
                o.bodies = true;
                o.harm = true;
            },
            "--output" => o.output = Some(args.next().ok_or("--output needs a path")?),
            // Draw 0's exact run, `<tick> <witness>` per tick (ruling 642).
            "--trace" => o.trace = Some(args.next().ok_or("--trace needs a path")?),
            "--trace-entries" => {
                o.trace_entries = Some(args.next().ok_or("--trace-entries needs a path")?)
            },
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    if (o.trace.is_some() || o.trace_entries.is_some()) && o.crowds {
        return Err("--trace follows the exact run, which --crowds leaves out".into());
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
    bodies.harm = o.harm.then(HarmFounding::default);
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
    let execution = probe_parallel::Execution::new(o.workers, o.draws);
    eprintln!(
        "Probe receipt: master seed {}, {} draws, arms {arms:?}, {} workers (requested {})",
        o.master, o.draws, execution.workers, execution.requested_workers
    );
    let settings = Settings {
        alpha: 0.05,
        permutations: o.permutations,
        seed: cosmograph::draw(o.master, "probe-check", &[]),
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
    probe_parallel::run(&o, &domain, &arms, |completed| {
        let probe_draw_execution::Completed { draw, infos: these, read_set: set, checks: first } = completed;
        let k = draw.k;
        infos.get_or_insert(these);
        read_set.get_or_insert(set);
        for (&index, a) in arms.iter().zip(&draw.arms) {
            rows[index].push(a.refused.is_none().then(|| a.readings.clone()));
        }
        if let Some(first) = first { checks = first; }
        out.push(draw);
        if (k + 1) % 25 == 0 || k + 1 == o.draws {
            let spent = began.elapsed().as_secs_f64();
            eprintln!(
                "{} draws in {spent:.0} s; {:.2} s per draw",
                k + 1,
                spent / (k + 1) as f64
            );
        }
        Ok(())
    })?;
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
        version: cosmograph::VERSION,
        kind: if o.crowds {
            "cosmograph-probe-crowds"
        } else if o.density {
            "cosmograph-probe-density"
        } else {
            "cosmograph-probe"
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
    let receipt = ExecutedReceipt { receipt, execution };
    let json = serde_json::to_string_pretty(&receipt).map_err(|e| e.to_string())?;
    match o.output {
        Some(path) => std::fs::write(path, json).map_err(|e| e.to_string())?,
        None => println!("{json}"),
    }
    Ok(())
}

/// Scheduling metadata is separate from the unchanged experiment fields.
#[derive(serde::Serialize)]
struct ExecutedReceipt {
    #[serde(flatten)]
    receipt: Receipt,
    execution: probe_parallel::Execution,
}
