use super::*;

use isocosm::Founding;

fn world() -> World {
    let genesis = Founding {
        seed: 97,
        sites: 2,
        population: 4,
        lineages: 2,
        ecology: false,
        ..Default::default()
    }
    .generate()
    .expect("small test genesis");
    let session = Session::new(genesis, Execution::Individuals).expect("test session");
    let sections = BTreeMap::from([
        ("mesocosm".to_string(), vec![1, 2]),
        ("unknown".to_string(), vec![0, 255]),
        ("remove-me".to_string(), vec![7]),
    ]);
    World::with_sections(session, sections)
}

fn host() -> FamilyHost {
    FamilyHost::new(SessionId([9; 16]), world()).expect("family host")
}

fn join(host: &mut FamilyHost) -> AcceptedRecord {
    let (_, _, record) = host
        .accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands: vec![Command::Join],
                final_tick: None,
                section_updates: BTreeMap::new(),
            })
        })
        .expect("native Join accepted");
    record.expect("Join creates a record")
}

fn assert_converged(host: &FamilyHost, replica: &FamilyReplica) {
    let hosted = host
        .read(|session, sections| {
            (
                session.entries.clone(),
                session.sim.state().tick,
                session.sim.state_hash(),
                isocosm::digest(sections),
            )
        })
        .unwrap();
    let copied = replica
        .read(|session, sections| {
            (
                session.entries.clone(),
                session.sim.state().tick,
                session.sim.state_hash(),
                isocosm::digest(sections),
            )
        })
        .unwrap();
    assert_eq!(copied, hosted);
    assert_eq!(
        replica.snapshot().unwrap().revision,
        host.revision().unwrap()
    );
}

#[test]
fn serialized_native_and_section_updates_converge() {
    let mut host = host();
    let checkpoint = Checkpoint::decode(&host.checkpoint().unwrap().encode().unwrap()).unwrap();
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();

    let (_, outcomes, record) = host
        .accept(|_, _| {
            Ok(HostAdmission {
                value: "joined",
                commands: vec![Command::Join],
                final_tick: None,
                section_updates: BTreeMap::from([("mesocosm".to_string(), Some(vec![3, 4, 5]))]),
            })
        })
        .unwrap();
    assert_eq!(outcomes.len(), 1);
    assert_eq!(record.as_ref().unwrap().sequence, 1);
    let record = AcceptedRecord::decode(&record.unwrap().encode().unwrap()).unwrap();
    replica.apply(record).unwrap();
    assert_converged(&host, &replica);
}

#[test]
fn section_only_updates_preserve_unknown_bytes_and_record_deletions() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    let (_, _, record) = host
        .accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands: Vec::new(),
                final_tick: None,
                section_updates: BTreeMap::from([
                    ("mesocosm".into(), Some(vec![8])),
                    ("unknown".into(), Some(vec![0, 255, 17])),
                    ("remove-me".into(), None),
                ]),
            })
        })
        .unwrap();
    let record = record.expect("section changes create a record");
    assert_eq!(record.native_suffix.len(), 0);
    assert_eq!(record.section_updates.get("remove-me"), Some(&None));
    replica.apply(record).unwrap();
    assert_converged(&host, &replica);
    let unknown = replica
        .read(|_, sections| sections.get("unknown").cloned())
        .unwrap();
    assert_eq!(unknown, Some(vec![0, 255, 17]));
}

#[test]
fn rejected_and_identical_updates_create_no_record_or_revision() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let original = host
        .read(|session, sections| (session.save(), sections.clone()))
        .unwrap();

    assert!(
        host.accept::<()>(|_, _| Err("intent refused".into()))
            .is_err()
    );
    let (_, _, identical) = host
        .accept(|_, sections| {
            Ok(HostAdmission {
                value: (),
                commands: Vec::new(),
                final_tick: None,
                section_updates: sections
                    .iter()
                    .map(|(key, bytes)| (key.clone(), Some(bytes.clone())))
                    .collect(),
            })
        })
        .unwrap();
    assert!(identical.is_none());
    assert_eq!(host.checkpoint().unwrap(), checkpoint);
    assert_eq!(
        host.read(|session, sections| (session.save(), sections.clone()))
            .unwrap(),
        original
    );

    let record = join(&mut host);
    assert_eq!(
        record.sequence, 1,
        "neither refusal nor no-op consumes sequence"
    );
}

#[test]
fn native_failure_after_an_accepted_candidate_command_rolls_back_everything() {
    let mut host = host();
    let before = host.world.replication_snapshot().unwrap();
    let err = host
        .accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands: vec![
                    Command::Join,
                    Command::Take {
                        participant: u64::MAX,
                        critter: u64::MAX,
                    },
                ],
                final_tick: None,
                section_updates: BTreeMap::from([("mesocosm".into(), Some(vec![99]))]),
            })
        })
        .unwrap_err();

    assert!(err.contains("is no participant"));
    assert_eq!(host.world.replication_snapshot().unwrap(), before);
    assert_eq!(host.sequence(), 0);
    assert_eq!(host.revision().unwrap(), before.revision);
}

