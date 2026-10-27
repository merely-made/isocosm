// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Opt-in matched cold benchmark from the retained, unchosen shore draw.
use super::*;
use crate::{Command, Execution, Session, spatial::Preparation};
use std::time::Instant;

#[test]
#[ignore = "set ISOCOSM_COLD_TRACE to the retained shore trace; explicit benchmark"]
fn retained_cold_failures_keep_their_search_census_with_fewer_posture_evaluations() {
    let path = std::env::var("ISOCOSM_COLD_TRACE").expect("retained trace path");
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let saved: crate::history::Saved = serde_json::from_value(value["native"].clone()).unwrap();
    assert_eq!(saved.genesis.seed, 3580977844374872680);
    let mut session = Session::new(saved.genesis, Execution::Individuals).unwrap();
    for entry in saved.entries {
        let Command::AdvanceBirthSpatial {
            ticks,
            terrain,
            arrivals,
            births,
            ..
        } = &entry.command
        else {
            session.command(entry.command).unwrap();
            continue;
        };
        let Preparation::Step(pending) = session
            .prepare_birth_spatial(*ticks, terrain, &[], arrivals, births)
            .unwrap()
        else {
            panic!("first native food-choice cut absent")
        };
        let sim = &pending.sim;
        for actor in [13, 26] {
            for (walk, f) in &sim
                .genesis()
                .rules
                .spatial_templates
                .as_ref()
                .unwrap()
                .foraging
            {
                let at = sim.spatial_pose_at_start(actor).unwrap();
                let t = sim.genesis().rules.spatial_templates.as_ref().unwrap();
                let profile = &sim.genesis().rules.step_profiles[walk];
                let foods = sim
                    .food_candidates(actor, &sim.genesis().rules.processes[&f.feed], f)
                    .into_iter()
                    .filter(|(_, d)| *d <= f.search)
                    .filter_map(|(id, _)| sim.current_placed_boxes(id, at.site).map(|b| (id, b)))
                    .collect();
                let input = Input {
                    balance: embodiment::balance_for(&sim, sim.body_at_start(actor).unwrap())
                        .unwrap(),
                    actor,
                    site: at.site,
                    terrain: sim
                        .site_geometry(
                            at.site,
                            &embodiment::site_window(t, at.pose.at),
                            profile.places,
                        )
                        .unwrap(),
                    document: sim
                        .body_at_start(actor)
                        .unwrap()
                        .body
                        .as_ref()
                        .unwrap()
                        .clone(),
                    before: at.pose.clone(),
                    profile: profile.clone(),
                    foods,
                    blocked: sim
                        .state()
                        .spatial_poses
                        .keys()
                        .copied()
                        .filter(|id| *id != actor)
                        .filter_map(|id| sim.current_placed_boxes(id, at.site))
                        .flatten()
                        .collect(),
                    reach: f.reach,
                    nodes: f.nodes,
                    walking: embodiment::walking_rules(sim),
                };
                let mut results = Vec::new();
                let mut measures = Vec::new();
                for reuse in [false, true] {
                    let mut checks = embodiment::StrideChecks::new(
                        &input.terrain.volume,
                        &input.document,
                        &input.blocked,
                    );
                    if !reuse {
                        checks = checks.without_reuse();
                    }
                    let start = Instant::now();
                    results.push(find_using(&input, &mut checks).unwrap());
                    measures.push((start.elapsed().as_micros(), checks.counts()));
                }
                assert_eq!(results[0], results[1]);
                assert!(results[0].route.is_none() && results[0].exhausted);
                assert!(measures[1].1.1 < measures[0].1.1);
                println!(
                    "{}",
                    serde_json::json!({"actor":actor,"walk":walk,"report":results[1],
                    "without_reuse_us":measures[0].0,"reuse_us":measures[1].0,
                    "without_reuse":measures[0].1,"reused":measures[1].1})
                );
            }
        }
        return;
    }
    panic!("retained draw has no physical round");
}
