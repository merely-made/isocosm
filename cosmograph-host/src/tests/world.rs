// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn current_product_matches_guarded_publication_and_preserves_refusal_atomicity() {
    let saved = fixture().world.save().unwrap();
    let guarded = World::resume(saved.clone(), Execution::Individuals).unwrap();
    let current = World::resume(saved, Execution::Individuals).unwrap();
    let guarded_before = guarded.cut().unwrap();
    let guarded_revision = guarded.revision().unwrap();
    let current_revision = current.revision().unwrap();

    let guarded_out = guarded.product(guarded_before, |session| session.command(Command::Join)).unwrap();
    let current_out = current.product_current(|session| session.command(Command::Join)).unwrap();
    assert_eq!(current_out, guarded_out);
    assert_eq!(current.save().unwrap(), guarded.save().unwrap());
    assert_eq!(current.revision().unwrap(), current_revision + 1);
    assert_eq!(guarded.revision().unwrap(), guarded_revision + 1);
    assert_eq!(current.cut().unwrap(), guarded.cut().unwrap());

    let before_save = current.save().unwrap();
    let before_revision = current.revision().unwrap();
    let before_cut = current.cut().unwrap();
    let refused: Result<(), String> = current.product_current(|session| {
        session.command(Command::Join)?;
        Err("planted current-product refusal".into())
    });
    assert!(refused.is_err());
    assert_eq!(current.save().unwrap(), before_save);
    assert_eq!(current.revision().unwrap(), before_revision);
    assert_eq!(current.cut().unwrap(), before_cut);
}

#[test]
fn current_product_obeys_commit_gate_and_read_borrow() {
    use std::{collections::BTreeMap, rc::Rc};
    use cosmograph::Session;
    use crate::world::CommitGate;

    struct Reject;
    impl CommitGate for Reject {
        fn before_publish(
            &self,
            _: &Session,
            _: &BTreeMap<String, Vec<u8>>,
            _: u64,
            _: &Session,
            _: &BTreeMap<String, Vec<u8>>,
            _: u64,
        ) -> Result<(), String> {
            Err("planted commit-gate refusal".into())
        }
    }

    let world = fixture().world;
    let before = world.save().unwrap();
    let revision = world.revision().unwrap();
    let cut = world.cut().unwrap();
    world.install_commit_gate(Rc::new(Reject)).unwrap();
    assert!(world.product_current(|session| session.command(Command::Join)).is_err());
    assert_eq!(world.save().unwrap(), before);
    assert_eq!(world.revision().unwrap(), revision);
    assert_eq!(world.cut().unwrap(), cut);

    world.read(|_| {
        assert_eq!(world.product_current(|session| session.command(Command::Join)).unwrap_err(),
            "world is being read");
    }).unwrap();
    assert_eq!(world.save().unwrap(), before);
    assert_eq!(world.revision().unwrap(), revision);
}

#[test]
fn native_cut_stays_exact_through_each_transaction_path_and_refusal() {
    let world = fixture().world;
    let exact = |world: &World| {
        let cut = world.cut().unwrap();
        world.read(|session| {
            assert_eq!(cut.state_hash, session.sim.state_hash());
            assert_eq!(cut.tick, session.sim.state().tick);
            assert_eq!(cut.entries, session.entries.len());
        }).unwrap();
        cut
    };
    let start = exact(&world);
    world.put_section("cut-control", vec![1]).unwrap();
    assert_eq!(exact(&world), start);
    world.command(Command::Join).unwrap();
    let joined = exact(&world);
    assert_ne!(joined, start);
    assert!(world.product::<()>(start, |_| panic!("stale admission must not run")).is_err());
    let refused: Result<(), String> = world.product(joined, |session| {
        session.command(Command::Join)?;
        Err("planted compound refusal".into())
    });
    assert!(refused.is_err());
    assert_eq!(exact(&world), joined);
    world.product(joined, |session| session.command(Command::Join)).unwrap();
    let product = exact(&world);
    assert_ne!(product, joined);
    world.product_sections(product, |session, sections| {
        session.command(Command::Join)?;
        Ok(((), sections.clone()))
    }).unwrap();
    assert_ne!(exact(&world), product);
}

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
        assert_eq!(f.world.product_current(|session| session.command(Command::Join)).unwrap_err(),
            "world is being read");
    }).unwrap();
    assert_eq!(f.world.save().unwrap(), before);
}
