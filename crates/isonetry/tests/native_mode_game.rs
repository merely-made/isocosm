// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../isocosm-vtt/tests/native_founding/fixture.rs"]
mod fixture;
use isocosm::{Execution, Session};
use isocosm_vtt::founding::{Campaign, NativeWorld};
use isometry_core::{MapDocument, TurnList};
use isonetry::{GameSnapshot, ModeGame};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn vtt_mode_bytes_keep_table_data_but_borrow_native_authority_on_resume() {
    let campaign = Campaign::from_draft_bench(fixture::genesis(), &fixture::draft(), Execution::Grouped)
        .unwrap().campaign;
    let state = GameSnapshot {
        map: MapDocument::new("inert", 3, 3), turns: TurnList::new(), roll_log: Vec::new(),
        journal: Vec::new(), inventories: BTreeMap::new(), generations: vec![fixture::record()],
        maps: BTreeMap::new(), active_map: None, world: fixture::draft().world,
        native: Some(NativeWorld::from_campaign(&campaign).unwrap()),
        native_table: None,
        clocks: BTreeMap::from([("inert".into(), 73)]), party_cap: 5,
        last_beats: Vec::new(), beat_seq: 19, applied_actions: BTreeSet::new(),
    };
    let game = ModeGame::capture(&state, campaign.content().clone(), campaign.origin().cloned());
    let bytes = game.encode().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("native").is_none() && json["table"].get("native").is_none(),
        "mode game bytes duplicated native Session");
    assert!(json["table"].get("world").is_none());
    assert!(json["table"]["generations"].as_array().unwrap().is_empty());
    let loaded = ModeGame::decode(&bytes).unwrap();
    let mut native = Session::load(campaign.save().native, Execution::Individuals).unwrap();
    native.advance(2).unwrap();
    let snapshot = loaded.snapshot(&native).unwrap();
    assert_eq!(snapshot.native.as_ref().unwrap().load(Execution::Grouped).unwrap().save().native,
        native.save(), "mode resume replaced borrowed native authority");
    assert_eq!(snapshot.clocks, state.clocks);
    assert_eq!(snapshot.map, state.map);
    assert_eq!(snapshot.party_cap, 5);
    assert_eq!(snapshot.beat_seq, 19);
    let mut invalid = json;
    invalid["version"] = 99.into();
    assert!(ModeGame::decode(&serde_json::to_vec(&invalid).unwrap()).unwrap_err()
        .contains("unsupported VTT game section version"));
}
