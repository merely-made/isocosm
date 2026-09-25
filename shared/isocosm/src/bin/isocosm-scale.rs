// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Scale receipt for ruling 124: what the core runs on this machine, by size.
//! Clocks and heap counting live here, in the host; the sim reads neither.
//! Every world is a draw: per-point seeds come from one master seed, chosen
//! from the system clock before the run unless `--seed` is given.

use isocosm::{Execution, Founding, Session, schema::Method};
use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeSet;
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::time::Instant;

/// Heap accounting for this measurement binary only. `GlobalAlloc` is an
/// unsafe trait, so implementing it needs `unsafe`; the wrapper forwards every
/// call unchanged to `System` and only counts bytes. It is the one portable way
/// to read live and peak heap without a new dependency, and the core library
/// itself stays free of `unsafe`.
struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn grew(bytes: usize) {
    let now = LIVE.fetch_add(bytes, Relaxed) + bytes;
    PEAK.fetch_max(now, Relaxed);
}

// SAFETY: each method forwards its arguments unchanged to `System`, which
// upholds the `GlobalAlloc` contract; the counters never touch the memory.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            grew(layout.size());
        }
        p
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            grew(layout.size());
        }
        p
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        LIVE.fetch_sub(layout.size(), Relaxed);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let p = unsafe { System.realloc(ptr, layout, size) };
        if !p.is_null() {
            LIVE.fetch_sub(layout.size(), Relaxed);
            grew(size);
        }
        p
    }
}

#[global_allocator]
static HEAP: Counting = Counting;

fn live() -> usize {
    LIVE.load(Relaxed)
}
fn reset_peak() -> usize {
    let now = live();
    PEAK.store(now, Relaxed);
    now
}

#[derive(Serialize)]
struct TickRow {
    tick: u64,
    micros: u64,
    evaluations: u64,
    represented: u64,
    accepted: u64,
    blocked: u64,
    alive: u64,
    stored_members: u64,
    groups: usize,
    events: usize,
    notes: usize,
    arrivals: usize,
    heap_live_bytes: usize,
}

#[derive(Serialize)]
struct Components {
    clone_micros: u64,
    matter_micros: u64,
    state_hash_micros: u64,
}

#[derive(Serialize)]
struct Point {
    run: &'static str,
    family: &'static str,
    mode: &'static str,
    founding: Founding,
    generate_micros: u64,
    ticks_run: usize,
    stopped: Option<String>,
    mean_tick_micros: u64,
    median_tick_micros: u64,
    evaluations_per_tick: u64,
    heap_peak_bytes: usize,
    heap_live_end_bytes: usize,
    distinct_states: usize,
    distinct_read_states: usize,
    final_hash: String,
    components: Components,
    per_tick: Vec<TickRow>,
}

#[derive(Serialize)]
struct Fit {
    family: &'static str,
    mode: &'static str,
    quantity: &'static str,
    sizes: Vec<u64>,
    exponent: f64,
    coefficient: f64,
    r_squared: f64,
    predicted: Vec<(u64, f64)>,
}

#[derive(Serialize)]
struct Receipt {
    version: u32,
    kind: &'static str,
    master_seed: u64,
    pilot: bool,
    design: Design,
    heap_control: Control,
    points: Vec<Point>,
    fits: Vec<Fit>,
}

#[derive(Serialize)]
struct Design {
    ladder_sizes: Vec<u64>,
    ladder_sites: u32,
    ladder_lineages: u32,
    cohort_size: u64,
    ticks_per_point: u64,
    seeds_per_point: u64,
    ladder_stops_after_mean_tick_micros: u64,
    point_wall_cap_micros: u64,
    lineage_sweep: Vec<u32>,
    lineage_sweep_population: u64,
    history_population: u64,
    history_ticks: u64,
    history_wall_cap_micros: u64,
    largest: Founding,
    note: &'static str,
}

#[derive(Serialize)]
struct Control {
    allocated_bytes: usize,
    live_rise_bytes: usize,
    live_fall_bytes: usize,
    passed: bool,
}

