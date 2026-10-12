// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm_overlay::{AttentionChange, EntityHandle, PlaceHandle, Pointable, WindowHandle};

#[test]
fn admitted_mode_commit_preserves_world_subject_body_and_the_other_window() {
    let f = fixture();
    let mut first_binding = binding("mesocosm");
    first_binding.attention.examined = Some(PlaceHandle(0));
    let mut first = Window::attach(&f.world, f.participants[0], Seat::Player, first_binding).unwrap();
    let mut second_binding = binding("vtt");
    second_binding.attention.examined = Some(PlaceHandle(1));
    let second = Window::attach(&f.world, f.participants[0], Seat::Player, second_binding).unwrap();
    let body_before = body(&f, f.subjects[0]);
    let other_before = second.binding().clone();
    let id_before = first.handle().unwrap();
    let mut requested = binding("eponym");
    requested.attention.examined = Some(PlaceHandle(2));
    requested.attention.played.insert(EntityHandle(Id::MAX));
    requested.attention.care.insert(Pointable::Entity(EntityHandle(Id::MAX)));
    let prepared = first.prepare_commit(requested).unwrap();
    let entries = f.world.cut().unwrap().entries;
    first.commit_binding(prepared).unwrap();
    assert_eq!(first.handle().unwrap(), id_before);
    assert_eq!(first.binding().mode, "eponym", "native acceptance must publish the local mode binding");
    assert_eq!(first.binding().attention.played, BTreeSet::from([EntityHandle(f.subjects[0])]));
    assert!(!first.binding().attention.care.contains(&Pointable::Entity(EntityHandle(Id::MAX))));
    assert_eq!(second.binding(), &other_before);
    assert_eq!(body(&f, f.subjects[0]), body_before);
    assert_eq!(first.cut().unwrap(), second.cut().unwrap());
    assert_eq!(f.world.cut().unwrap().entries, entries + 1);
    assert_eq!(f.world.player_attention(f.participants[0]).unwrap().examined,
        BTreeSet::from([PlaceHandle(1), PlaceHandle(2)]));
}

#[test]
fn invalid_target_and_stale_core_commits_leave_native_and_local_binding_unchanged() {
    let f = fixture();
    let mut current = Window::attach(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let mut requested = binding("eponym");
    requested.attention.pinned.insert(Pointable::Entity(EntityHandle(Id::MAX)));
    let prepared = current.prepare_commit(requested).unwrap();
    let saved = f.world.save().unwrap();
    let local = current.binding().clone();
    assert!(current.commit_binding(prepared).is_err(), "native invalid target must refuse a prepared core binding");
    assert_eq!(f.world.save().unwrap(), saved, "refusal must leave shared native history and witness untouched");
    assert_eq!(current.binding(), &local, "refusal must not publish prepared mode or controls");
    let stale = current.prepare_commit(binding("vtt")).unwrap();
    f.world.command(Command::Join).unwrap();
    let saved = f.world.save().unwrap();
    assert!(current.commit_binding(stale).is_err(), "a stale prepared core binding must refuse");
    assert_eq!(f.world.save().unwrap(), saved);
    assert_eq!(current.binding(), &local);
}

#[test]
fn foreign_prepared_binding_and_request_cannot_change_another_window() {
    let f = fixture();
    let mut first = Window::attach(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let second = Window::attach(&f.world, f.participants[1], Seat::Player, binding("vtt")).unwrap();
    let before = f.world.save().unwrap();
    let foreign = second.prepare_commit(binding("eponym")).unwrap();
    assert!(first.commit_binding(foreign).is_err());
    let request = second.window_attention_request(AttentionChange::Examine(PlaceHandle(0))).unwrap();
    assert!(first.apply_attention(request).is_err());
    assert_eq!(f.world.save().unwrap(), before);
    assert_eq!(first.binding().mode, "mesocosm");
}

#[test]
fn admitted_close_withdraws_only_that_window_and_unchanged_region_does_not_log() {
    let f = fixture();
    let mut first = Window::attach(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let second = Window::attach(&f.world, f.participants[0], Seat::Player, binding("vtt")).unwrap();
    let unchanged = first.window_attention_request(AttentionChange::StopExamining).unwrap();
    let before = f.world.save().unwrap();
    first.apply_attention(unchanged).unwrap();
    assert_eq!(f.world.save().unwrap(), before, "an unchanged region is presentation, not a native watch entry");
    let request = first.window_attention_request(AttentionChange::Examine(PlaceHandle(0))).unwrap();
    first.apply_attention(request).unwrap();
    first.close().unwrap();
    assert!(first.projection().is_err());
    assert!(second.projection().is_ok());
    assert!(f.world.player_attention(f.participants[0]).unwrap().examined.is_empty());
}

#[test]
fn active_native_save_resume_preserves_opaque_sections_and_exact_named_window() {
    let f = fixture();
    let first = Window::attach(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let second = Window::attach(&f.world, f.participants[1], Seat::Player, binding("eponym")).unwrap();
    let handle = second.handle().unwrap();
    assert_ne!(first.handle().unwrap(), handle);
    let mut saved = f.world.save().unwrap();
    saved.sections.insert("uninstalled-mode".into(), vec![0, 255, 128, b'{', b'}', 0]);
    let resumed = World::resume(saved.clone(), Execution::Grouped).unwrap();
    assert_eq!(resumed.save().unwrap(), saved, "core resume cannot attach, close or rewrite replayed native windows");
    let restored = Window::restore(&resumed, f.participants[1], Seat::Player, handle, second.binding().clone()).unwrap();
    assert_eq!(restored.projection().unwrap().access.subject.unwrap().0, f.subjects[1]);
    assert_eq!(restored.handle().unwrap(), handle);
    assert!(Window::restore(&resumed, f.participants[0], Seat::Player, handle, binding("eponym")).is_err());
    assert!(Window::restore(&resumed, f.participants[1], Seat::Player, handle, binding("vtt")).is_err());
    assert_eq!(resumed.save().unwrap(), saved);
    // No fabricated stale identity: close a genuinely issued native window.
    let mut restored = restored;
    restored.close().unwrap();
    assert!(Window::restore(&resumed, f.participants[1], Seat::Player, WindowHandle(handle.0), binding("eponym")).is_err());
}

#[test]
fn ordinary_core_attention_hides_known_native_pins_outside_the_seat_cut() {
    let f = fixture();
    let current = Window::new(&f.world, f.participants[0], Seat::Player, binding("mesocosm")).unwrap();
    let visible = current.projection().unwrap().visible;
    let hidden = all_subjects(&f).into_iter().find(|id| !visible.shows(*id) && !f.participants.iter().any(|p| p.0 == *id)).unwrap();
    let hidden_point = Pointable::Entity(EntityHandle(hidden));
    let mut requested = binding("vtt");
    requested.attention.pinned.insert(hidden_point);
    let attached = Window::attach(&f.world, f.participants[0], Seat::Player, requested).unwrap();
    assert!(f.world.player_attention(f.participants[0]).unwrap().pinned.contains(&hidden_point),
        "trusted native attention must preserve the admitted hidden pin");
    let ordinary = attached.projection().unwrap();
    assert!(!ordinary.visible.shows(hidden));
    assert!(!ordinary.attention.pinned.contains(&hidden_point), "ordinary attention must obey the same seat cut as the view");
}
