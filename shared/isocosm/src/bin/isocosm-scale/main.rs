// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Scale receipt for ruling 124: what the core runs on this machine, by size.
//! Clocks and heap counting live here, in the host; the sim reads neither.
//! Every world is a draw: per-point seeds come from one master seed, chosen
//! from the system clock before the run unless `--seed` is given.

mod heap;
mod point;

use isocosm::{Execution, Founding};
use point::{Point, family, point};
use serde::Serialize;
use std::collections::BTreeSet;

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
    heap_control: heap::Control,
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
    let r_squared = if syy == 0.0 {
        1.0
    } else {
        sxy * sxy / (sxx * syy)
    };
    Some(Fit {
        family,
        mode,
        quantity,
        sizes: sizes.into_iter().collect(),
        exponent,
        coefficient,
        r_squared,
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
    let mut extend = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--pilot" => pilot = true,
            // Two larger rungs for every ladder, one draw each, no stop rule.
            "--extend" => extend = true,
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
        } else if extend {
            vec![2048, 4096]
        } else {
            (8..=19).map(|k| 1u64 << k).chain([1_000_000]).collect()
        },
        ladder_sites: if pilot { 8 } else { 256 },
        ladder_lineages: 8,
        cohort_size: 32,
        ticks_per_point: if pilot {
            4
        } else if extend {
            8
        } else {
            16
        },
        seeds_per_point: if pilot || extend { 1 } else { 2 },
        ladder_stops_after_mean_tick_micros: if extend { u64::MAX } else { 2_500_000 },
        point_wall_cap_micros: 600_000_000,
        lineage_sweep: if pilot {
            vec![2, 4]
        } else {
            vec![2, 4, 8, 16, 32]
        },
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
        note: "Ladders double the population at fixed sites, lineages and cohort size; each size runs the same drawn worlds in both modes. A ladder stops after the first size whose mean tick exceeds the stated time. The lineage sweep holds population and sites. The history run is one long ecology draw in the grouped mode hosts use. The largest admitted worlds are generated and costed, not advanced. An extension run adds two larger rungs to every ladder, one draw each, with no stop rule and nothing else.",
    };
    let heap_control = heap::control();
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
    let run = if extend { "extension" } else { "ladder" };
    let (ticks, cap) = (design.ticks_per_point, design.point_wall_cap_micros);
    for ecology in [false, true] {
        for mode in [Execution::Grouped, Execution::Individuals] {
            for &n in &design.ladder_sizes {
                let mut slowest = 0;
                for k in 0..design.seeds_per_point {
                    let seed = isocosm::draw(master, "scale-ladder", &[u64::from(ecology), n, k]);
                    let f = base(
                        seed,
                        n,
                        design.ladder_lineages,
                        ecology,
                        design.ladder_sites,
                    );
                    let p = point(run, f, mode, ticks, cap)?;
                    eprintln!(
                        "{run} {} {} n={n} k={k}: mean tick {} us, {} evaluations/tick, peak heap {} MiB",
                        p.family,
                        p.mode,
                        p.mean_tick_micros,
                        p.evaluations_per_tick,
                        p.heap_peak_bytes >> 20
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
        for &l in design.lineage_sweep.iter().filter(|_| !extend) {
            for k in 0..design.seeds_per_point {
                let domain = [u64::from(ecology), u64::from(l), k];
                let seed = isocosm::draw(master, "scale-lineages", &domain);
                let n = design.lineage_sweep_population;
                let f = base(seed, n, l, ecology, design.ladder_sites);
                let p = point("lineages", f, Execution::Grouped, ticks, cap)?;
                eprintln!(
                    "lineages {} l={l} k={k}: mean tick {} us",
                    p.family, p.mean_tick_micros
                );
                points.push(p);
            }
        }
    }
    if !extend {
        let seed = isocosm::draw(master, "scale-history", &[0]);
        let n = design.history_population;
        let f = base(seed, n, design.ladder_lineages, true, design.ladder_sites);
        let (ticks, cap) = (design.history_ticks, design.history_wall_cap_micros);
        let p = point("history", f, Execution::Grouped, ticks, cap)?;
        eprintln!("history: {} ticks, stopped {:?}", p.ticks_run, p.stopped);
        points.push(p);
    }
    if !pilot && !extend {
        for ecology in [false, true] {
            let f = Founding {
                ecology,
                ..design.largest.clone()
            };
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
                .filter(|p| p.run == run && p.family == family(ecology) && p.mode == mode)
                .collect();
            let sizes: BTreeSet<u64> = ladder.iter().map(|p| p.founding.population).collect();
            let top: BTreeSet<u64> = sizes.iter().rev().take(3).copied().collect();
            let quantities = [
                ("mean_tick_seconds", true),
                ("mean_tick_seconds_top3", false),
            ];
            for (quantity, all) in quantities {
                let samples: Vec<(u64, f64)> = ladder
                    .iter()
                    .filter(|p| all || top.contains(&p.founding.population))
                    .map(|p| (p.founding.population, p.mean_tick_micros as f64 / 1e6))
                    .collect();
                fits.extend(power_fit(
                    family(ecology),
                    mode,
                    quantity,
                    &samples,
                    &targets,
                ));
            }
            let size = |p: &&Point| p.founding.population;
            let heap: Vec<(u64, f64)> = ladder
                .iter()
                .map(|p| (size(p), p.heap_peak_bytes as f64))
                .collect();
            fits.extend(power_fit(
                family(ecology),
                mode,
                "heap_peak_bytes",
                &heap,
                &targets,
            ));
            let evals: Vec<(u64, f64)> = ladder
                .iter()
                .map(|p| (size(p), p.evaluations_per_tick as f64))
                .collect();
            fits.extend(power_fit(
                family(ecology),
                mode,
                "evaluations_per_tick",
                &evals,
                &targets,
            ));
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