/// Positive control: the instrument must see a known allocation come and go.
fn heap_control() -> Control {
    let size = 64 << 20;
    let before = live();
    let block = black_box(vec![1u8; size]);
    let during = live();
    drop(block);
    let after = live();
    let rise = during.saturating_sub(before);
    let fall = during.saturating_sub(after);
    Control {
        allocated_bytes: size,
        live_rise_bytes: rise,
        live_fall_bytes: fall,
        passed: rise >= size && fall >= size,
    }
}

fn family(ecology: bool) -> &'static str {
    if ecology { "ecology" } else { "reservoir" }
}
fn mode_name(mode: Execution) -> &'static str {
    match mode {
        Execution::Individuals => "individuals",
        Execution::Grouped => "grouped",
    }
}
fn micros(start: Instant) -> u64 {
    start.elapsed().as_micros() as u64
}

fn row(session: &Session, tick_micros: u64, work: &isocosm::simulation::Work, baseline: usize) -> TickRow {
    let s = session.sim.state();
    TickRow {
        tick: s.tick,
        micros: tick_micros,
        evaluations: work.evaluations,
        represented: work.represented,
        accepted: work.accepted,
        blocked: work.blocked,
        alive: s
            .population
            .groups
            .values()
            .filter(|g| g.entity.alive && g.entity.method != Method::Inert)
            .map(|g| g.count)
            .sum(),
        stored_members: s.population.count(),
        groups: s.population.groups.len(),
        events: s.events.len(),
        notes: s.notes.len(),
        arrivals: s.reach.arrivals.values().map(|m| m.len()).sum(),
        heap_live_bytes: live().saturating_sub(baseline),
    }
}

fn median(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    values.get(values.len() / 2).copied().unwrap_or(0)
}

/// Runs one drawn world tick by tick through the host API and times each tick.
fn point(
    run: &'static str,
    founding: Founding,
    mode: Execution,
    ticks: u64,
    wall_cap: u64,
) -> Result<Point, String> {
    let baseline = reset_peak();
    let start = Instant::now();
    let genesis = founding.generate()?;
    let mut session = Session::new(genesis, mode)?;
    let generate_micros = micros(start);
    let mut rows = Vec::new();
    let mut stopped = None;
    let begun = Instant::now();
    for _ in 0..ticks {
        let t = Instant::now();
        match session.advance(1) {
            Ok(work) => rows.push(row(&session, micros(t), &work, baseline)),
            Err(why) => {
                stopped = Some(why);
                break;
            },
        }
        if micros(begun) > wall_cap {
            stopped = Some(format!("wall cap of {wall_cap} us reached"));
            break;
        }
    }
    let heap_peak_bytes = PEAK.load(Relaxed).saturating_sub(baseline);
    let heap_live_end_bytes = live().saturating_sub(baseline);
    let groups = &session.sim.state().population.groups;
    let distinct_states = groups.values().map(|g| &g.entity).collect::<BTreeSet<_>>().len();
    // The fields the generated families' processes read: no visits or arrival.
    let distinct_read_states = groups
        .values()
        .map(|g| {
            let e = &g.entity;
            (&e.lineage, &e.traits, e.place, e.alive, e.born, &e.accounts)
        })
        .collect::<BTreeSet<_>>()
        .len();
    let t = Instant::now();
    black_box(session.sim.clone());
    let clone_micros = micros(t);
    let t = Instant::now();
    black_box(session.sim.matter());
    let matter_micros = micros(t);
    let t = Instant::now();
    let final_hash = session.sim.state_hash();
    let state_hash_micros = micros(t);
    let mut times: Vec<u64> = rows.iter().map(|r| r.micros).collect();
    let total: u64 = times.iter().sum();
    let evaluations: u64 = rows.iter().map(|r| r.evaluations).sum();
    let n = rows.len().max(1) as u64;
    Ok(Point {
        run,
        family: family(founding.ecology),
        mode: mode_name(mode),
        founding,
        generate_micros,
        ticks_run: rows.len(),
        stopped,
        mean_tick_micros: total / n,
        median_tick_micros: median(&mut times),
        evaluations_per_tick: evaluations / n,
        heap_peak_bytes,
        heap_live_end_bytes,
        distinct_states,
        distinct_read_states,
        final_hash,
        components: Components {
            clone_micros,
            matter_micros,
            state_hash_micros,
        },
        per_tick: rows,
    })
}

