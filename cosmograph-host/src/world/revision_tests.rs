// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::history::LifeHistory;

fn cached_cut(world: &World) -> Option<Cut> {
    world.0.borrow().native_cut.get()
}

fn replay_bytes(world: &World) {
    let saved = world.save().unwrap();
    let bytes = saved.encode().unwrap();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let resumed = World::resume(saved.clone(), mode).unwrap();
        assert_eq!(resumed.save().unwrap().encode().unwrap(), bytes);
        assert_ne!(
            resumed.native_revision().unwrap(),
            world.native_revision().unwrap()
        );
    }
}

#[test]
fn tokens_share_only_the_live_owner_and_do_not_compute_a_cut_or_change_a_save() {
    let world = crate::fixtures::fixture().world;
    let saved = world.save().unwrap();
    let bytes = saved.encode().unwrap();
    assert_eq!(cached_cut(&world), None);
    let token = world.native_revision().unwrap();
    assert_eq!(token.sequence(), 0);
    assert_eq!(token.clone(), token);
    assert!(token.same_source(&token.clone()));
    let shared = world.share();
    assert_eq!(shared.native_revision().unwrap(), token);
    assert_eq!(
        cached_cut(&world),
        None,
        "token reads must not compute a native witness"
    );
    assert_eq!(world.save().unwrap().encode().unwrap(), bytes);
    let copied = World::new(world.read(Clone::clone).unwrap());
    let resumed = World::resume(saved.clone(), Execution::Grouped).unwrap();
    let native = World::resume_native(saved.native.clone(), Execution::Individuals).unwrap();
    let replica =
        World::resume_replica(world.replication_snapshot().unwrap(), Execution::Grouped).unwrap();
    for fresh in [&copied, &resumed, &native, &replica] {
        assert_eq!(fresh.native_revision().unwrap().sequence(), 0);
        assert_ne!(fresh.native_revision().unwrap(), token);
        assert!(!fresh.native_revision().unwrap().same_source(&token));
        assert_eq!(fresh.save().unwrap().encode().unwrap(), bytes);
    }
    assert_ne!(
        copied.native_revision().unwrap(),
        resumed.native_revision().unwrap()
    );
    drop(shared);
    let (session, sections) = world.into_exclusive().unwrap().into_parts().unwrap();
    assert_eq!(
        session.save(),
        saved.native,
        "tokens hold no extra World owner or native IDs"
    );
    assert_eq!(sections, saved.sections);
}

#[test]
fn accepted_commands_time_branches_and_saved_metadata_advance_native_identity() {
    let world = crate::fixtures::fixture().world;
    let mut reference = world.read(Clone::clone).unwrap();
    let mut previous = world.native_revision().unwrap();
    world.command(Command::Join).unwrap();
    reference.command(Command::Join).unwrap();
    let joined = world.native_revision().unwrap();
    assert_eq!(joined.sequence(), previous.sequence() + 1);
    assert_ne!(joined, previous);
    assert!(joined.same_source(&previous));
    previous = joined;
    world.advance(2).unwrap();
    reference.advance(2).unwrap();
    assert_eq!(world.save().unwrap().native, reference.save());
    let advanced = world.native_revision().unwrap();
    assert_eq!(advanced.sequence(), previous.sequence() + 1);
    assert_ne!(advanced, previous);
    assert!(advanced.same_source(&previous));
    previous = advanced;

    let before_branch = world.cut().unwrap();
    world
        .product_current(|session| {
            *session =
                session.fork_at(session.sim.state().tick, "branch:native-revision".into())?;
            Ok(())
        })
        .unwrap();
    assert_eq!(world.cut().unwrap(), before_branch);
    let branched = world.native_revision().unwrap();
    assert_ne!(
        branched, previous,
        "branch identity is native authority even at an equal cut"
    );
    assert_eq!(branched.sequence(), previous.sequence() + 1);
    previous = branched;
    world
        .product_current(|session| {
            session.life_history = LifeHistory::Ticks;
            Ok(())
        })
        .unwrap();
    assert_ne!(world.native_revision().unwrap(), previous);
    assert_eq!(
        world.native_revision().unwrap().sequence(),
        previous.sequence() + 1
    );
    replay_bytes(&world);
}

