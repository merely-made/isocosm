// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn exact_baseline_reset_and_idle_replay_preserve_world_and_activity() {
    let source = World::new(7, 60);
    let original = state_hash(&source);
    let mut trial = Trial::with_ceiling(&source, Trial::SHORT).unwrap();
    assert_eq!(trial.baseline_hash(), original);
    assert_eq!(trial.state_hash(), original);
    assert_eq!(trial.world(), &source);
    let mut batches = Vec::new();
    while trial.step() {
        batches.push(trial.activities().to_vec());
    }
    let final_hash = trial.state_hash();
    let history = trial.history().clone();
    let trace = trial.trace().to_vec();
    assert!(!batches.is_empty());
    assert!(batches.iter().flatten().next().is_some());
    assert_eq!(state_hash(&source), original);
    assert_eq!(trial.baseline_hash(), original);
    assert!(trace.iter().all(|intent| matches!(intent, Intent::Idle)));
    trial.reset();
    assert_eq!(trial.state_hash(), original);
    assert!(trial.activities().is_empty());
    let mut replay = Vec::new();
    while trial.step() {
        replay.push(trial.activities().to_vec());
    }
    assert_eq!(batches, replay);
    assert_eq!(trial.state_hash(), final_hash);
    assert_eq!(trial.history(), &history);
    assert_eq!(trial.trace(), trace.as_slice());
}

#[test]
fn activity_is_a_reading_of_ordinary_runtime_history_once_per_successful_step() {
    let source = World::new(42, 60);
    let mut trial = Trial::with_ceiling(&source, Trial::SHORT).unwrap();
    let mut ordinary = driver(&trial.baseline);
    let mut seen = std::collections::BTreeSet::new();
    let mut kinds = [0; 2];
    while trial.step() {
        let positions: BTreeMap<_, _> = ordinary
            .world()
            .organisms
            .iter()
            .map(|o| (o.id, o.position))
            .collect();
        assert_eq!(ordinary.step(1), 1);
        assert_eq!(trial.state_hash(), ordinary.state_hash());
        assert_eq!(trial.history(), ordinary.history());
        let batch = trial.activities().to_vec();
        assert_eq!(trial.activities(), batch);
        for activity in &batch {
            assert!(seen.insert(activity.sequence));
            let recorded = &ordinary.history().log().entries()[activity.sequence as usize];
            assert_eq!(recorded.tick, activity.tick);
            assert_eq!(recorded.record, activity.event);
            match recorded.record {
                Event::Moved { organism, from, to } => {
                    kinds[0] += 1;
                    assert_eq!(activity.source, organism);
                    assert_eq!(activity.target, None);
                    assert_eq!(activity.from, from);
                    assert_eq!(activity.to, to);
                },
                Event::Fed {
                    eater, from: donor, ..
                } => {
                    kinds[1] += 1;
                    assert_eq!(activity.source, eater);
                    assert_eq!(activity.target, Some(donor));
                    let before = positions.get(&donor).copied();
                    let after = ordinary
                        .world()
                        .organisms
                        .iter()
                        .find(|o| o.id == donor)
                        .map(|o| o.position);
                    assert_eq!(Some(activity.from), before.or(after));
                    let after = ordinary
                        .world()
                        .organisms
                        .iter()
                        .find(|o| o.id == eater)
                        .map(|o| o.position);
                    assert_eq!(
                        Some(activity.to),
                        after.or_else(|| positions.get(&eater).copied())
                    );
                },
                _ => panic!("unexpected activity"),
            }
        }
    }
    let before = trial.state_hash();
    let batch = trial.activities().to_vec();
    assert!(!trial.step());
    assert_eq!(trial.state_hash(), before);
    assert_eq!(trial.activities(), batch);
    assert!(trial.steps() <= 128);
    assert!(trial.steps() == 128 || trial.checkpoint().is_some());
    assert!(
        kinds[0] > 0 && kinds[1] > 0,
        "fixture must exercise movement and feeding: {kinds:?}"
    );
}

#[test]
fn bound_and_existing_checkpoint_stop_without_applying_or_clearing_activity() {
    let source = World::new(7, 0);
    let mut trial = Trial::new(&source).unwrap();
    trial.steps = MAX_TRIAL_STEPS;
    assert!(!trial.step());
    assert_eq!(trial.state_hash(), trial.baseline_hash());
    trial.reset();
    let held = trial.world().controlled().unwrap();
    trial.runtime.checkpoint = Some(crate::Checkpoint {
        tick: 0,
        occasion: crate::Occasion::Loss(crate::Loss {
            organism: held.id,
            lineage: held.species,
        }),
        heirs: Vec::new(),
    });
    assert!(!trial.step());
    assert!(trial.trace().is_empty());
    assert_eq!(trial.steps(), 0);
}

#[test]
fn advanced_worlds_are_refused_instead_of_losing_runtime_context() {
    let mut world = World::new(7, 0);
    world.apply(Intent::Idle);
    assert!(Trial::new(&world).is_err());
}

