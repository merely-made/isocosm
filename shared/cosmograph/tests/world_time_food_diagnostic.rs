// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The aligned food contract's three-month diagnostic, never a persistence
//! qualification. The unchanged-rule executable and receipts are immutable
//! retained evidence. This phase captures and verifies its own new identity.

use cosmograph::{
    Execution, Founding, Session,
    bodied::Bodies,
    ecology::{Rates, Web},
};
use support::{Outcome, Report};

#[path = "world_time_year/admission.rs"]
mod admission;
#[path = "world_time_food_diagnostic/catalog.rs"]
mod catalog;
#[path = "world_time_food_diagnostic/controls.rs"]
mod controls;
#[path = "world_time_food_diagnostic/identity.rs"]
mod identity;
#[path = "world_time_food_diagnostic/ingesta.rs"]
mod ingesta;
#[path = "world_time_food_diagnostic/rates.rs"]
mod rates;
#[path = "world_time_food_diagnostic/receipt.rs"]
mod receipt;
#[path = "world_time_year/support.rs"]
mod support;
#[path = "world_time_year/tissue.rs"]
mod tissue;

const SEED: u64 = 9;
const MONTHS: u64 = 3;
const MONTH_TICKS: u64 = 43_830;
const PHASE: &str = "aligned";

fn founding() -> Founding {
    let grid = cosmograph::map::Grid {
        width: 2,
        height: 2,
        ..cosmograph::map::Grid::drawn(SEED)
    };
    Founding {
        seed: SEED,
        sites: 4,
        population: 240,
        cohort_size: 1,
        lineages: 10,
        ecology: true,
        web: Web::Diets,
        ecology_rates: Rates::WorldTime,
        bodies: Some(Bodies {
            clutch: [1, 1],
            sensing: true,
            grazing: true,
            hazard: [0, 0],
            wound: [0, 0],
            ..Bodies::default()
        }),
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ..Founding::default()
    }
}

fn native_world() -> Session {
    Session::new(founding().generate().unwrap(), Execution::Grouped).unwrap()
}

fn assert_bounded_initial(report: &Report) {
    assert_eq!(report.seed, SEED);
    assert_eq!(report.tick_microseconds, 60_000_000);
    assert_eq!(report.started_tick, 2_629_800);
    assert_eq!(report.month_ticks, MONTH_TICKS);
    assert_eq!(report.execution, Execution::Grouped);
    assert_eq!(report.samples.len(), 1);
    assert_eq!(
        report.samples[0]
            .lineages
            .values()
            .map(|t| t.members)
            .sum::<u64>(),
        240
    );
    assert_eq!(report.samples[0].lineages.len(), 10);
    assert!(
        report.root_store_qualified,
        "actual native founder structural admission: {:?}",
        report.root_store_admission.failures
    );
}

#[test]
#[ignore = "zero-tick reference capture; requires frozen aligned source and executable"]
fn capture_aligned_seed9_initial_reference() {
    let session = native_world();
    let report = Report::new(&session);
    assert_bounded_initial(&report);
    identity::capture_reference(&session, &report);
}

#[test]
#[ignore = "131490 exact native ticks; requires a frozen aligned initial reference"]
fn aligned_seed9_food_contract_for_three_months() {
    let mut session = native_world();
    let mut report = Report::new(&session);
    assert_bounded_initial(&report);
    let identity = identity::Inputs::from_environment(&session, &report);
    let path = identity::absolute_environment_path("FOOD_DIAGNOSTIC_REPORT");
    // This reference belongs to the newly frozen aligned build. It makes no
    // claim of identical food rules or dynamics to the old unchanged phase.
    enable(&mut session);
    receipt::advance(&mut session, &mut report, &identity, &path);
    assert_eq!(report.samples.len(), 4);
    assert_eq!(report.samples[3].elapsed_ticks, MONTHS * MONTH_TICKS);
    assert_eq!(report.outcome, Outcome::Inconclusive);
    assert!(report.first_year.is_none());
}

// The observer adapter lives here so the durable receipt does not depend on its
// internal Rust representation. Serialization preserves the native diagnostic's
// own labels for planned, refused, staged and committed work.
fn observer_config() -> cosmograph::watch::diagnostic::FoodDiagnosticConfig {
    use cosmograph::watch::diagnostic::FoodDiagnosticConfig;
    FoodDiagnosticConfig {
        focus_lineages: [1, 3, 4, 6, 8, 9]
            .into_iter()
            .map(|i| format!("lineage:{i}"))
            .collect(),
        examples_per_kind: 16,
        bucket_ticks: 1440,
    }
}

fn enable(session: &mut Session) {
    session
        .sim
        .begin_food_diagnostic(observer_config())
        .unwrap();
}

fn snapshot(session: &Session) -> serde_json::Value {
    serde_json::to_value(session.sim.food_diagnostic().expect("active diagnostic")).unwrap()
}

fn take(session: &mut Session) -> serde_json::Value {
    serde_json::to_value(
        session
            .sim
            .take_food_diagnostic()
            .expect("active diagnostic"),
    )
    .unwrap()
}
