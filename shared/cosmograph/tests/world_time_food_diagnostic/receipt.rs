// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Atomic receipts for diagnostic observations outside native world authority.

use super::{MONTH_TICKS, MONTHS, Report, Session, identity::Inputs, snapshot, take};
use serde::Serialize;
use serde_json::Value;
use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Serialize)]
pub struct Run {
    pub requested_ticks: u64,
    pub advanced_ticks: u64,
    pub wall_millis: u128,
    pub status: &'static str,
    pub failure: Option<String>,
    pub final_state_hash: Option<u64>,
    pub final_history_digest: Option<String>,
}

#[derive(Serialize)]
pub struct Interval {
    /// An initial state is sampled separately; intervals contain native work in
    /// (from_elapsed_ticks, through_elapsed_ticks], with no synthesized rows.
    pub from_elapsed_ticks: u64,
    pub through_elapsed_ticks: u64,
    pub native_observations: Value,
    pub ending_ingesta: super::ingesta::Stores,
}

#[derive(Serialize)]
struct Diagnostic {
    version: u32,
    phase: &'static str,
    observer_configuration: Value,
    initial_ingesta: super::ingesta::Stores,
    scope: &'static str,
    aggregate_basis: &'static str,
    examples_basis: &'static str,
    intervals: Vec<Interval>,
    active_interval: Interval,
}

#[derive(Serialize)]
struct Receipt<'a> {
    #[serde(flatten)]
    trajectory: &'a Report,
    diagnostic_inputs: &'a Inputs,
    diagnostic: &'a Diagnostic,
    run: &'a Run,
}

fn write(path: &Path, report: &Report, inputs: &Inputs, data: &Diagnostic, run: &Run) {
    let parent = path.parent().expect("receipt parent");
    std::fs::create_dir_all(parent).expect("receipt directory");
    let name = path.file_name().expect("receipt name");
    let pending = parent.join(format!(
        ".{}.pending-{}",
        name.to_string_lossy(),
        std::process::id()
    ));
    let mut file = std::fs::File::create(&pending).expect("pending receipt");
    serde_json::to_writer_pretty(
        &mut file,
        &Receipt {
            trajectory: report,
            diagnostic_inputs: inputs,
            diagnostic: data,
            run,
        },
    )
    .expect("serialize receipt");
    file.write_all(b"\n").expect("receipt newline");
    file.sync_all().expect("durable receipt");
    std::fs::rename(pending, path).expect("publish atomic receipt");
}

fn update(data: &mut Diagnostic, s: &Session, started: u64, from: u64) {
    data.active_interval = Interval {
        from_elapsed_ticks: from,
        through_elapsed_ticks: s.sim.state().tick - started,
        native_observations: snapshot(s),
        ending_ingesta: super::ingesta::read(s),
    };
}

fn unchanged(file: &super::identity::FileIdentity) -> bool {
    let now = super::identity::FileIdentity::read(file.path.clone());
    now.sha256 == file.sha256 && now.bytes == file.bytes
}

/// Every native minute runs. Batches only bound operation budgets and progress
/// publication; the observer never selects a process, a target or an outcome.
pub fn advance(s: &mut Session, report: &mut Report, inputs: &Inputs, path: &Path) {
    let horizon = MONTHS * MONTH_TICKS;
    let started = s.sim.state().tick;
    let end = started.checked_add(horizon).unwrap();
    let timer = Instant::now();
    let mut run = Run {
        requested_ticks: horizon,
        advanced_ticks: 0,
        wall_millis: 0,
        status: "running",
        failure: None,
        final_state_hash: None,
        final_history_digest: None,
    };
    let mut data = Diagnostic {
        version: 2,
        phase: super::PHASE,
        observer_configuration: serde_json::to_value(super::observer_config()).unwrap(),
        initial_ingesta: super::ingesta::read(s),
        scope: "aligned food-rule seed9 native observation; its own frozen initial identity; three months cannot qualify persistence",
        aggregate_basis: "complete enabled native observations, grouped in 1440-tick buckets and drained at exact monthly boundaries; observe native first/last ticks for partial buckets",
        examples_basis: "bounded native examples for six nominal animal lineages; examples are not an exhaustive event trace or random sample",
        intervals: vec![],
        active_interval: Interval {
            from_elapsed_ticks: 0,
            through_elapsed_ticks: 0,
            native_observations: snapshot(s),
            ending_ingesta: super::ingesta::read(s),
        },
    };
    write(path, report, inputs, &data, &run);
    let mut interval_start = 0;
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let mut publication = Instant::now();
            for month in 1..=MONTHS {
                let target = started + month * MONTH_TICKS;
                while s.sim.state().tick < target {
                    let ticks = (target - s.sim.state().tick).min(60);
                    s.advance(ticks)
                        .map_err(|why| format!("native advance {ticks}: {why}"))?;
                    run.advanced_ticks = s.sim.state().tick - started;
                    run.wall_millis = timer.elapsed().as_millis();
                    if publication.elapsed() >= Duration::from_secs(30) {
                        update(&mut data, s, started, interval_start);
                        write(path, report, inputs, &data, &run);
                        eprintln!(
                            "food diagnostic seed9: {}/{} ticks, wall {:?}",
                            run.advanced_ticks,
                            horizon,
                            timer.elapsed()
                        );
                        publication = Instant::now();
                    }
                }
                report.observe(s);
                data.intervals.push(Interval {
                    from_elapsed_ticks: interval_start,
                    through_elapsed_ticks: month * MONTH_TICKS,
                    native_observations: take(s),
                    ending_ingesta: super::ingesta::read(s),
                });
                interval_start = month * MONTH_TICKS;
                update(&mut data, s, started, interval_start);
                write(path, report, inputs, &data, &run);
                eprintln!(
                    "food diagnostic seed9: exact month {month}, native lineages {:?}",
                    report.samples.last().unwrap().lineages
                );
                publication = Instant::now();
            }
            if s.sim.state().tick != end {
                return Err("diagnostic did not reach its exact native horizon".into());
            }
            for file in [
                &inputs.executable,
                &inputs.source_manifest,
                &inputs.source_patch,
                &inputs.reference_receipt,
            ] {
                if !unchanged(file) {
                    return Err(format!(
                        "identity input changed during run: {}",
                        file.path.display()
                    ));
                }
            }
            Ok(())
        }));
    run.advanced_ticks = s.sim.state().tick - started;
    run.wall_millis = timer.elapsed().as_millis();
    run.final_state_hash = Some(s.sim.state_hash());
    run.final_history_digest = Some(cosmograph::digest(&s.entries));
    update(&mut data, s, started, interval_start);
    match result {
        Ok(Ok(())) => {
            run.status = "completed";
            write(path, report, inputs, &data, &run);
        },
        Ok(Err(why)) => {
            run.status = "failed";
            run.failure = Some(why.clone());
            write(path, report, inputs, &data, &run);
            panic!("food diagnostic failed: {why}");
        },
        Err(payload) => {
            run.status = "failed";
            run.failure = Some(
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).into()))
                    .unwrap_or_else(|| "native diagnostic panicked".into()),
            );
            write(path, report, inputs, &data, &run);
            std::panic::resume_unwind(payload);
        },
    }
}