#[test]
fn a_genesis_replacement_invalidates_even_when_native_state_is_equal() {
    let genesis = cosmograph::Founding {
        population: 4,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let world = World::new(Session::new(genesis, Execution::Individuals).unwrap());
    let token = world.native_revision().unwrap();
    let state = world.read(|session| session.sim.state().clone()).unwrap();
    world
        .product_current(|session| {
            let mut genesis = session.sim.genesis().clone();
            genesis.dynamics = Some(genesis.seed.wrapping_add(1));
            *session = Session::new(genesis, session.sim.mode())?;
            Ok(())
        })
        .unwrap();
    assert_eq!(
        world.read(|session| session.sim.state().clone()).unwrap(),
        state
    );
    assert_ne!(world.native_revision().unwrap(), token);
    assert_eq!(
        world.native_revision().unwrap().sequence(),
        token.sequence() + 1
    );
    replay_bytes(&world);
}

#[test]
fn mode_only_and_mixed_products_preserve_the_right_revision_and_cached_cut() {
    let world = crate::fixtures::fixture().world;
    let cut = world.cut().unwrap();
    let native = world.native_revision().unwrap();
    let saved = world.save().unwrap().native;
    let revision = world.revision().unwrap();
    world.put_section("mode-control", vec![1]).unwrap();
    assert_eq!(world.revision().unwrap(), revision + 1);
    assert_eq!(world.native_revision().unwrap(), native);
    world
        .product_sections(cut, |session, sections| {
            session.sim.set_mode(Execution::Grouped);
            let mut sections = sections.clone();
            sections.insert("mode-control".into(), vec![2]);
            Ok(((), sections))
        })
        .unwrap();
    assert_eq!(
        world.read(|session| session.sim.mode()).unwrap(),
        Execution::Grouped
    );
    assert_eq!(world.revision().unwrap(), revision + 2);
    assert_eq!(world.native_revision().unwrap(), native);
    assert_eq!(
        cached_cut(&world),
        Some(cut),
        "mode publication must retain the cached native cut"
    );
    assert_eq!(world.save().unwrap().native, saved);
    world
        .product_sections(cut, |_, sections| Ok(((), sections.clone())))
        .unwrap();
    assert_eq!(world.revision().unwrap(), revision + 2);
    assert_eq!(world.native_revision().unwrap(), native);
    world
        .product_sections(cut, |session, sections| {
            session.command(Command::Join)?;
            let mut sections = sections.clone();
            sections.insert("mode-control".into(), vec![3]);
            Ok(((), sections))
        })
        .unwrap();
    assert_eq!(world.revision().unwrap(), revision + 3);
    assert_eq!(
        world.native_revision().unwrap().sequence(),
        native.sequence() + 1
    );
    assert_ne!(world.native_revision().unwrap(), native);
    assert_eq!(cached_cut(&world), None);
    assert_ne!(world.cut().unwrap(), cut);
    replay_bytes(&world);
}

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
        Err("planted native revision gate refusal".into())
    }
}

#[test]
fn noops_refusals_and_read_guards_preserve_native_identity() {
    let world = crate::fixtures::fixture().world;
    let token = world.native_revision().unwrap();
    let cut = world.cut().unwrap();
    let before = world.replication_snapshot().unwrap();
    world.advance(0).unwrap();
    world.product_current(|_| Ok(())).unwrap();
    assert!(world.command(Command::Inspect(u64::MAX)).is_err());
    let refused: Result<(), String> = world.product(cut, |session| {
        session.command(Command::Join)?;
        Err("planted compound refusal".into())
    });
    assert!(refused.is_err());
    world
        .read(|_| {
            assert_eq!(world.native_revision().unwrap(), token);
            assert_eq!(
                world.command(Command::Join).unwrap_err(),
                "world is being read"
            );
        })
        .unwrap();
    world
        .product_current(|_| {
            assert_eq!(
                world.native_revision().unwrap_err(),
                "world is being advanced"
            );
            Ok(())
        })
        .unwrap();
    world.install_commit_gate(Rc::new(Reject)).unwrap();
    assert!(world.command(Command::Join).is_err());
    assert!(world.put_section("refused", vec![1]).is_err());
    assert!(
        world
            .product_sections(cut, |session, sections| {
                session.command(Command::Join)?;
                let mut sections = sections.clone();
                sections.insert("refused".into(), vec![1]);
                Ok(((), sections))
            })
            .is_err()
    );
    assert_eq!(world.native_revision().unwrap(), token);
    assert_eq!(world.replication_snapshot().unwrap(), before);
    assert_eq!(cached_cut(&world), Some(cut));
}