#[test]
fn family_host_rejects_a_world_with_an_external_owner() {
    let world = world();
    let external_owner = world.share();
    assert!(
        FamilyHost::new(SessionId([9; 16]), world)
            .err()
            .unwrap()
            .contains("shared owners")
    );
    assert!(external_owner.revision().is_ok());
}

#[test]
fn direct_endpoint_world_commits_are_journaled_and_drain_frees_capacity() {
    let mut host = host();
    let initial = host.checkpoint().unwrap();
    let endpoint_world = host.world().share();
    let before = endpoint_world.cut().unwrap();
    endpoint_world
        .product_sections(before, |_, sections| {
            let mut sections = sections.clone();
            sections.insert("mesocosm".into(), vec![4, 5]);
            Ok(((), sections))
        })
        .unwrap();
    assert_eq!(host.sequence(), 1);
    assert!(host.outbox_bytes() > 0);
    let records = host.drain_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].parent_digest, initial.digest);
    assert_eq!(host.outbox_bytes(), 0);

    let replica = FamilyReplica::install(initial, &records, Execution::Individuals).unwrap();
    assert_eq!(
        replica.snapshot().unwrap(),
        host.world().replication_snapshot().unwrap()
    );
}

#[test]
fn hosted_world_refuses_branch_replacement_and_unreplayable_native_history() {
    let host = host();
    let before = host.world().replication_snapshot().unwrap();
    let error = host
        .world()
        .product(cut(&before.save), |session| {
            session.branch = "branch:replacement".into();
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("branch or genesis"));
    assert_eq!(host.world().replication_snapshot().unwrap(), before);
    assert_eq!(host.sequence(), 0);

    let error = host
        .world()
        .product(cut(&before.save), |session| {
            session.entries.push(Entry {
                id: "intent:forged".into(),
                tick: session.sim.state().tick,
                order: session.entries.len() as u64,
                command: Command::Join,
                outcome: "forged outcome".into(),
            });
            Ok(())
        })
        .unwrap_err();
    assert!(error.contains("native suffix entry differs from replay"));
    assert_eq!(host.world().replication_snapshot().unwrap(), before);
    assert_eq!(host.sequence(), 0);
    assert_eq!(host.outbox_bytes(), 0);
}

#[test]
fn bounded_outbox_refuses_publication_and_hosted_world_refuses_replica_writes() {
    let host = FamilyHost::new_with_limits(
        SessionId([9; 16]),
        world(),
        FamilyLimits {
            wire: WireLimits::default(),
            max_outbox_bytes: 1,
        },
    )
    .unwrap();
    let before = host.world().replication_snapshot().unwrap();
    let write_error = host
        .world()
        .put_section("new-section", vec![1])
        .unwrap_err();
    assert!(write_error.contains("outbox is full"));
    assert_eq!(host.world().replication_snapshot().unwrap(), before);
    assert_eq!(host.sequence(), 0);
    assert_eq!(host.outbox_bytes(), 0);

    let replay_error = host
        .world()
        .apply_replica_record(
            before.revision,
            before.save.native.entries.len(),
            &[],
            before.save.native.tick,
            &BTreeMap::new(),
            |_, _, _| Ok(()),
        )
        .unwrap_err();
    assert!(replay_error.contains("hosted world refuses replica replay"));
    assert_eq!(host.world().replication_snapshot().unwrap(), before);
}

#[test]
fn native_time_only_acceptance_is_a_replicated_record() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    let (_, _, record) = host
        .accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands: Vec::new(),
                final_tick: Some(1),
                section_updates: BTreeMap::new(),
            })
        })
        .unwrap();
    let record = record.expect("accepted native time creates a record");
    assert!(record.native_suffix.is_empty());
    assert_eq!(record.final_tick, 1);
    replica.apply(record).unwrap();
    assert_converged(&host, &replica);
}

#[test]
fn duplicate_gapped_reordered_and_checkpoint_resumed_records_are_checked() {
    let mut host = host();
    let initial = host.checkpoint().unwrap();
    let first = join(&mut host);
    let second = join(&mut host);
    let mut replica = FamilyReplica::install(initial, &[], Execution::Individuals).unwrap();

    let before = replica.snapshot().unwrap();
    assert!(replica.apply(second.clone()).unwrap_err().contains("gap"));
    assert_eq!(replica.snapshot().unwrap(), before);
    replica.apply(first.clone()).unwrap();
    assert!(replica.apply(first).unwrap_err().contains("duplicate"));
    replica.apply(second).unwrap();
    assert_converged(&host, &replica);

    let resumed_checkpoint = host.checkpoint().unwrap();
    let third = join(&mut host);
    let resumed = FamilyReplica::install(
        Checkpoint::decode(&resumed_checkpoint.encode().unwrap()).unwrap(),
        &[AcceptedRecord::decode(&third.encode().unwrap()).unwrap()],
        Execution::Individuals,
    )
    .unwrap();
    assert_converged(&host, &resumed);
}

