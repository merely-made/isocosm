// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Durable progress for the native experiment, outside saved world authority.

use super::{Report, Session};
use serde::Serialize;
use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Serialize)]
pub struct Run {
    /// The source identity the run was built from (`WORLD_TIME_SOURCE`).
    pub source: Option<String>,
    pub requested_ticks: u64,
    pub advanced_ticks: u64,
    pub wall_millis: u128,
    pub status: &'static str,
    pub failure: Option<String>,
    pub final_state_hash: Option<u64>,
    pub final_history_digest: Option<String>,
}

#[derive(Serialize)]
struct Receipt<'a> {
    #[serde(flatten)]
    report: &'a Report,
    run: &'a Run,
}

fn source() -> Option<String> {
    std::env::var("WORLD_TIME_SOURCE").ok()
}

fn write(path: Option<&Path>, report: &Report, run: &Run) -> std::io::Result<()> {
    let Some(path) = path else { return Ok(()) };
    assert!(
        path.is_absolute(),
        "WORLD_TIME_REPORT must name an absolute receipt path"
    );
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("receipt has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("receipt has no file name"))?;
    let pending = parent.join(format!(
        ".{}.pending-{}",
        name.to_string_lossy(),
        std::process::id()
    ));
    let mut file = std::fs::File::create(&pending)?;
    file.write_all(&serde_json::to_vec_pretty(&Receipt { report, run })?)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    std::fs::rename(pending, path)
}

pub fn admission_failure(s: &Session, report: &Report, horizon: u64, path: Option<&Path>) {
    let run = Run {
        source: source(),
        requested_ticks: horizon,
        advanced_ticks: 0,
        wall_millis: 0,
        status: "qualification-failed",
        failure: Some(report.root_store_admission.failures.join("; ")),
        final_state_hash: Some(s.sim.state_hash()),
        final_history_digest: Some(cosmograph::digest(&s.entries)),
    };
    write(path, report, &run).expect("rooted founder qualification failure receipt");
}

/// Every native tick remains in the simulation; only observer publication is paced.
pub fn advance(s: &mut Session, report: &mut Report, horizon: u64, path: Option<&Path>) {
    let started_tick = s.sim.state().tick;
    let end = started_tick
        .checked_add(horizon)
        .expect("world-time horizon overflow");
    let timer = Instant::now();
    let mut run = Run {
        source: source(),
        requested_ticks: horizon,
        advanced_ticks: 0,
        wall_millis: 0,
        status: "running",
        failure: None,
        final_state_hash: None,
        final_history_digest: None,
    };
    write(path, report, &run).expect("initial world-time receipt");
    let result =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let mut next_month = report.month_ticks;
            let mut last_publication = Instant::now();
            while s.sim.state().tick < end {
                let target = started_tick
                    .checked_add(next_month)
                    .ok_or("monthly sampling overflow")?
                    .min(end);
                while s.sim.state().tick < target {
                    // The batch only bounds an advance's operation budget, never its tick semantics.
                    let ticks = (target - s.sim.state().tick).min(60);
                    s.advance(ticks)
                        .map_err(|why| format!("native advance of {ticks} ticks: {why}"))?;
                    run.advanced_ticks = s.sim.state().tick - started_tick;
                    run.wall_millis = timer.elapsed().as_millis();
                    if last_publication.elapsed() >= Duration::from_secs(30) {
                        write(path, report, &run)
                            .map_err(|why| format!("progress receipt: {why}"))?;
                        eprintln!(
                            "seed {}, progress {}/{} minute-clock ticks, wall {:?}",
                            report.seed,
                            run.advanced_ticks,
                            horizon,
                            timer.elapsed()
                        );
                        last_publication = Instant::now();
                    }
                }
                report.observe(s);
                let sample = report.samples.last().ok_or("monthly observation absent")?;
                eprintln!(
                    "seed {}, elapsed ticks {}, wall {:?}, lineages {:?}",
                    report.seed,
                    sample.elapsed_ticks,
                    timer.elapsed(),
                    sample.lineages
                );
                write(path, report, &run).map_err(|why| format!("monthly receipt: {why}"))?;
                last_publication = Instant::now();
                next_month = next_month
                    .checked_add(report.month_ticks)
                    .ok_or("monthly sampling overflow")?;
            }
            Ok(())
        }));
    run.advanced_ticks = s.sim.state().tick - started_tick;
    run.wall_millis = timer.elapsed().as_millis();
    match result {
        Ok(Ok(())) => {
            run.status = "completed";
            run.final_state_hash = Some(s.sim.state_hash());
            run.final_history_digest = Some(cosmograph::digest(&s.entries));
            write(path, report, &run).expect("final world-time receipt");
        },
        Ok(Err(why)) => {
            run.status = "failed";
            run.failure = Some(why.clone());
            run.final_state_hash = Some(s.sim.state_hash());
            run.final_history_digest = Some(cosmograph::digest(&s.entries));
            report.observe(s);
            write(path, report, &run).expect("failed world-time receipt");
            panic!("world-time run failed: {why}");
        },
        Err(payload) => {
            run.status = "failed";
            run.failure = Some(
                payload
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).into()))
                    .unwrap_or_else(|| "native experiment panicked".into()),
            );
            write(path, report, &run).expect("panicked world-time receipt");
            std::panic::resume_unwind(payload);
        },
    }
}
