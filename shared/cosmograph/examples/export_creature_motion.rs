// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native generated anatomy and an admitted step for the renderer comparison.
//! JSON is the boundary: the product-neutral renderer does not depend on Cosmograph.
use cosmograph::{Execution, Founding, simulation::Simulation, spatial::embodiment as body};
use isometer_core::{
    Yaw,
    legs::{self, UNIT},
};
use serde_json::json;
use std::{fs, time::Instant};

fn main() -> Result<(), String> {
    let path = std::env::args().nth(1).ok_or("output JSON path required")?;
    let seed = 20261103;
    let generated = Founding {
        seed,
        population: 60,
        sites: 6,
        ecology: true,
        map: Some(cosmograph::map::Layout::Grid(cosmograph::map::Grid {
            width: 3,
            height: 2,
            ..cosmograph::map::Grid::drawn(seed)
        })),
        bodies: Some(cosmograph::bodied::Bodies {
            roster: true,
            founding: Some("SpacedRoster".into()),
            proportions: true,
            ..Default::default()
        }),
        ..Default::default()
    }
    .generate()?;
    // A disclosed flat diagnostic pad removes terrain selection from this render probe.
    let terrain = Founding {
        seed: 7,
        sites: 4,
        population: 8,
        map: Some(cosmograph::map::Layout::Grid(cosmograph::map::Grid {
            shape: "shape:plane".into(),
            width: 2,
            height: 2,
            side: 768,
            elevation: [16, 16],
            relief: [0, 0],
            sea_per_mille: 0,
        })),
        ..Default::default()
    }
    .generate()?;
    let sim = Simulation::new(terrain, Execution::Individuals)?;
    let volume = sim.volume(0, [320; 2], [448; 2], 16)?;
    let walking = body::Walking::default();
    let mut attempts = Vec::new();
    for (id, cohort) in &generated.population.groups {
        let Some(doc) = &cohort.entity.body else {
            continue;
        };
        let chains = legs::chains(doc);
        if chains.is_empty() {
            continue;
        }
        let clock = Instant::now();
        let mut grounding = body::Grounding::new(&volume)
            .with_articulation(true)
            .with_walking(Some(walking));
        let a = match grounding.at(doc, 384, 384, Yaw::Zero) {
            Ok(a) => a,
            Err(e) => {
                attempts.push(json!({"id":id,"failure":e}));
                continue;
            },
        };
        let b = match grounding.at(doc, 385, 384, Yaw::Zero) {
            Ok(b) => b,
            Err(e) => {
                attempts.push(json!({"id":id,"failure":e}));
                continue;
            },
        };
        let stride = match body::plan_stride(&volume, doc, &a, &b, walking, &[]) {
            Ok(s) => s,
            Err(e) => {
                attempts.push(json!({"id":id,"failure":e}));
                continue;
            },
        };
        let planning_us = clock.elapsed().as_secs_f64() * 1e6;
        let clock = Instant::now();
        let mut frames = Vec::new();
        for i in 0..24 {
            let sample = body::stride_sample(&a, &b, &stride, i * UNIT / 23)?;
            let feet = body::stride_feet(&sample, a.yaw)?;
            frames.push(json!({"root":sample.root, "feet":feet,"bearing":sample.bearing}));
        }
        let sampling_us = clock.elapsed().as_secs_f64() * 1e6;
        let out = json!({"seed":seed,"id":id,"lineage":cohort.entity.lineage,
            "selection":"SpacedRoster with generated proportions; first leg-bearing cohort admitting +x step; failures retained",
            "prior_configuration":"stock Bodies at this seed had no articulated leg chains; changed generator domain, not anatomy or strength",
            "attempts":attempts,"document":doc,"walking":walking,
            "from":a,"to":b,"stride":stride,"frames":frames,"unit":UNIT,
            "legs":chains.len(),"ground_y":16,"planning_us":planning_us,"sampling_24_us":sampling_us});
        fs::write(&path, serde_json::to_vec_pretty(&out).unwrap()).map_err(|e| e.to_string())?;
        println!(
            "seed={seed}, id={id}, legs={}, parts={}, prior failures={}, plan={planning_us:.1}us",
            chains.len(),
            doc.parts.len(),
            attempts.len()
        );
        return Ok(());
    }
    Err(format!("no admitted generated step: {attempts:?}"))
}