#[test]
fn writer_can_resume_from_a_verified_checkpoint_and_suffix() {
    let mut original = host();
    let checkpoint = original.checkpoint().unwrap();
    let first = join(&mut original);
    let mut resumed = FamilyHost::resume(
        checkpoint.clone(),
        std::slice::from_ref(&first),
        Execution::Individuals,
        FamilyLimits::default(),
    )
    .unwrap();
    assert_eq!(resumed.sequence(), 1);
    assert_eq!(
        resumed.world().replication_snapshot().unwrap(),
        original.world().replication_snapshot().unwrap()
    );

    let second = join(&mut resumed);
    let replica =
        FamilyReplica::install(checkpoint, &[first, second], Execution::Individuals).unwrap();
    assert_eq!(
        replica.snapshot().unwrap(),
        resumed.world().replication_snapshot().unwrap()
    );
}

#[test]
fn initial_checkpoint_digest_binds_opaque_root_state() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let mut alternate_root = checkpoint.clone();
    alternate_root
        .world
        .sections
        .insert("unknown".into(), vec![42]);
    alternate_root.section_digest = isocosm::digest(&alternate_root.world.sections);
    let alternate_root = alternate_root.seal();
    let mut replica = FamilyReplica::install(alternate_root, &[], Execution::Individuals).unwrap();

    let record = join(&mut host);
    let before = replica.snapshot().unwrap();
    assert!(replica.apply(record).unwrap_err().contains("parent digest"));
    assert_eq!(replica.snapshot().unwrap(), before);
}

#[test]
fn unknown_family_format_versions_refuse_even_with_valid_digests() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let mut unknown_checkpoint = checkpoint.clone();
    unknown_checkpoint.format += 1;
    let unknown_checkpoint = unknown_checkpoint.seal();
    assert!(unknown_checkpoint.authentic());
    assert!(
        FamilyReplica::install(unknown_checkpoint, &[], Execution::Individuals)
            .err()
            .unwrap()
            .contains("unsupported family checkpoint format")
    );

    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    let mut unknown_record = join(&mut host);
    unknown_record.format += 1;
    let unknown_record = unknown_record.seal();
    assert!(unknown_record.authentic());
    let before = replica.snapshot().unwrap();
    assert!(
        replica
            .apply(unknown_record)
            .unwrap_err()
            .contains("unsupported family record format")
    );
    assert_eq!(replica.snapshot().unwrap(), before);
}

#[test]
fn record_and_checkpoint_tampering_and_revision_jumps_are_refused() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let mut replica =
        FamilyReplica::install(checkpoint.clone(), &[], Execution::Individuals).unwrap();
    let mut tampered_checkpoint = checkpoint;
    tampered_checkpoint.native_hash ^= 1;
    assert!(
        FamilyReplica::install(tampered_checkpoint, &[], Execution::Individuals)
            .err()
            .unwrap()
            .contains("checkpoint digest")
    );

    let valid = join(&mut host);
    let before = replica.snapshot().unwrap();
    let mut tampered = valid.clone();
    tampered.section_digest.push('0');
    assert!(
        replica
            .apply(tampered)
            .unwrap_err()
            .contains("digest mismatch")
    );
    assert_eq!(replica.snapshot().unwrap(), before);

    let mut jumped = valid;
    jumped.revision += 1;
    let jumped = jumped.seal();
    assert!(replica.apply(jumped).unwrap_err().contains("revision"));
    assert_eq!(replica.snapshot().unwrap(), before);
}

#[test]
fn valid_digest_with_wrong_native_poststate_is_atomic_and_recoverable() {
    let mut host = host();
    let checkpoint = host.checkpoint().unwrap();
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    let valid = join(&mut host);
    let before = replica.snapshot().unwrap();
    let sequence = replica.sequence();

    let mut wrong_hash = valid.clone();
    wrong_hash.native_hash ^= 1;
    let wrong_hash = wrong_hash.seal();
    assert!(wrong_hash.authentic());
    assert!(
        replica
            .apply(wrong_hash)
            .unwrap_err()
            .contains("postcondition mismatch")
    );
    assert_eq!(replica.snapshot().unwrap(), before);
    assert_eq!(replica.sequence(), sequence);

    replica.apply(valid).unwrap();
    assert_converged(&host, &replica);
}

#[test]
fn wire_limit_is_checked_before_decoding() {
    let checkpoint = host().checkpoint().unwrap();
    assert!(
        Checkpoint::decode_with_limits(&checkpoint.encode().unwrap(), WireLimits { max_bytes: 8 })
            .unwrap_err()
            .contains("wire limit")
    );
}
