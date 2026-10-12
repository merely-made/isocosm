// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../isometry-campaign/tests/native_founding/fixture.rs"]
mod fixture;
use isocosm::Execution;
use isometry_campaign::founding::{Campaign, NativeWorld};
use isometry_core::{MapDocument, TurnList};
use isonetry::{GameEvent, GameSnapshot, GameSourceHistory, HostSession, NativePublication};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn native_site_versions_replay_the_selected_prefix_and_preserve_the_live_source() {
    let origin = GameSnapshot {
        map: MapDocument::new("empty", 4, 4), turns: TurnList::new(), roll_log: Vec::new(),
        journal: Vec::new(), inventories: BTreeMap::new(), generations: Vec::new(),
        maps: BTreeMap::new(), active_map: None, world: Default::default(), native: None,
        clocks: BTreeMap::new(), party_cap: 4, last_beats: Vec::new(), beat_seq: 0,
        applied_actions: BTreeSet::new(),
    };
    let first = Campaign::from_draft(fixture::genesis(), &fixture::draft(), Execution::Grouped)
        .unwrap().campaign;
    let mut host = HostSession::new(origin.clone());
    host.local_event(GameEvent::NativePublished(NativePublication {
        expected: None, world: NativeWorld::from_campaign(&first).unwrap(),
        boards: Vec::new(), inventories: BTreeMap::new(), activate: None,
    }));
    let next = first.with_commands(&[], 2).unwrap();
    host.local_event(GameEvent::NativePublished(NativePublication::update(
        host.state().native.as_ref().unwrap(), &next).unwrap()));
    let live = host.state().clone();
    let source = GameSourceHistory::new(origin, host.history().clone());
    assert!(source.native_site_at(0, 0, Execution::Grouped).is_err());
    let old = source.native_site_at(1, 0, Execution::Individuals).unwrap();
    let new = source.native_site_at(2, 0, Execution::Grouped).unwrap();
    assert_eq!(old.load(Execution::Grouped).unwrap().save(), first.save(),
        "historical native site used current state");
    assert_eq!(new.load(Execution::Individuals).unwrap().save(), next.save());
    assert_eq!(old.cursor.tick, 0);
    assert_eq!(new.cursor.tick, 2);
    assert_eq!(host.state(), &live, "historical selection mutated authority");
    assert!(source.native_site_at(1, u64::MAX, Execution::Grouped).is_err());
}