// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world-time ecology's trajectory (1077, 1116, 1145): 24 exact world
//! months over native bodies, retaining the original first-year bar. This
//! instrument admits the actual rooted founder structure separately from the
//! trajectory; behavioral root regressions belong to the frozen build evidence.
//! The full run is ignored because the minute clock remains expensive.
//! `WORLD_TIME_DAYS` selects a diagnostic span; a short span is inconclusive.
//! `WORLD_TIME_REPORT` may name an absolute JSON receipt path.

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    ecology::{Rates, Web},
};
use support::{Outcome, Report};

#[path = "world_time_year/admission.rs"]
mod admission;
#[path = "world_time_year/controls.rs"]
mod controls;
#[path = "world_time_year/receipt.rs"]
mod receipt;
#[path = "world_time_year/support.rs"]
mod support;
#[path = "world_time_year/tissue.rs"]
mod tissue;

fn env(key: &str, default: u64) -> u64 {
    match std::env::var(key) {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| panic!("{key} must be an unsigned integer")),
        Err(std::env::VarError::NotPresent) => default,
        Err(why) => panic!("cannot read {key}: {why}"),
    }
}

#[test]
#[ignore = "two native years at the minute; requires admitted protected-root founders"]
fn a_bodied_world_time_web_holds_two_years() {
    let seed = env("WORLD_TIME_SEED", 9);
    let lineages = u32::try_from(env("WORLD_TIME_LINEAGES", 10)).unwrap();
    let grid = cosmograph::map::Grid {
        width: 2,
        height: 2,
        ..cosmograph::map::Grid::drawn(seed)
    };
    let genesis = Founding {
        seed,
        sites: 4,
        population: env("WORLD_TIME_POP", 240),
        cohort_size: 1,
        lineages,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::WorldTime,
        experimental_food_contract: true,
        bodies: Some(Bodies {
            clutch: [1, 1],
            sensing: true,
            grazing: true,
            hazard: [0, 0],
            wound: [0, 0],
            ..Bodies::default()
        }),
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ..Default::default()
    }
    .generate()
    .unwrap();
    // `WORLD_TIME_EXECUTION=individuals` runs the same founding member by
    // member, for checking an execution change in both modes.
    let mode = match std::env::var("WORLD_TIME_EXECUTION").as_deref() {
        Ok("individuals") => Execution::Individuals,
        Ok("grouped") | Err(std::env::VarError::NotPresent) => Execution::Grouped,
        other => panic!("WORLD_TIME_EXECUTION must be grouped or individuals: {other:?}"),
    };
    let mut s = Session::new(genesis, mode).unwrap();
    let mut report = Report::new(&s);
    let months = env("WORLD_TIME_MONTHS", 24);
    let days = env("WORLD_TIME_DAYS", 0);
    let horizon = if days > 0 {
        cosmograph::rules::minutes_ticks(
            days.checked_mul(1440).expect("diagnostic span overflow"),
            s.sim.genesis().rules.tick_microseconds(),
        )
    } else {
        report
            .month_ticks
            .checked_mul(months)
            .expect("monthly horizon overflow")
    };
    assert!(horizon > 0, "supply a positive world-time horizon");
    let path = std::env::var_os("WORLD_TIME_REPORT").map(std::path::PathBuf::from);
    if !report.root_store_qualified {
        receipt::admission_failure(&s, &report, horizon, path.as_deref());
        panic!(
            "native rooted founder admission failed: {:?}",
            report.root_store_admission.failures
        );
    }
    receipt::advance(&mut s, &mut report, horizon, path.as_deref());
    // Summed spans (`COSMOGRAPH_PROFILE_ACTIVE=sum`): label, ms, spans, count.
    for (label, [nanos, spans, count]) in cosmograph::profile_totals() {
        eprintln!("[cosmograph-sum] {:.1} ms {spans} {count} {label}", nanos as f64 / 1e6);
    }
    eprintln!(
        "seed {seed}, trajectory {:?}, first year {:?}, year-two extinctions {:?}; root-store qualified {}",
        report.outcome,
        report.first_year,
        report.second_year_extinctions,
        report.root_store_qualified
    );
    if horizon < 24 * report.month_ticks {
        assert_eq!(
            report.outcome,
            Outcome::Inconclusive,
            "a short diagnostic cannot establish two-year persistence"
        );
    } else {
        assert_eq!(
            report.outcome,
            Outcome::Passed,
            "the first-year bar or second-year persistence failed; inspect the monthly receipt"
        );
    }
}

/// A laid world reads scatter strengths from terrain without needing bodies.
#[test]
fn laid_terrain_gives_scatter_strengths() {
    for seed in [5u64, 9, 11] {
        let grid = cosmograph::map::Grid {
            width: 2 + seed as u32 % 2,
            height: 2,
            ..cosmograph::map::Grid::drawn(seed)
        };
        let founding = Founding {
            seed,
            sites: grid.width * grid.height,
            population: 240,
            cohort_size: 1,
            lineages: 10,
            ecology: true,
            web: Web::Diets,
            ecology_rates: Rates::WorldTime,
            experimental_food_contract: true,
            map: Some(cosmograph::map::Layout::Grid(grid)),
            ..Default::default()
        };
        let g = founding.generate().unwrap();
        let read =
            |key: &str| -> Vec<i64> { g.sites.values().map(|s| s.conditions[key]).collect() };
        let (air, water) = (read("world:air"), read("world:water"));
        assert!(
            air.iter().chain(&water).all(|v| (0..=100).contains(v)),
            "{air:?} {water:?}"
        );
        assert!(
            water.contains(&100) && air.iter().any(|&a| a > 0),
            "seed {seed}: {air:?} {water:?}"
        );
    }
}
