// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn identical_section_bytes_preserve_revision() {
    let f = fixture();
    let native = f.world.cut().unwrap();
    let before = f.world.revision().unwrap();
    f.world.put_section("mesocosm", vec![0, 255]).unwrap();
    assert_eq!(f.world.revision().unwrap(), before + 1);
    f.world.put_section("mesocosm", vec![0, 255]).unwrap();
    assert_eq!(f.world.revision().unwrap(), before + 1);
    f.world.put_section("mesocosm", vec![0, 254]).unwrap();
    assert_eq!(f.world.revision().unwrap(), before + 2);
    assert_eq!(f.world.cut().unwrap(), native);
}

#[test]
fn only_an_unshared_world_can_transfer_its_complete_contents() {
    let f = fixture();
    let shared = f.world.share();
    assert!(shared.into_parts().is_err());
    f.world.put_section("unknown", vec![0,255]).unwrap();
    let before = f.world.save().unwrap();
    let (session, sections) = f.world.into_parts().unwrap();
    assert_eq!(session.save(), before.native);
    assert_eq!(sections, before.sections);
}

#[test]
fn both_windows_read_one_accepted_world() {
    let f = fixture();
    let first = window(&f, 0, Seat::Player, "mesocosm");
    let second = window(&f, 1, Seat::CreativeObserver, "vtt");
    let before = f.world.cut().unwrap();
    f.world.command(Command::Join).unwrap();
    let accepted = f.world.cut().unwrap();
    assert_ne!(before.state_hash, accepted.state_hash, "native Join must change this world");
    assert_eq!(first.cut().unwrap(), accepted, "first window must observe the accepted native cut");
    assert_eq!(second.cut().unwrap(), accepted, "second window must observe the same accepted native cut");
    assert_eq!(accepted.entries, before.entries + 1);
}

#[test]
fn native_refusal_leaves_both_window_cuts_intact() {
    let f = fixture();
    let first = window(&f, 0, Seat::Player, "mesocosm");
    let second = window(&f, 1, Seat::Player, "eponym");
    let before = f.world.save().unwrap();
    let refusal = f.world.command(Command::Inspect(Id::MAX)).unwrap_err();
    assert!(!refusal.is_empty(), "native refusal must be retained");
    assert_eq!(f.world.save().unwrap(), before);
    assert_eq!(first.cut().unwrap(), second.cut().unwrap());
}

#[test]
fn native_take_updates_only_the_named_participants_window() {
    let f = fixture();
    let first = window(&f, 0, Seat::Player, "mesocosm");
    let second = window(&f, 1, Seat::Player, "eponym");
    let second_before = second.projection().unwrap();
    f.world.command(Command::Take {
        participant: f.participants[0].0,
        critter: f.subjects[1],
    }).unwrap();
    assert_eq!(first.projection().unwrap().access.subject.unwrap().0, f.subjects[1]);
    assert_eq!(second.projection().unwrap().access.subject.unwrap().0, f.subjects[1]);
    assert_eq!(second.projection().unwrap(), second_before);
    assert_eq!(first.cut().unwrap(), second.cut().unwrap());
}

#[test]
fn read_borrow_cannot_overlap_native_mutation() {
    let f = fixture();
    let before = f.world.save().unwrap();
    f.world.read(|_| {
        assert_eq!(f.world.command(Command::Join).unwrap_err(), "world is being read");
    }).unwrap();
    assert_eq!(f.world.save().unwrap(), before);
}