#[test]
fn uptake_matches_actual_soil_transfers_and_replays_without_touching_history() {
    let source = World::new(7, 60);
    let mut trial = Trial::with_ceiling(&source, Trial::SHORT).unwrap();
    let mut ordinary = driver(&source);
    let mut observed = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut total = 0;
    while trial.step() {
        assert_eq!(ordinary.step(1), 1);
        assert_eq!(trial.state_hash(), ordinary.state_hash());
        assert_eq!(trial.history(), ordinary.history());
        let expected: Vec<_> = ordinary
            .trial_flows
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                f.record.process == Process::Uptake
                    && f.record.source == Account::Soil
                    && f.record.amount_mg > 0
                    && f.record.to.is_some()
            })
            .collect();
        assert_eq!(trial.uptakes().len(), expected.len());
        for (actual, (index, record)) in trial.uptakes().iter().zip(expected) {
            assert_eq!(actual.record, *record);
            assert_eq!(actual.sequence, index as u64);
            assert_eq!(actual.tick, record.tick);
            assert_eq!(actual.organism, record.record.to.unwrap().organism);
            assert!(ids.insert((actual.tick, actual.sequence)));
            total += record.record.amount_mg;
        }
        observed.push(trial.uptakes().to_vec());
        assert_eq!(trial.uptakes(), observed.last().unwrap());
    }
    assert!(total > 0, "fixture exercises accepted soil uptake");
    let hash = trial.state_hash();
    let last = trial.uptakes().to_vec();
    assert!(!trial.step());
    assert_eq!(trial.uptakes(), last);
    assert_eq!(trial.state_hash(), hash);
    trial.reset();
    assert!(trial.uptakes().is_empty());
    let mut replay = Vec::new();
    while trial.step() {
        replay.push(trial.uptakes().to_vec());
    }
    assert_eq!(observed, replay);
    assert_eq!(trial.state_hash(), hash);
}

#[test]
fn uptake_filter_refuses_zero_internal_and_unrelated_transfers_and_labels_positions() {
    let source = World::new(7, 60);
    let mut trial = Trial::new(&source).unwrap();
    let record = loop {
        assert!(trial.step());
        if let Some(sample) = trial.uptakes().first() {
            break sample.record;
        }
    };
    let organism = record.record.to.unwrap().organism;
    let before = BTreeMap::from([(organism, [11, 22, 33])]);
    let present = uptake(record, 3, &before, trial.world()).unwrap();
    assert_eq!(present.position_basis, UptakePosition::AfterTick);
    assert_eq!(
        present.at,
        trial
            .world()
            .organisms
            .iter()
            .find(|o| o.id == organism)
            .map(|o| o.position)
    );
    let mut absent = trial.world().clone();
    absent.organisms.retain(|o| o.id != organism);
    let earlier = uptake(record, 3, &before, &absent).unwrap();
    assert_eq!(earlier.at, Some([11, 22, 33]));
    assert_eq!(earlier.position_basis, UptakePosition::BeforeTick);
    let unknown = uptake(record, 3, &BTreeMap::new(), &absent).unwrap();
    assert_eq!(unknown.at, None);
    assert_eq!(unknown.position_basis, UptakePosition::Unavailable);
    let mut zero = record;
    zero.record.amount_mg = 0;
    let mut internal = record;
    internal.record.source = Account::Substance;
    internal.record.destination = Account::Reserve;
    let mut unrelated = record;
    unrelated.record.process = Process::Upkeep;
    let mut unaddressed = record;
    unaddressed.record.to = None;
    for excluded in [zero, internal, unrelated, unaddressed] {
        assert!(uptake(excluded, 3, &before, trial.world()).is_none());
    }
}

#[test]
fn ordinary_runtime_does_not_capture_extra_flows_and_rejected_intent_does_not_invent_uptake() {
    let source = World::new(7, 60);
    let ordinary = Runtime::new(7, 60, 1);
    assert!(ordinary.trial_flows.is_none());
    let mut runtime = driver(&source);
    runtime.queue(Intent::TakeControl {
        organism: OrganismId(999999),
    });
    assert_eq!(runtime.step(1), 1);
    assert!(matches!(
        runtime.last_outcomes(),
        [mesocosm_core::Outcome::Rejected(_)]
    ));
    // Rejection does not freeze the ecology. Its legitimate producer transfers
    // remain available; none purport to be uptake by the rejected target.
    let before = source
        .organisms
        .iter()
        .map(|o| (o.id, o.position))
        .collect();
    for (index, record) in runtime.trial_flows.as_ref().unwrap().iter().enumerate() {
        if let Some(activity) = uptake(*record, index as u64, &before, runtime.world()) {
            assert_ne!(activity.organism, OrganismId(999999));
            assert_eq!(activity.record, *record);
        }
    }
}

/// The positive control for the raised ceiling: a trial at the shipped
/// ceiling reaches the world's own first epoch boundary, where the reckoning
/// feats are read from happens, and runs on past it. 128 stopped 872 ticks
/// short. Slow in a debug build, since it is a thousand real ticks.
///
/// A trial never answers a checkpoint, so a birth or death under the hand
/// before the boundary would stop it first; this names the occasion if so.
/// The boundary itself opens none for a trial, measured 2026-09-16.
#[test]
fn a_trial_at_the_shipped_ceiling_reaches_the_first_epoch_boundary() {
    let source = World::new(7, 60);
    let mut trial = Trial::new(&source).unwrap();
    while trial.world().epoch == 0 {
        if !trial.step() {
            panic!(
                "trial stopped at step {} of {MAX_TRIAL_STEPS}, epoch 0, checkpoint {:?}",
                trial.steps(),
                trial.checkpoint().map(|c| (c.tick, c.occasion))
            );
        }
    }
    assert_eq!(
        u64::from(trial.steps()),
        mesocosm_core::rules::DEFAULT_EPOCH_TICKS,
        "the boundary falls on the epoch's own budget"
    );
    assert!(
        !trial.runtime.reckoning().is_empty(),
        "the runtime reckoned the epoch the trial just closed"
    );
    assert!(trial.checkpoint().is_none(), "the boundary holds no trial");
    assert!(trial.step(), "and the trial runs on past it");
}
