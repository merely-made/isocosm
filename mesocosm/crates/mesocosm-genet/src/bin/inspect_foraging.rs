// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Verify a saved run's food steps at their exact native pass-start cuts.
use isocosm::{
    Command, Execution, Session,
    spatial::{Preparation, foraging::FoodDetection},
};
use std::{path::Path, time::Instant};

fn main() -> Result<(), String> {
    let path = std::env::args()
        .nth(1)
        .ok_or("inspect_foraging <world trace>")?;
    let saved = mesocosm_genet::played::read_trace(Path::new(&path))?;
    let started = Instant::now();
    let session = Session::load(saved.clone(), Execution::Individuals)?;
    let mut witnesses = Vec::new();
    let mut refused = 0;
    for entry in &session.entries {
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
        if steps.is_empty() {
            continue;
        }
        let before = session.before_entry(&entry.id)?;
        for (index, answer) in steps.iter().enumerate() {
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
            witnesses.push(
                serde_json::json!({"tick":r.tick,"actor":r.actor,"food":target,
                "process":r.process,"action":r.action,"search":policy.search,"detection":detection,
                "path":answer.geometry.as_ref().unwrap().path}),
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
