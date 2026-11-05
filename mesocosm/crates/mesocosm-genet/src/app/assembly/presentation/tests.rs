// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

pub(crate) fn stamp(sequence: u64) -> PresentedFrame {
    PresentedFrame {
        host: 1,
        sequence,
        width: 192,
        height: 144,
        layout_scale: 1.0,
        context_generation: cambium_rootstock::ContextGeneration(1),
    }
}
fn fixture() -> State {
    let mut state = State::new(HostConfig {
        bench_world: true,
        seed: 9,
        population: 256,
        dev: true,
        start_paused: true,
        ..Default::default()
    });
    state.refresh();
    state
}

#[test]
fn acceptance_publishes_prepare_time_subject_and_picture_while_newer_desired_inputs_wait() {
    let mut state = fixture();
    let native = state.host.runtime.world().save().unwrap().native;
    let a = state.picture.snapshot().unwrap();
    let subject_a = state.committed.clone().unwrap();
    let prepared = state.prepare_publication(stamp(1));
    assert_eq!(state.host.frames, 0);
    assert!(state.presentation.accepted.is_none());

    state.host.pan.x += 3.0;
    state.host.inspection.selected = Some(crate::section::BodySelection {
        organism: state.host.runtime.critter().unwrap(),
        part: cosmograph::schema::PartId(0),
        revision: isometer::mesh::BodyDependencyRevision(1),
    });
    state.refresh();
    let b = state.picture.snapshot().unwrap();
    let selection_b = state.host.inspection.selected;
    assert_ne!(a.fingerprint, b.fingerprint);
    assert!(
        prepared.is_live(),
        "camera/selection changes coalesce without revoking this source"
    );
    assert!(prepared.publish(stamp(1)));
    state.presented();
    assert_eq!(state.host.frames, 1);
    let accepted = state.presentation.accepted.as_ref().unwrap();
    assert_eq!(accepted.stamp, stamp(1));
    assert!(Rc::ptr_eq(accepted.facts.picture.as_ref().unwrap(), &a));
    assert_eq!(
        accepted.facts.subject.as_ref().unwrap().native,
        subject_a.native
    );
    assert_eq!(
        accepted.facts.subject.as_ref().unwrap().selection,
        subject_a.selection
    );
    assert!(Rc::ptr_eq(&state.picture.snapshot().unwrap(), &b));
    assert_eq!(
        state.host.inspection.selected, selection_b,
        "A does not expire desired B"
    );
    assert_eq!(state.host.runtime.world().save().unwrap().native, native);
    state.presented();
    assert_eq!(
        state.host.frames, 1,
        "an after-frame callback cannot manufacture another acceptance"
    );
}

#[test]
fn cancellation_or_a_replaced_owner_cannot_publish_the_prepared_product_cut() {
    let mut state = fixture();
    drop(state.prepare_publication(stamp(1)));
    state.presented();
    assert_eq!(state.host.frames, 0);
    let prepared = state.prepare_publication(stamp(2));
    let replacement = fixture();
    assert_eq!(
        state.host.runtime.revision().unwrap(),
        replacement.host.runtime.revision().unwrap()
    );
    state.host.runtime = replacement.host.runtime;
    state.refresh();
    assert!(!prepared.is_live());
    assert!(!prepared.publish(stamp(2)));
    state.presented();
    assert_eq!(state.host.frames, 0);
    assert!(state.presentation.accepted.is_none());
}

#[test]
fn source_withdrawal_revokes_but_an_unaccepted_attempt_never_advances_frames() {
    let mut state = fixture();
    let prepared = state.prepare_publication(stamp(1));
    state.picture.clear();
    assert!(!prepared.is_live());
    assert!(!prepared.publish(stamp(1)));
    state.presented();
    assert_eq!(state.host.frames, 0);
}
