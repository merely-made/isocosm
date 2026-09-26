// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Re-runs the points of earlier receipts: the same drawn worlds, modes and
//! tick counts, so a change to the core is timed against its predecessor on
//! identical work. Every deterministic field must come out as it was.

use super::point::{Point, point};
use isocosm::{Execution, Founding};
use serde::Serialize;
use serde_json::Value;

/// Long enough for any point an earlier receipt ran to finish.
const WALL_CAP_MICROS: u64 = 1_200_000_000;

#[derive(Serialize)]
pub(super) struct Comparison {
    source: String,
    run: String,
    mode: String,
    population: u64,
    lineages: u32,
    cohort_size: u64,
    seed: u64,
    ticks: u64,
    before_mean_tick_micros: u64,
    after_mean_tick_micros: u64,
    speedup: f64,
    evaluations_per_tick: u64,
    /// Final state hash, per-tick work, stored groups, events, notes and
    /// arrivals, and distinct states, all as before.
    identical: bool,
    differing: Vec<String>,
}

#[derive(Serialize)]
pub(super) struct Remeasure {
    version: u32,
    kind: &'static str,
    sources: Vec<(String, u64)>,
    family: Option<String>,
    note: &'static str,
    comparisons: Vec<Comparison>,
    points: Vec<Point>,
}

/// A run's name as a static tag, for the points it produces.
fn tag(run: &str) -> &'static str {
    ["ladder", "lineages", "history", "extension", "living"]
        .into_iter()
        .find(|t| *t == run)
        .unwrap_or("remeasure")
}

/// The fields that must not change, with timings and heap left out.
fn deterministic(point: &Value) -> Value {
    let mut p = point.clone();
    let fields = ["final_hash", "ticks_run", "evaluations_per_tick"];
    let mut kept: serde_json::Map<String, Value> = fields
        .iter()
        .chain(&["distinct_states", "distinct_read_states"])
        .map(|k| (k.to_string(), p[*k].take()))
        .collect();
    let rows = p["per_tick"].as_array_mut().map(std::mem::take);
    let rows: Vec<Value> = rows
        .unwrap_or_default()
        .into_iter()
        .map(|mut row| {
            if let Some(r) = row.as_object_mut() {
                r.remove("micros");
                r.remove("heap_live_bytes");
            }
            row
        })
        .collect();
    kept.insert("per_tick".into(), Value::Array(rows));
    Value::Object(kept)
}

pub(super) fn run(paths: &[String], family: Option<String>) -> Result<Remeasure, String> {
    let mut sources = Vec::new();
    let mut comparisons = Vec::new();
    let mut points = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let receipt: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        sources.push((path.clone(), receipt["master_seed"].as_u64().unwrap_or(0)));
        for old in receipt["points"]
            .as_array()
            .ok_or("a receipt without points")?
        {
            let ticks = old["ticks_run"].as_u64().unwrap_or(0);
            let kind = old["family"].as_str().unwrap_or_default();
            if ticks == 0 || family.as_deref().is_some_and(|f| f != kind) {
                continue;
            }
            let founding: Founding =
                serde_json::from_value(old["founding"].clone()).map_err(|e| e.to_string())?;
            let mode = match old["mode"].as_str() {
                Some("individuals") => Execution::Individuals,
                _ => Execution::Grouped,
            };
            let run = old["run"].as_str().unwrap_or_default();
            let new = point(tag(run), founding.clone(), mode, ticks, WALL_CAP_MICROS)?;
            let after = serde_json::to_value(&new).map_err(|e| e.to_string())?;
            let (before, after) = (deterministic(old), deterministic(&after));
            let differing: Vec<String> = before
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(k, v)| after.get(k.as_str()) != Some(v))
                .map(|(k, _)| k.clone())
                .collect();
            let (old_mean, new_mean) = (
                old["mean_tick_micros"].as_u64().unwrap_or(0),
                new.mean_tick_micros,
            );
            eprintln!(
                "{run} {} n={} l={}: {old_mean} -> {new_mean} us per tick, identical {}",
                old["mode"],
                founding.population,
                founding.lineages,
                differing.is_empty()
            );
            comparisons.push(Comparison {
                source: path.clone(),
                run: run.into(),
                mode: old["mode"].as_str().unwrap_or_default().into(),
                population: founding.population,
                lineages: founding.lineages,
                cohort_size: founding.cohort_size,
                seed: founding.seed,
                ticks,
                before_mean_tick_micros: old_mean,
                after_mean_tick_micros: new_mean,
                speedup: old_mean as f64 / new_mean.max(1) as f64,
                evaluations_per_tick: old["evaluations_per_tick"].as_u64().unwrap_or(0),
                identical: differing.is_empty(),
                differing,
            });
            points.push(new);
        }
    }
    Ok(Remeasure {
        version: isocosm::VERSION,
        kind: "isocosm-scale-remeasure",
        sources,
        family,
        note: "Each point of the named receipts is run again: the same founding, seed, mode and tick count, so the worlds are the same draws. The earlier mean tick is the receipt's; the later is this run's. Identical means the final state hash, every per-tick count and the distinct states match the earlier point.",
        comparisons,
        points,
    })
}
