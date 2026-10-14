// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One independent draw, with the original arm order and first-draw checks.

use super::{Options, arm, report::*};
use isocosm::probe::{REFUSED, readings, run_exact, run_exact_traced};
use std::collections::BTreeSet;

#[derive(serde::Serialize)]
pub(super) struct Completed {
    pub draw: Draw,
    pub infos: Vec<ReadingInfo>,
    pub read_set: BTreeSet<String>,
    pub checks: Option<Checks>,
}

pub(super) fn signature(infos: &[ReadingInfo]) -> Vec<(String, u32)> {
    infos.iter().map(|r| (r.key.clone(), r.bound_per_mille)).collect()
}

pub(super) fn execute(
    o: &Options, domain: &Domain, arms: &[usize], k: u64,
    expected: Option<&[(String, u32)]>,
) -> Result<Completed, String> {
    let seed = isocosm::draw(o.master, "probe-world", &[k]);
    let world = domain.generate(seed)?;
    let derived = readings::derive(&world);
    let similitude = world.similitude()?;
    let infos: Vec<ReadingInfo> = derived.iter()
        .map(|r| ReadingInfo::new(r, similitude.bound(&r.key))).collect();
    if let Some(first) = expected {
        let these = signature(&infos);
        if first != these.as_slice() {
            let only = |x: &[(String, u32)], y: &[(String, u32)]| {
                x.iter().filter(|r| !y.contains(r)).map(|r| r.0.clone()).collect::<Vec<_>>()
            };
            return Err(format!(
                "draw {k} reads differently from draw 0: only in draw 0 {:?}, only in draw {k} {:?}",
                only(first, &these), only(&these, first)
            ));
        }
    }
    let read_set = readings::read_set(&world);
    let mut draw = Draw::new(k, seed, &world)?;
    for &index in arms {
        let dynamics = isocosm::draw(o.master, "probe-dynamics", &[k, index as u64]);
        let a = match arm(&world, &derived, index, dynamics, o.harm) {
            Ok(a) => a,
            Err(why) if why.contains(REFUSED) => Arm::refused(ARMS[index], dynamics, why),
            Err(why) => return Err(format!("draw {k}, {}: {why}", ARMS[index])),
        };
        draw.arms.push(a);
    }
    let checks = if k == 0 && !o.crowds {
        let dynamics = draw.arms[0].dynamics;
        let mut lines = String::new();
        let mut entries = state_witness::Trace::new();
        let mut line = |sim: &isocosm::Simulation| {
            let tick = sim.state().tick;
            lines += &format!("{tick} {:016x}\n", sim.state_hash());
            if o.trace_entries.is_some() {
                entries.push(tick, sim.entity_witness()).expect("ticks advance");
            }
        };
        let reference = run_exact_traced(&world, dynamics, true, &mut line)?.sim.state_hash();
        if let Some(path) = &o.trace {
            std::fs::write(path, &lines).map_err(|e| e.to_string())?;
        }
        if let Some(path) = &o.trace_entries {
            let framed = entries.to_framed().map_err(|e| format!("{e:?}"))?;
            std::fs::write(path, framed).map_err(|e| e.to_string())?;
        }
        Some(Checks {
            exact_rerun_identical: reference == run_exact(&world, dynamics, true)?.sim.state_hash(),
            collect_changes_no_outcome: reference == run_exact(&world, dynamics, false)?.sim.state_hash(),
        })
    } else { None };
    Ok(Completed { draw, infos, read_set, checks })
}