#[test]
fn a_recorded_refused_process_is_still_a_native_history_publication() {
    let fixture = crate::fixtures::fixture();
    let world = fixture.world;
    let before = world.native_revision().unwrap();
    let entries = world.read(|session| session.entries.len()).unwrap();
    let outcome = world
        .command(Command::Act {
            actor: fixture.subjects[0],
            target: None,
            process: "test:missing-process".into(),
            cause: None,
        })
        .unwrap();
    let receipt: cosmograph::simulation::Receipt = serde_json::from_str(&outcome).unwrap();
    assert!(!receipt.accepted());
    assert_eq!(
        world.read(|session| session.entries.len()).unwrap(),
        entries + 1
    );
    assert_ne!(world.native_revision().unwrap(), before);
    assert_eq!(
        world.native_revision().unwrap().sequence(),
        before.sequence() + 1
    );
    replay_bytes(&world);
}

#[test]
fn replica_verification_gates_native_identity_and_section_only_records_keep_the_cut() {
    let host = crate::fixtures::fixture().world;
    let initial = host.replication_snapshot().unwrap();
    let replica = World::resume_replica(initial.clone(), Execution::Grouped).unwrap();
    let token = replica.native_revision().unwrap();
    let cut = replica.cut().unwrap();
    host.put_section("replica-control", vec![1]).unwrap();
    let mode_only = host.replication_snapshot().unwrap();
    let updates = BTreeMap::from([("replica-control".into(), Some(vec![1]))]);
    assert!(
        replica
            .apply_replica_record(
                initial.revision,
                initial.save.native.entries.len(),
                &[],
                initial.save.native.tick,
                &updates,
                |_, _, _| Err("planted verify refusal".into())
            )
            .is_err()
    );
    assert_eq!(replica.native_revision().unwrap(), token);
    assert_eq!(replica.replication_snapshot().unwrap(), initial);
    replica
        .apply_replica_record(
            initial.revision,
            initial.save.native.entries.len(),
            &[],
            initial.save.native.tick,
            &updates,
            |_, _, _| Ok(()),
        )
        .unwrap();
    assert_eq!(replica.replication_snapshot().unwrap(), mode_only);
    assert_eq!(replica.native_revision().unwrap(), token);
    assert_eq!(cached_cut(&replica), Some(cut));
    assert!(
        replica
            .apply_replica_record(
                mode_only.revision,
                mode_only.save.native.entries.len(),
                &[],
                mode_only.save.native.tick,
                &BTreeMap::new(),
                |_, _, _| Ok(())
            )
            .is_err()
    );
    assert_eq!(replica.native_revision().unwrap(), token);

    host.product_sections(host.cut().unwrap(), |session, sections| {
        session.command(Command::Join)?;
        session.advance(2)?;
        let mut sections = sections.clone();
        sections.insert("replica-control".into(), vec![2]);
        Ok(((), sections))
    })
    .unwrap();
    let mixed = host.replication_snapshot().unwrap();
    let suffix = &mixed.save.native.entries[mode_only.save.native.entries.len()..];
    let updates = BTreeMap::from([("replica-control".into(), Some(vec![2]))]);
    assert!(
        replica
            .apply_replica_record(
                mode_only.revision,
                mode_only.save.native.entries.len(),
                suffix,
                mixed.save.native.tick,
                &updates,
                |_, _, _| Err("planted native verify refusal".into())
            )
            .is_err()
    );
    assert_eq!(replica.native_revision().unwrap(), token);
    assert_eq!(replica.replication_snapshot().unwrap(), mode_only);
    assert_eq!(cached_cut(&replica), Some(cut));
    replica
        .apply_replica_record(
            mode_only.revision,
            mode_only.save.native.entries.len(),
            suffix,
            mixed.save.native.tick,
            &updates,
            |_, _, _| Ok(()),
        )
        .unwrap();
    let accepted = replica.native_revision().unwrap();
    assert_ne!(accepted, token);
    assert_eq!(accepted.sequence(), token.sequence() + 1);
    assert_eq!(cached_cut(&replica), None);
    assert_eq!(replica.replication_snapshot().unwrap(), mixed);
    assert_eq!(replica.cut().unwrap(), host.cut().unwrap());
    assert_ne!(
        accepted,
        host.native_revision().unwrap(),
        "replica owners are distinct"
    );
    host.advance(1).unwrap();
    let timed = host.replication_snapshot().unwrap();
    replica
        .apply_replica_record(
            mixed.revision,
            mixed.save.native.entries.len(),
            &[],
            timed.save.native.tick,
            &BTreeMap::new(),
            |_, _, _| Ok(()),
        )
        .unwrap();
    assert_eq!(
        replica.native_revision().unwrap().sequence(),
        accepted.sequence() + 1
    );
    assert_eq!(replica.replication_snapshot().unwrap(), timed);
    replay_bytes(&replica);
}
