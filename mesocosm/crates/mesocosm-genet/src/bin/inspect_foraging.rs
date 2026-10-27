// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Verify a saved run's food steps at their exact native pass-start cuts.
use isocosm::{
    Command, Execution, Session,
    spatial::{Preparation, foraging::FoodDetection},
};
use std::{path::Path, time::Instant};
#[path = "inspect_foraging/cursor.rs"]
mod cursor;
#[path = "inspect_foraging/probe.rs"]
mod probe;

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or(
        "inspect_foraging <world trace> [--from-tick N] [--loads | --census | --probe [--actors=6,19] [--try-walking]]",
    )?;
    let saved = mesocosm_genet::played::read_trace(Path::new(&path))?;
    let started = Instant::now();
    let arguments: Vec<_> = std::env::args().collect();
    let from_tick = if let Some(i) = arguments.iter().position(|a| a == "--from-tick") {
        arguments
            .get(i + 1)
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or("--from-tick wants a native tick")?
    } else {
        0
    };
    let session = Session::load(saved.clone(), Execution::Individuals)?;
    if std::env::args().any(|arg| arg == "--probe") {
        return probe::run(&session);
    }
    if std::env::args().any(|arg| arg == "--census") {
        return probe::census(&session);
    }
    let mut witnesses = Vec::new();
    let mut refused = 0;
    let mut cursor = cursor::Cursor::new(&session);
    for (entry_index, entry) in session.entries.iter().enumerate() {
        let Command::AdvanceBirthSpatial {
            ticks,
            terrain,
            steps,
            arrivals,
            births,
        } = &entry.command
        else {
            continue;
        };
        if steps.iter().all(|answer| answer.request.tick < from_tick) {
            continue;
        }
        let before = cursor.before(entry_index)?;
        for (index, answer) in steps.iter().enumerate() {
            if answer.request.tick < from_tick {
                continue;
            }
            let Some(policy) = before.sim.foraging_policy(&answer.request.profile) else {
                continue;
            };
            if !policy.sight {
                continue;
            }
            if answer.geometry.is_err() {
                refused += 1;
                continue;
            }
            let Preparation::Step(pending) =
                before.prepare_birth_spatial(*ticks, terrain, &steps[..index], arrivals, births)?
            else {
                return Err(format!(
                    "recorded food step {} has no matching native request",
                    answer.request.action
                ));
            };
            if pending.request != answer.request {
                return Err("recorded request differs from its native cut".into());
            }
            let r = &pending.request;
            let target = r.target.ok_or("food step has no target")?;
            let detection = pending
                .sim
                .food_detection(r.actor, target, &r.profile)
                .ok_or_else(|| {
                    format!(
                        "accepted food step {} could not detect its target",
                        r.action
                    )
                })?;
            if !matches!(detection, FoodDetection::Sight(_)) {
                return Err("food step departed from a meal already in contact".into());
            }
            let route = if policy.reachable {
                let found = pending.sim.food_route(r.actor, &r.profile)?;
                let route = found
                    .route
                    .as_ref()
                    .ok_or("chosen meal has no complete physical route")?;
                if route.target != target
                    || route.path.get(..2)
                        != Some(answer.geometry.as_ref().unwrap().path.as_slice())
                {
                    return Err("recorded step differs from the chosen complete route".into());
                }
                probe::verify(&pending.sim, r, route)?;
                if route.strides.get(..1) != answer.geometry.as_ref().unwrap().strides.get(..1) {
                    return Err("recorded gait differs from the chosen complete route".into());
                }
                Some(found)
            } else {
                None
            };
            let mut loads = Vec::new();
            if std::env::args().any(|arg| arg == "--loads") {
                let g = answer.geometry.as_ref().unwrap();
                let terrain = pending.sim.site_geometry(
                    r.site,
                    &g.window,
                    pending.sim.genesis().rules.step_profiles[&r.profile].places,
                )?;
                for (i, pair) in g.path.windows(2).enumerate() {
                    if let Some(stride) = g.strides.get(i).and_then(Option::as_ref) {
                        for sample in 0..=stride.samples() {
                            let load = isocosm::spatial::embodiment::stride_load(
                                &pending.sim,
                                r.actor,
                                &terrain.volume,
                                &pair[0],
                                &pair[1],
                                stride,
                                sample * 1024 / stride.samples(),
                            )?;
                            if !load.supported {
                                return Err(
                                    "accepted step has an unsupported posed mass centre".into()
                                );
                            }
                            loads.push(load);
                        }
                    }
                }
            }
            witnesses.push(
                serde_json::json!({"tick":r.tick,"actor":r.actor,"food":target,
                "process":r.process,"action":r.action,"search":policy.search,"detection":detection,
                "path":answer.geometry.as_ref().unwrap().path,"route":route,"loads":loads}),
            );
        }
    }
    if witnesses.is_empty() {
        return Err("run contains no accepted sight-directed food step".into());
    }
    println!(
        "{}",
        serde_json::json!({"state_hash":format!("{:016x}",session.sim.state_hash()),
        "recorded_hash":format!("{:016x}",saved.state_hash),"tick":saved.tick,
        "accepted_sighted_steps":witnesses.len(),"refused_routes":refused,
        "inspection_us":started.elapsed().as_micros(),"witnesses":witnesses})
    );
    Ok(())
}
