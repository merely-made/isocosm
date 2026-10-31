// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A measurement, not a gate (ruling 888's tuning fork for the light credit):
//! what the producers of the seed-9 A1 world draw from `sim:energy` per site
//! per tick. Run with `--ignored --nocapture`; `LIGHT_DRAW_TICKS` sets the span.

use cosmograph::Execution;
use mesocosm_runtime::{Founded, Runtime, runtime::ecology::Setup};

const CREDIT: u64 = 1_000_000;
const CAP: u64 = 2_000_000;

#[test]
#[ignore = "a measurement; seconds per tick"]
fn what_producers_draw_from_the_sun() {
    let ticks: u64 = std::env::var("LIGHT_DRAW_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(60);
    let mut founded = Founded::generated(9, 256).unwrap();
    let mut founding = founded.genesis.founding.clone().unwrap();
    founding.ecology_rates = cosmograph::ecology::Rates::A1;
    founded.genesis = founding.generate().unwrap();
    let setup = Setup::generated(founded, Execution::Grouped).unwrap();
    let mut runtime = setup.offer(30).unwrap();
    let level = |runtime: &Runtime| -> Vec<u64> {
        runtime.sim().state().sites.values()
            .map(|site| site.accounts.get("sim:energy").copied().unwrap_or(0)).collect()
    };
    let mut drawn: Vec<Vec<u64>> = Vec::new();
    for _ in 0..ticks {
        let before = level(&runtime);
        if runtime.step(1) == 0 { break; }
        let after = level(&runtime);
        drawn.push(before.iter().zip(&after).map(|(b, a)| {
            (b + if *b < CAP { CREDIT } else { 0 }).saturating_sub(*a)
        }).collect());
    }
    let sites = drawn.first().map_or(0, Vec::len);
    let per_site: Vec<u64> = (0..sites).map(|s| drawn.iter().map(|t| t[s]).sum()).collect();
    let peak = drawn.iter().flatten().copied().max().unwrap_or(0);
    eprintln!("[light-draw] ticks={} sites={sites} per_site_total={per_site:?} peak_site_tick={peak}",
        drawn.len());
    eprintln!("[light-draw] levels_now={:?}", level(&runtime));
}
