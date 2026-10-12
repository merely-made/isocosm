// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::save::{WorldSave, FORMAT_VERSION};

#[test]
fn unknown_mode_bytes_survive_native_resume_and_resave() {
    let f = fixture();
    let mut saved = f.world.save().unwrap();
    let bytes = vec![0, 255, 128, b'{', b'\n', b'}', 0, 19];
    saved.sections.insert("uninstalled-mode".into(), bytes.clone());
    saved.sections.insert("mesocosm".into(), vec![1, 2, 3]);
    let encoded = saved.encode().unwrap();
    for execution in [Execution::Individuals, Execution::Grouped] {
        let resumed = World::resume(WorldSave::decode(&encoded).unwrap(), execution).unwrap();
        let resaved = resumed.save().unwrap();
        assert_eq!(resaved.native, saved.native, "native save authority must pass through replay");
        assert_eq!(resaved.sections.get("uninstalled-mode"), Some(&bytes),
            "uninstalled mode bytes must remain unread and unchanged");
        assert_eq!(resaved.sections, saved.sections);
        assert_eq!(resaved.encode().unwrap(), encoded);
    }
}

#[test]
fn resume_binds_the_named_participant_instead_of_the_first_one() {
    let f = fixture();
    let saved = f.world.save().unwrap();
    let resumed = World::resume(saved, Execution::Grouped).unwrap();
    let second = Window::new(&resumed, f.participants[1], Seat::Player, binding("mesocosm")).unwrap();
    let projected = second.projection().unwrap();
    assert_eq!(projected.access.participant, f.participants[1]);
    assert_eq!(projected.access.subject.unwrap().0, f.subjects[1],
        "resume must retain the specifically named second participant's subject");
    let embodiment = resumed.read(|session| {
        session.sim.state().population.get(f.subjects[1]).unwrap().clone()
    }).unwrap();
    assert_eq!(embodiment, body(&f, f.subjects[1]));
}

#[test]
fn corrupt_native_witness_cannot_publish_an_open_world() {
    let f = fixture();
    let mut saved = f.world.save().unwrap();
    saved.native.state_hash ^= 1;
    assert!(World::resume(saved, Execution::Grouped).is_err());
}

#[test]
fn unknown_envelope_format_refuses_before_native_resume() {
    let f = fixture();
    let mut saved = f.world.save().unwrap();
    saved.format = FORMAT_VERSION + 1;
    assert!(World::resume(saved, Execution::Grouped).is_err());
}

#[test]
fn native_saved_entry_point_retains_the_exact_current_native_save() {
    let f = fixture();
    let saved = f.world.save().unwrap().native;
    let resumed = World::resume_native(saved.clone(), Execution::Individuals).unwrap();
    let resaved = resumed.save().unwrap();
    assert_eq!(resaved.native, saved);
    assert!(resaved.sections.is_empty());
}