/// Least squares on logarithms: quantity = coefficient * size ^ exponent.
fn power_fit(
    family: &'static str,
    mode: &'static str,
    quantity: &'static str,
    samples: &[(u64, f64)],
    targets: &[u64],
) -> Option<Fit> {
    let sizes: BTreeSet<u64> = samples.iter().map(|s| s.0).collect();
    if sizes.len() < 2 || samples.iter().any(|s| s.1 <= 0.0) {
        return None;
    }
    let n = samples.len() as f64;
    let xs: Vec<f64> = samples.iter().map(|s| (s.0 as f64).ln()).collect();
    let ys: Vec<f64> = samples.iter().map(|s| s.1.ln()).collect();
    let (mx, my) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let sxx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    let syy: f64 = ys.iter().map(|y| (y - my).powi(2)).sum();
    let sxy: f64 = xs.iter().zip(&ys).map(|(x, y)| (x - mx) * (y - my)).sum();
    let exponent = sxy / sxx;
    let coefficient = (my - exponent * mx).exp();
    Some(Fit {
        family,
        mode,
        quantity,
        sizes: sizes.into_iter().collect(),
        exponent,
        coefficient,
        r_squared: if syy == 0.0 { 1.0 } else { sxy * sxy / (sxx * syy) },
        predicted: targets
            .iter()
            .map(|&t| (t, coefficient * (t as f64).powf(exponent)))
            .collect(),
    })
}

