// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A real world grant followed by an explicitly separate progression probe.
//! Prints a JSON receipt; it neither saves a world nor casts a spell.

use mesocosm_core::{World, state_hash};
use mesocosm_runtime::{
    Trial,
    glyphs::{AcceptedKind, EventGrant, GlyphRules},
};
use wing_glyphs::divine::{DomainChoice, DomainWindow, WindowLimits};
use wing_glyphs::{CanonSpec, GlyphDefinition, Journey, SCHEMA_VERSION};

fn main() -> Result<(), String> {
    let (seed, world, at) = (0..32)
        .find_map(|seed| {
            let world = World::new(seed, 0);
            let p = world.controlled()?.position;
            let at = [p[0], p[1] - 1, p[2]];
            (at[1] >= 1 && world.ground().solid(at) && world.in_reach(at))
                .then_some((seed, world, at))
        })
        .ok_or("no grounded founder in fixture range")?;
    let baseline = state_hash(&world);
    let actor = world.controlled_id().ok_or("missing founder")?;
    let mut trial = Trial::new(&world)?;
    trial.enable_glyphs(GlyphRules {
        canon: CanonSpec {
            version: SCHEMA_VERSION,
            limits: Default::default(),
            id: "demo:carving-canon".into(),
            revision: 1,
            glyphs: vec![GlyphDefinition {
                id: "demo:earth".into(),
                display: "#".into(),
                effect: "demo:reshape-reference".into(),
            }],
            variants: vec![],
        },
        individual: format!("demo:seed-{seed}-organism-{}", actor.0),
        organism: actor,
        unlock_thresholds: vec![10],
        grants: vec![EventGrant {
            event: AcceptedKind::Carved,
            glyph: "demo:earth".into(),
        }],
    })?;
    let metric = format!("demo:seed-{seed}-actor-{}-removed-voxels-v1", actor.0);
    let mut window = DomainWindow::new(
        DomainChoice {
            metric_id: metric.clone(),
            unit: "removed voxels".into(),
            start_tick: 0,
            duration_ticks: 4,
        },
        WindowLimits {
            min_ticks: 1,
            max_ticks: 128,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let mut observations = Vec::new();
    for step in 0..4 {
        let tick = trial.world().tick;
        let applied = match step {
            0 | 1 => trial.carve(at, 1),
            2 => trial.carve(at, 0),
            _ => trial.step(),
        };
        if !applied {
            return Err("fixture stopped before four ticks".into());
        }
        let removed: u64 = trial
            .carves()
            .iter()
            .filter(|e| e.organism == actor)
            .map(|e| u64::from(e.removed))
            .sum();
        window
            .observe(&metric, tick, removed)
            .map_err(|e| format!("{e:?}"))?;
        observations.push(serde_json::json!({"tick": tick, "removed_voxels": removed}));
    }
    let settlement = window
        .settle(trial.world().tick)
        .map_err(|e| format!("{e:?}"))?;
    let reading = trial.glyphs().ok_or("missing glyph reading")?;
    let mut probe = Journey::from_json(&reading.journey().to_json()?)?;
    let eligibility = probe.eligibility();
    probe.ascend()?;
    probe.reincarnate()?;
    probe.add_experience(9)?;
    let before_threshold = probe.inherent_unlocked().len();
    probe.add_experience(1)?;
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({
        "scope": "real accepted-event acquisition; separate shared progression and period probe",
        "seed": seed, "baseline_hash": format!("{baseline:016x}"),
        "final_world_hash": format!("{:016x}", trial.state_hash()),
        "actor": actor, "carve_target": at, "accepted_grants": reading.records(),
        "qualification": eligibility, "observations": observations, "measurement": settlement,
        "standalone_progression": {"journey": probe.snapshot(), "motif": probe.motif(),
            "ascension_basis": probe.ascension_basis(),
            "inherent_at_9_xp": before_threshold, "inherent_at_10_xp": probe.inherent_unlocked().len()},
        "world_reincarnation_applied": false, "experience_awarded_by_world": false,
        "effect_reference_executed": false, "divine_power_minted": false,
    })).map_err(|e| e.to_string())?);
    Ok(())
}
