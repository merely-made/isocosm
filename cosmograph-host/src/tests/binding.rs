// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph_overlay::{AttentionChange, EntityHandle, Intent, PlaceHandle, Pointable, Tick};

#[test]
fn preparing_a_mode_binding_keeps_native_world_subject_and_body() {
    let f = fixture();
    let current = window(&f, 0, Seat::Player, "mesocosm");
    let before_save = f.world.save().unwrap();
    let before_projection = current.projection().unwrap();
    let before_body = body(&f, f.subjects[0]);
    let mut requested = binding("eponym");
    requested.attention.played.insert(EntityHandle(Id::MAX));
    let prepared = current.prepare_binding(requested).unwrap();
    assert_eq!(prepared.requested.mode, "eponym");
    assert_ne!(prepared.requested.controls, current.binding().controls);
    assert_ne!(prepared.requested.view_defaults, current.binding().view_defaults);
    assert_eq!(prepared.requested.attention.played, BTreeSet::from([EntityHandle(f.subjects[0])]));
    assert_eq!(f.world.save().unwrap(), before_save, "binding preparation cannot reload or command native state");
    assert_eq!(current.projection().unwrap(), before_projection);
    assert_eq!(body(&f, f.subjects[0]), before_body);
    assert_eq!(current.binding().mode, "mesocosm", "an unaccepted binding is not a completed switch");
}

#[test]
fn attention_requests_are_stamped_for_the_named_participant_and_native_tick() {
    let f = fixture();
    let current = window(&f, 1, Seat::Player, "mesocosm");
    let change = AttentionChange::Examine(PlaceHandle(0));
    let request = current.attention_request(change).unwrap();
    assert_eq!(request.participant, f.participants[1]);
    assert_eq!(request.tick, Tick(f.world.cut().unwrap().tick));
    assert_eq!(request.intent, Intent::Attention(change));
}

#[test]
fn binding_attention_diff_uses_the_existing_contract_without_implicit_acceptance() {
    let f = fixture();
    let mut initial = binding("mesocosm");
    initial.attention.pinned.insert(Pointable::Entity(EntityHandle(f.subjects[0])));
    initial.attention.examined = Some(PlaceHandle(0));
    let current = Window::new(&f.world, f.participants[0], Seat::Player, initial).unwrap();
    let before = f.world.save().unwrap();
    let mut requested = binding("vtt");
    requested.attention.pinned.insert(Pointable::Entity(EntityHandle(f.subjects[1])));
    let prepared = current.prepare_binding(requested).unwrap();
    let expected = [
        AttentionChange::Unpin(Pointable::Entity(EntityHandle(f.subjects[0]))),
        AttentionChange::Pin(Pointable::Entity(EntityHandle(f.subjects[1]))),
        AttentionChange::StopExamining,
    ];
    assert_eq!(prepared.changes.len(), expected.len());
    for (request, change) in prepared.changes.iter().zip(expected) {
        assert_eq!(request.participant, f.participants[0]);
        assert_eq!(request.intent, Intent::Attention(change));
    }
    assert_eq!(f.world.save().unwrap(), before);
    assert_eq!(current.binding().attention.examined, Some(PlaceHandle(0)));
}

#[test]
fn pinning_a_hidden_subject_never_grants_visibility() {
    let f = fixture();
    let current = window(&f, 0, Seat::Player, "mesocosm");
    let before = current.projection().unwrap();
    let hidden = all_subjects(&f).into_iter().find(|id| !before.visible.shows(*id)).unwrap();
    let request = current.attention_request(AttentionChange::Pin(Pointable::Entity(EntityHandle(hidden)))).unwrap();
    assert_eq!(request.participant, f.participants[0]);
    assert_eq!(current.projection().unwrap(), before);
    let mut requested = binding("vtt");
    requested.attention.pinned.insert(Pointable::Entity(EntityHandle(hidden)));
    current.prepare_binding(requested).unwrap();
    assert_eq!(current.projection().unwrap(), before, "attention routing must not widen seat access");
}
