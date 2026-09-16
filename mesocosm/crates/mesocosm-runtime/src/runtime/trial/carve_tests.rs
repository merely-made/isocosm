// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn fixture() -> (World, [i32; 3]) {
    let mut source = World::new(7, 0);
    let at = (-16..=16)
        .find_map(|x| {
            (-16..=16).find_map(|z| {
                let y = source.ground().surface(x, z)?;
                (y >= 1).then_some([x, y, z])
            })
        })
        .expect("seeded terrain has removable surface matter");
    let id = source.controlled_id().unwrap();
    source
        .organisms
        .iter_mut()
        .find(|o| o.id == id)
        .unwrap()
        .position = [at[0], at[1] + 1, at[2]];
    assert!(source.in_reach(at));
    (source, at)
}

#[test]
fn carve_projects_only_positive_history_and_dirty_terrain() {
    let (source, at) = fixture();
    let mut trial = Trial::new(&source).unwrap();
    trial.drain_ground_dirty();
    let revision = trial.world().ground().revision();
    assert!(trial.carve(at, 1));
    assert!(matches!(trial.last_outcomes(), [Outcome::Carved { removed, .. }] if *removed > 0));
    assert_eq!(trial.carves().len(), 1);
    let carve = trial.carves()[0].clone();
    assert_eq!(carve.at, at);
    assert_eq!(carve.organism, source.controlled_id().unwrap());
    let recorded = &trial.history().log().entries()[carve.sequence as usize];
    assert_eq!(carve.tick, recorded.tick);
    assert_eq!(
        recorded.record,
        Event::Carved {
            organism: carve.organism,
            at,
            removed: carve.removed
        }
    );
    assert!(trial.world().ground().revision() > revision);
    assert!(!trial.drain_ground_dirty().is_empty());
    assert_eq!(trial.carves(), &[carve]);
    assert_eq!(trial.runtime.queued_len(), 0);

    // Repeating the same operation removes nothing and emits no positive fact.
    let revision = trial.world().ground().revision();
    assert!(trial.carve(at, 1));
    assert!(matches!(
        trial.last_outcomes(),
        [Outcome::Carved { removed: 0, .. }]
    ));
    assert!(trial.carves().is_empty());
    assert_eq!(trial.world().ground().revision(), revision);
    assert!(trial.drain_ground_dirty().is_empty());
    // Invalid radius is an ordinary core rejection, with its own applied tick.
    assert!(trial.carve(at, 0));
    assert!(matches!(trial.last_outcomes(), [Outcome::Rejected(_)]));
    assert!(trial.carves().is_empty());
    assert_eq!(trial.world().ground().revision(), revision);
    assert!(trial.drain_ground_dirty().is_empty());
    assert_eq!(trial.runtime.queued_len(), 0);
    assert!(trial.step());
    assert_eq!(trial.trace().last(), Some(&Intent::Idle));
}

#[test]
fn carve_refuses_before_queue_at_bound_checkpoint_and_numeric_overflow() {
    let (source, at) = fixture();
    let mut trial = Trial::new(&source).unwrap();
    trial.steps = MAX_TRIAL_STEPS;
    assert!(!trial.carve(at, 1));
    assert_eq!(trial.runtime.queued_len(), 0);
    assert!(trial.trace().is_empty());
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
    assert!(!trial.carve(at, 1));
    assert_eq!(trial.runtime.queued_len(), 0);
    assert!(trial.trace().is_empty());
    trial.reset();
    assert!(!trial.carve([i32::MAX; 3], 2));
    assert!(!trial.carve([i32::MIN; 3], 2));
    assert_eq!(trial.runtime.queued_len(), 0);
    assert_eq!(trial.state_hash(), trial.baseline_hash());
    assert!(trial.step());
    assert_eq!(trial.trace(), &[Intent::Idle]);
}

#[test]
fn mixed_idle_and_carve_trace_matches_ordinary_runtime_and_exact_reset() {
    let (source, at) = fixture();
    let original = state_hash(&source);
    let trace = [
        Intent::Idle,
        Intent::Carve { at, radius: 1 },
        Intent::Carve { at, radius: 0 },
        Intent::Idle,
    ];
    let mut trial = Trial::new(&source).unwrap();
    let mut ordinary = driver(&source, &History::new());
    let mut batches = Vec::new();
    for intent in &trace {
        assert!(match *intent {
            Intent::Idle => trial.step(),
            Intent::Carve { at, radius } => trial.carve(at, radius),
            _ => unreachable!(),
        });
        ordinary.queue(intent.clone());
        assert_eq!(ordinary.step(1), 1);
        assert_eq!(trial.last_outcomes(), ordinary.last_outcomes());
        assert_eq!(trial.state_hash(), ordinary.state_hash());
        assert_eq!(trial.history(), ordinary.history());
        assert_eq!(trial.runtime.queued_len(), 0);
        batches.push((
            trial.activities().to_vec(),
            trial.uptakes().to_vec(),
            trial.carves().to_vec(),
        ));
    }
    assert!(batches.iter().any(|(_, _, c)| !c.is_empty()));
    let final_hash = trial.state_hash();
    let history = trial.history().clone();
    assert_eq!(trial.trace(), &trace);
    assert_eq!(state_hash(&source), original);
    trial.reset();
    assert_eq!(trial.world(), &source);
    assert_eq!(trial.state_hash(), original);
    assert!(trial.carves().is_empty());
    assert!(trial.last_outcomes().is_empty());
    for (intent, expected) in trace.iter().zip(&batches) {
        assert!(match *intent {
            Intent::Idle => trial.step(),
            Intent::Carve { at, radius } => trial.carve(at, radius),
            _ => unreachable!(),
        });
        assert_eq!(
            &(
                trial.activities().to_vec(),
                trial.uptakes().to_vec(),
                trial.carves().to_vec()
            ),
            expected
        );
    }
    assert_eq!(trial.state_hash(), final_hash);
    assert_eq!(trial.history(), &history);
    assert_eq!(trial.trace(), &trace);
}
