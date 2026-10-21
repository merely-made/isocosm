// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Equivalence controls for the owned atomic spatial-clock API.

#[path = "ecology_independence_879/fixture.rs"]
#[allow(dead_code)]
mod native_fixture;

use isocosm::{Execution, Session, spatial::{BirthProfile, BirthResolution, Preparation, SpatialAdvance,
    BirthMedium, BirthSupport}};
use isometer_space::places::Rules;

fn source() -> Session {
    let candidate = native_fixture::candidate_with_played("test:atomic-spatial-clock", 1);
    Session::new(candidate.genesis, Execution::Grouped).unwrap()
}

fn birth_source() -> Session {
    let candidate = native_fixture::candidate_with_played("test:atomic-spatial-clock", 1);
    let mut genesis = candidate.genesis;
    genesis.rules.birth_profiles.insert("test:atomic-birth-profile".into(), BirthProfile {
        process: "test:atomic-spatial-clock".into(), places: Rules::default(),
        clearance: [[-1, -1, -1], [1, 1, 1]], medium: BirthMedium::AirOrWater,
        support: BirthSupport::Free,
    });
    Session::new(genesis, Execution::Grouped).unwrap()
}

#[test]
fn default_birth_refusals_expand_to_the_same_explicit_native_history() {
    let mut explicit = birth_source();
    let source = explicit.clone();
    let mut inputs = Vec::new();
    let expected = loop {
        match explicit.try_advance_birth_spatial_with_flows(1, &[], &[], &[], &inputs).unwrap() {
            SpatialAdvance::Birth(pending) => inputs.push(BirthResolution {
                request: pending.request, geometry: Err("test: unbound parent".into()),
            }),
            SpatialAdvance::Advanced(out) => break out,
            _ => panic!("fixture needs only birth answers"),
        }
        assert!(inputs.len() <= 16);
    };
    assert!(!inputs.is_empty());
    let mut batched = source.clone();
    let fallback = isocosm::spatial::BirthFallback {
        known_parents: Default::default(), refusal: "test: unbound parent".into(),
    };
    let (SpatialAdvance::Advanced(actual), automatic) = batched.try_advance_birth_spatial_with_fallback(
        1, &[], &[], &[], &[], &fallback).unwrap() else { panic!("all refusals should expand") };
    assert_eq!(actual, expected);
    assert_eq!(automatic, inputs);
    assert_eq!(batched.save(), explicit.save(), "ordinary receipts, order, ids and witnesses");
    let restored = Session::load(batched.save(), Execution::Grouped).unwrap();
    assert_eq!(restored.save(), batched.save(), "replay requires no fallback policy");

    let mut known = source;
    let before = known.save();
    let fallback = isocosm::spatial::BirthFallback {
        known_parents: [inputs[0].request.parent].into(), refusal: "test: unbound parent".into(),
    };
    let (SpatialAdvance::Birth(pending), automatic) = known.try_advance_birth_spatial_with_fallback(
        1, &[], &[], &[], &[], &fallback).unwrap() else { panic!("known parent requires its provider") };
    assert_eq!(pending.request, inputs[0].request);
    assert!(automatic.is_empty());
    assert_eq!(known.save(), before);
    assert!(known.try_advance_birth_spatial_with_fallback(0, &[], &[], &[], &[], &fallback).is_err());
    assert_eq!(known.save(), before, "failed discovery cannot publish");
}

#[test]
fn atomic_advance_matches_guarded_preparation_and_publish() {
    let source = source();
    let mut guarded = source.clone();
    let mut atomic = source;

    let Preparation::Ready(prepared) = guarded.prepare_birth_spatial(1, &[], &[], &[], &[]).unwrap()
        else { panic!("fixture should produce a ready one-tick clock") };
    let expected_arrivals = prepared.arrivals.clone();
    let expected_steps = prepared.steps.clone();
    let expected_births = prepared.births.clone();
    let expected_order = prepared.order.clone();
    let expected = prepared.publish(&mut guarded).unwrap();

    let SpatialAdvance::Advanced(actual) = atomic
        .try_advance_birth_spatial_with_flows(1, &[], &[], &[], &[]).unwrap()
        else { panic!("fixture should produce an advanced one-tick clock") };
    assert_eq!(actual, expected);
    assert_eq!(atomic.save(), guarded.save());
    assert_eq!(atomic.sim.spatial_arrivals(), expected_arrivals);
    assert_eq!(atomic.sim.spatial_steps(), &expected_steps);
    assert_eq!(atomic.sim.spatial_births(), expected_births);
    assert_eq!(atomic.sim.spatial_order(), expected_order);
}

#[test]
fn atomic_error_preserves_source_and_guarded_preparation_still_refuses_stale_publish() {
    let mut atomic = source();
    let before = atomic.save();
    assert!(atomic.try_advance_birth_spatial_with_flows(0, &[], &[], &[], &[]).is_err());
    assert_eq!(atomic.save(), before);

    let mut guarded = source();
    let Preparation::Ready(prepared) = guarded.prepare_birth_spatial(1, &[], &[], &[], &[]).unwrap()
        else { panic!("fixture should produce a ready one-tick clock") };
    guarded.advance(1).unwrap();
    let changed = guarded.save();
    assert!(prepared.publish(&mut guarded).is_err());
    assert_eq!(guarded.save(), changed);
}

#[test]
fn pending_birth_leaves_both_sources_unchanged_and_retry_matches_guarded_path() {
    let source = birth_source();
    let mut guarded = source.clone();
    let mut atomic = source;
    let original = guarded.save();

    let mut refusals = Vec::new();
    for _ in 0..16 {
        let staged = guarded.prepare_birth_spatial(1, &[], &[], &[], &refusals).unwrap();
        let attempted = atomic.try_advance_birth_spatial_with_flows(
            1, &[], &[], &[], &refusals).unwrap();
        match (staged, attempted) {
            (Preparation::Birth(expected), SpatialAdvance::Birth(actual)) => {
                assert_eq!(expected.request, actual.request);
                assert_eq!(guarded.save(), original);
                assert_eq!(atomic.save(), original);
                assert!(!refusals.iter().any(|r: &BirthResolution| r.request.child == expected.request.child));
                refusals.push(BirthResolution { request: expected.request,
                    geometry: Err("test: no authored placement".into()) });
            },
            (Preparation::Ready(prepared), SpatialAdvance::Advanced(actual)) => {
                assert!(!refusals.is_empty(), "must exercise a genuine pending birth");
                assert_eq!(actual, prepared.publish(&mut guarded).unwrap());
                assert_eq!(atomic.save(), guarded.save());
                assert_eq!(atomic.sim.spatial_births(), guarded.sim.spatial_births());
                assert_eq!(atomic.sim.spatial_order(), guarded.sim.spatial_order());
                return;
            },
            _ => panic!("guarded and atomic preparation diverged"),
        }
    }
    panic!("birth retry failed to make finite progress");
}
