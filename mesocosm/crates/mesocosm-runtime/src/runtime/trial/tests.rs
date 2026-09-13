// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn exact_baseline_reset_and_idle_replay_preserve_world_and_activity() {
    let source = World::new(7, 60);
    let original = state_hash(&source);
    let mut trial = Trial::new(&source).unwrap();
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
    let mut trial = Trial::new(&source).unwrap();
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
