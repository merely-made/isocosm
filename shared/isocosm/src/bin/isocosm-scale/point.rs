// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One drawn world, run tick by tick through the host API and timed.

use super::heap;
use isocosm::{Execution, Founding, Session, schema::Method, simulation::Work};
use serde::Serialize;
use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

#[derive(Serialize)]
pub(super) struct TickRow {
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
pub(super) struct Components {
    clone_micros: u64,
    matter_micros: u64,
    state_hash_micros: u64,
}

#[derive(Serialize)]
pub(super) struct Point {
    pub(super) run: &'static str,
    pub(super) family: &'static str,
    pub(super) mode: &'static str,
    pub(super) founding: Founding,
    generate_micros: u64,
    pub(super) ticks_run: usize,
    pub(super) stopped: Option<String>,
    pub(super) mean_tick_micros: u64,
    median_tick_micros: u64,
    pub(super) evaluations_per_tick: u64,
    pub(super) heap_peak_bytes: usize,
    heap_live_end_bytes: usize,
    distinct_states: usize,
    distinct_read_states: usize,
    final_hash: String,
    components: Components,
    per_tick: Vec<TickRow>,
}

pub(super) fn family(ecology: bool) -> &'static str {
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

fn row(session: &Session, tick_micros: u64, work: &Work, baseline: usize) -> TickRow {
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
        heap_live_bytes: heap::live().saturating_sub(baseline),
    }
}

fn median(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    values.get(values.len() / 2).copied().unwrap_or(0)
}

/// Runs one drawn world tick by tick through the host API and times each tick.
pub(super) fn point(
    run: &'static str,
    founding: Founding,
    mode: Execution,
    ticks: u64,
    wall_cap: u64,
) -> Result<Point, String> {
    let baseline = heap::reset_peak();
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
    let heap_peak_bytes = heap::peak().saturating_sub(baseline);
    let heap_live_end_bytes = heap::live().saturating_sub(baseline);
    let groups = &session.sim.state().population.groups;
    let distinct_states = groups
        .values()
        .map(|g| &g.entity)
        .collect::<BTreeSet<_>>()
        .len();
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
