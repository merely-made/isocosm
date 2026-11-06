// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! CPU representation comparison, outside native scene admission/extraction.
//! This does not measure rendering or claim that a store should be faster.

use super::*;
use serde_json::{Value, json};
use std::{hint::black_box, time::Instant};

const BODY_COUNTS: [u64; 3] = [3, 12, 48];
const WARMUPS: usize = 5;
const SAMPLES: usize = 51;

fn elapsed_ns(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

/// The same lightweight metadata traversal on both body vectors. Exact full
/// body equivalence is checked separately, outside all timed regions.
fn traverse(bodies: &[PlacedBody]) -> u64 {
    bodies.iter().fold(0u64, |mut sum, body| {
        let body = black_box(body);
        sum = sum.wrapping_add(body.id);
        for at in body.at {
            sum = sum.wrapping_mul(31).wrapping_add(at as u64);
        }
        for tint in body.tint {
            sum = sum.wrapping_add(u64::from(tint.to_bits()));
        }
        for count in [
            body.document.living().count(),
            body.parts.len(),
            body.materials.len(),
            body.palette.len(),
            body.walks.len(),
            usize::from(body.physical.is_some()),
            usize::from(body.alive),
        ] {
            sum = sum.wrapping_mul(31).wrapping_add(count as u64);
        }
        sum
    })
}

fn distribution(samples: &[u64]) -> Value {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let p95 = (ordered.len() * 95).div_ceil(100) - 1;
    json!({
        "sample_count": samples.len(), "samples_ns": samples,
        "median_ns": ordered[ordered.len() / 2], "p95_ns": ordered[p95],
        "min_ns": ordered[0], "max_ns": ordered[ordered.len() - 1],
    })
}

#[derive(Default)]
struct Timings {
    vector_traversal: Vec<u64>,
    vector_clone: Vec<u64>,
    capture: Vec<u64>,
    query_materialize: Vec<u64>,
    materialized_traversal: Vec<u64>,
}

fn vector_sample(scene: &SiteScene, timings: &mut Timings) -> Vec<PlacedBody> {
    let start = Instant::now();
    black_box(traverse(black_box(&scene.bodies)));
    timings.vector_traversal.push(elapsed_ns(start));
    let start = Instant::now();
    let copied = black_box(scene.bodies.clone());
    timings.vector_clone.push(elapsed_ns(start));
    copied
}

fn component_sample(scene: &SiteScene, timings: &mut Timings) -> Vec<PlacedBody> {
    let start = Instant::now();
    let columns = black_box(ComponentScene::capture(black_box(scene)));
    timings.capture.push(elapsed_ns(start));
    let start = Instant::now();
    let materialized = black_box(columns.bodies().unwrap());
    timings.query_materialize.push(elapsed_ns(start));
    let start = Instant::now();
    black_box(traverse(black_box(&materialized)));
    timings.materialized_traversal.push(elapsed_ns(start));
    materialized
}

fn optional_json(receipt: &Value) {
    let Some(path) = std::env::var_os("MESOCOSM_COMPONENT_COMPARISON_JSON") else {
        return;
    };
    let path = std::path::PathBuf::from(path);
    let write = || -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_vec_pretty(receipt)?)?;
        Ok(())
    };
    if let Err(why) = write() {
        // Output location is optional; a bad path must not change correctness.
        eprintln!("component comparison JSON {}: {why}", path.display());
    }
}

#[test]
#[ignore = "explicit CPU component-store comparison; run release with --ignored --exact --nocapture"]
fn component_scene_comparison() {
    let mut rows = Vec::new();
    for body_count in BODY_COUNTS {
        let session = native_with_body_count(body_count);
        let saved = serde_json::to_vec(&session.save()).unwrap();
        let before_hash = session.sim.state_hash();
        // Shared native extraction, skinning, terrain and all setup are excluded.
        let admitted = scene(&session);
        assert_eq!(admitted.bodies.len(), body_count as usize);
        let expected_traversal = traverse(&admitted.bodies);
        for _ in 0..WARMUPS {
            let columns = black_box(ComponentScene::capture(black_box(&admitted)));
            let materialized = black_box(columns.bodies().unwrap());
            black_box(admitted.bodies.clone());
            black_box(traverse(&admitted.bodies));
            same_roster(&materialized, &admitted.bodies);
        }
        let mut timings = Timings::default();
        for sample in 0..SAMPLES {
            // Alternate which representation runs first, retaining both outputs
            // until after timing. Allocation is included, destruction excluded.
            let (copied, materialized) = if sample % 2 == 0 {
                let copied = vector_sample(&admitted, &mut timings);
                let materialized = component_sample(&admitted, &mut timings);
                (copied, materialized)
            } else {
                let materialized = component_sample(&admitted, &mut timings);
                let copied = vector_sample(&admitted, &mut timings);
                (copied, materialized)
            };
            same_roster(&copied, &admitted.bodies);
            same_roster(&materialized, &admitted.bodies);
            assert_eq!(traverse(&materialized), expected_traversal);
        }
        let combined: Vec<_> = timings
            .capture
            .iter()
            .zip(&timings.query_materialize)
            .map(|(capture, query)| capture.saturating_add(*query))
            .collect();
        assert_eq!(serde_json::to_vec(&session.save()).unwrap(), saved);
        assert_eq!(session.sim.state_hash(), before_hash);
        rows.push(json!({
            "declared_bodies": body_count, "actual_bodies": admitted.bodies.len(),
            "native_population_including_site_bodies": session.sim.state().population.count(),
            "native_state_hash": before_hash,
            "component_rows": ComponentScene::capture(&admitted).counts(),
            "exact_body_equivalence": true, "native_save_unchanged": true,
            "retained_vector_traversal": distribution(&timings.vector_traversal),
            "retained_vector_owned_clone": distribution(&timings.vector_clone),
            "component_capture": distribution(&timings.capture),
            "component_query_and_owned_materialization": distribution(&timings.query_materialize),
            "component_materialized_vector_traversal": distribution(&timings.materialized_traversal),
            "sum_of_component_capture_and_materialization": distribution(&combined),
        }));
    }
    let receipt = json!({
        "kind": "mesocosm-component-scene-comparison", "version": 1,
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "native_fixture_seed": 7, "native_cohort_size": 8,
        "native_residents_explicitly_colocated_at_site": 0,
        "declared_body_counts": BODY_COUNTS, "warmups": WARMUPS, "samples_per_count": SAMPLES,
        "timed_scope": "CPU representation work after shared native SiteScene extraction; metadata traversal, owned vector clone, sparse capture, ordered joins plus owned materialization",
        "excluded": ["native admission and scene extraction", "terrain and skin generation", "correctness assertions", "output serialization", "GPU rendering", "destruction of returned stores/vectors"],
        "percentile_rule": "nearest rank p95; odd-sample median",
        "qualification": "No speedup premise. Retained traversal and component capture/materialization are different stages; owned vector clone is an additional materialization control.",
        "cases": rows,
    });
    println!(
        "component_scene_comparison {}",
        serde_json::to_string(&receipt).unwrap()
    );
    optional_json(&receipt);
}
