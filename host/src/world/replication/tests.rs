// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn checkpoint_and_native_time_and_sections_publish_atomically() {
    let host = crate::fixtures::fixture().world;
    host.put_section("unknown", vec![0, 255]).unwrap();
    let before = host.replication_snapshot().unwrap();
    let replica = World::resume_replica(before.clone(), Execution::Grouped).unwrap();
    let original_cut = replica.cut().unwrap();
    host.product_sections(host.cut().unwrap(), |session, sections| {
        session.command(Command::Join)?;
        session.advance(2)?;
        let mut next = sections.clone();
        next.insert("product".into(), vec![3, 4]);
        Ok(((), next))
    }).unwrap();
    let after = host.replication_snapshot().unwrap();
    let suffix = &after.save.native.entries[before.save.native.entries.len()..];
    let updates = BTreeMap::from([("product".into(), Some(vec![3, 4]))]);
    let apply = |verify: fn(&Session, &BTreeMap<String, Vec<u8>>, u64) -> Result<(), String>| {
        replica.apply_replica_record(before.revision, before.save.native.entries.len(),
            suffix, after.save.native.tick, &updates, verify)
    };
    assert!(apply(|_, _, _| Err("planted final verification refusal".into())).is_err());
    assert_eq!(replica.replication_snapshot().unwrap(), before);
    assert_eq!(replica.cut().unwrap(), original_cut);
    apply(|_, _, _| Ok(())).unwrap();
    assert_eq!(replica.replication_snapshot().unwrap(), after);
    assert_eq!(replica.cut().unwrap(), host.cut().unwrap());
    assert!(apply(|_, _, _| Ok(())).is_err());
    let delete = BTreeMap::from([("unknown".into(), None)]);
    replica.apply_replica_record(after.revision, after.save.native.entries.len(), &[],
        after.save.native.tick, &delete, |_, sections, _| {
            assert!(!sections.contains_key("unknown")); Ok(())
        }).unwrap();
}

#[test]
fn unchanged_transactions_and_malformed_records_do_not_advance_revision() {
    let world = crate::fixtures::fixture().world;
    let before = world.replication_snapshot().unwrap();
    world.advance(0).unwrap();
    world.product_sections(world.cut().unwrap(), |_, sections| Ok(((), sections.clone()))).unwrap();
    assert_eq!(world.replication_snapshot().unwrap(), before);
    let updates = BTreeMap::new();
    assert!(world.apply_replica_record(before.revision, before.save.native.entries.len(), &[],
        before.save.native.tick, &updates, |_, _, _| Ok(())).is_err());
    world.command(Command::Join).unwrap();
    let after = world.replication_snapshot().unwrap();
    let mut entry = after.save.native.entries.last().unwrap().clone();
    entry.outcome.push_str("tampered");
    let replica = World::resume_replica(before.clone(), Execution::Individuals).unwrap();
    assert!(replica.apply_replica_record(before.revision, before.save.native.entries.len(),
        &[entry], after.save.native.tick, &updates, |_, _, _| Ok(())).is_err());
    assert_eq!(replica.replication_snapshot().unwrap(), before);
    let mut max = before;
    max.revision = u64::MAX;
    let max = World::resume_replica(max, Execution::Individuals).unwrap();
    assert!(max.put_section("new", vec![1]).is_err());
    assert_eq!(max.section("new").unwrap(), None);
}