fn main() {
    if let Err(why) = run() {
        eprintln!("{why}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut master = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos() as u64;
    let mut output = None;
    let mut pilot = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pilot" => pilot = true,
            "--seed" => {
                master = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--seed needs an integer")?
            },
            "--output" => output = Some(args.next().ok_or("--output needs a path")?),
            _ => return Err(format!("unknown argument {arg}")),
        }
    }
    eprintln!("Scale receipt: master seed {master}");
    let design = Design {
        // Doubling from 256 inside the generator's domain, which ends at 1,000,000.
        ladder_sizes: if pilot {
            vec![64, 128]
        } else {
            (8..=19).map(|k| 1u64 << k).chain([1_000_000]).collect()
        },
        ladder_sites: if pilot { 8 } else { 256 },
        ladder_lineages: 8,
        cohort_size: 32,
        ticks_per_point: if pilot { 4 } else { 16 },
        seeds_per_point: if pilot { 1 } else { 2 },
        ladder_stops_after_mean_tick_micros: 2_500_000,
        point_wall_cap_micros: 600_000_000,
        lineage_sweep: if pilot { vec![2, 4] } else { vec![2, 4, 8, 16, 32] },
        lineage_sweep_population: if pilot { 64 } else { 1024 },
        history_population: if pilot { 64 } else { 1024 },
        history_ticks: if pilot { 16 } else { 512 },
        history_wall_cap_micros: 1_200_000_000,
        largest: Founding {
            seed: isocosm::draw(master, "scale-largest", &[0]),
            sites: 256,
            population: 1_000_000,
            lineages: 32,
            cohort_size: 32,
            ..Founding::default()
        },
        note: "Ladders double the population at fixed sites, lineages and cohort size; each size runs the same drawn worlds in both modes. A ladder stops after the first size whose mean tick exceeds the stated time. The lineage sweep holds population and sites. The history run is one long ecology draw in the grouped mode hosts use. The largest admitted worlds are generated and costed, not advanced.",
    };
    let heap_control = heap_control();
    eprintln!("heap control passed: {}", heap_control.passed);
    let mut points = Vec::new();
    let base = |seed, population, lineages, ecology, sites| Founding {
        seed,
        sites,
        population,
        lineages,
        cohort_size: design.cohort_size,
        ecology,
        ..Founding::default()
    };
    for ecology in [false, true] {
        for mode in [Execution::Grouped, Execution::Individuals] {
            for &n in &design.ladder_sizes {
                let mut slowest = 0;
                for k in 0..design.seeds_per_point {
                    let seed = isocosm::draw(master, "scale-ladder", &[u64::from(ecology), n, k]);
                    let f = base(seed, n, design.ladder_lineages, ecology, design.ladder_sites);
                    let p = point("ladder", f, mode, design.ticks_per_point, design.point_wall_cap_micros)?;
                    eprintln!(
                        "ladder {} {} n={n} k={k}: mean tick {} us, {} evaluations/tick, peak heap {} MiB",
                        p.family, p.mode, p.mean_tick_micros, p.evaluations_per_tick, p.heap_peak_bytes >> 20
                    );
                    slowest = slowest.max(p.mean_tick_micros);
                    points.push(p);
                }
                if slowest > design.ladder_stops_after_mean_tick_micros {
                    break;
                }
            }
        }
    }
    for ecology in [false, true] {
        for &l in &design.lineage_sweep {
            for k in 0..design.seeds_per_point {
                let seed = isocosm::draw(master, "scale-lineages", &[u64::from(ecology), u64::from(l), k]);
                let f = base(seed, design.lineage_sweep_population, l, ecology, design.ladder_sites);
                let p = point("lineages", f, Execution::Grouped, design.ticks_per_point, design.point_wall_cap_micros)?;
                eprintln!("lineages {} l={l} k={k}: mean tick {} us", p.family, p.mean_tick_micros);
                points.push(p);
            }
        }
    }
    let seed = isocosm::draw(master, "scale-history", &[0]);
    let f = base(seed, design.history_population, design.ladder_lineages, true, design.ladder_sites);
    let p = point("history", f, Execution::Grouped, design.history_ticks, design.history_wall_cap_micros)?;
    eprintln!("history: {} ticks, stopped {:?}", p.ticks_run, p.stopped);
    points.push(p);
    if !pilot {
        for ecology in [false, true] {
            let f = Founding { ecology, ..design.largest.clone() };
            points.push(point("largest", f, Execution::Grouped, 0, 0)?);
            eprintln!("largest {} generated", family(ecology));
        }
    }
    let targets = [10_000, 20_000, 50_000, 100_000, 1_000_000];
    let mut fits = Vec::new();
    for ecology in [false, true] {
        for mode in ["grouped", "individuals"] {
            let ladder: Vec<&Point> = points
                .iter()
                .filter(|p| p.run == "ladder" && p.family == family(ecology) && p.mode == mode)
                .collect();
            let sizes: BTreeSet<u64> = ladder.iter().map(|p| p.founding.population).collect();
            let top: BTreeSet<u64> = sizes.iter().rev().take(3).copied().collect();
            for (quantity, all) in [("mean_tick_seconds", true), ("mean_tick_seconds_top3", false)] {
                let samples: Vec<(u64, f64)> = ladder
                    .iter()
                    .filter(|p| all || top.contains(&p.founding.population))
                    .map(|p| (p.founding.population, p.mean_tick_micros as f64 / 1e6))
                    .collect();
                fits.extend(power_fit(family(ecology), mode, quantity, &samples, &targets));
            }
            let heap: Vec<(u64, f64)> =
                ladder.iter().map(|p| (p.founding.population, p.heap_peak_bytes as f64)).collect();
            fits.extend(power_fit(family(ecology), mode, "heap_peak_bytes", &heap, &targets));
            let evals: Vec<(u64, f64)> =
                ladder.iter().map(|p| (p.founding.population, p.evaluations_per_tick as f64)).collect();
            fits.extend(power_fit(family(ecology), mode, "evaluations_per_tick", &evals, &targets));
        }
    }
    let receipt = Receipt {
        version: isocosm::VERSION,
        kind: "isocosm-scale",
        master_seed: master,
        pilot,
        design,
        heap_control,
        points,
        fits,
    };
    let json = serde_json::to_string_pretty(&receipt).map_err(|e| e.to_string())?;
    match output {
        Some(path) => std::fs::write(path, json).map_err(|e| e.to_string())?,
        None => println!("{json}"),
    }
    Ok(())
}
