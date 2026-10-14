// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../isometry-campaign/tests/native_founding/fixture.rs"]
mod fixture;
#[path = "../../isometry-campaign/tests/native_setup/fixture.rs"]
mod setup_fixture;
use isocosm::Execution;
use isometry_campaign::founding::{Campaign, NativeWorld};
use isometry_core::{Facing, MapDocument, Token, TokenId, TurnList};
use isonetry::{GameSnapshot, HostSession, ClientSession, PeerId, NetMessage};
use std::collections::{BTreeMap, BTreeSet};

fn origin() -> GameSnapshot {
    let mut map = MapDocument::new("origin", 4, 4);
    map.tokens.push(Token { id: TokenId(1), at: (0, 0), facing: Facing::South,
        sprite: "observer".into(), owner: Some("A".into()) });
    GameSnapshot { map, turns: TurnList::new(),
        roll_log: vec![], journal: vec![], inventories: BTreeMap::new(), generations: vec![],
        maps: BTreeMap::new(), active_map: None, world: Default::default(), native: None,
        native_table: None,
        clocks: BTreeMap::new(), party_cap: 4, last_beats: vec![], beat_seq: 0,
        applied_actions: BTreeSet::new() }
}

#[test]
fn inspected_setup_is_one_native_host_publication_wire_and_saved_replay() {
    let setup = setup_fixture::setup(Execution::Grouped);
    let record = fixture::record();
    let expected = NativeWorld::from_campaign(&Campaign::from_setup(&setup, &record, 16).unwrap().campaign).unwrap();
    let mut host = HostSession::new(origin());
    host.commit_native_setup(&setup, &expected, &record, Some(TokenId(1))).unwrap();
    assert_eq!(host.state().native.as_ref().unwrap(), &expected, "host published a different world from the preview");
    assert!(host.state().world.assertions().is_empty());
    assert_eq!(host.state().maps["river-map"].document.ground.width(), 4, "inert authored board was discarded");
    let message = host.on_connect(PeerId(1)).pop().unwrap().1;
    let bytes = postcard::to_allocvec(&message).unwrap();
    let decoded: NetMessage = postcard::from_bytes(&bytes).unwrap();
    let mut client = ClientSession::new(); client.on_message(decoded);
    assert_eq!(client.state(), Some(host.state()), "attached world diverged at the real receiver");
    let saved = expected.load(Execution::Individuals).unwrap().save();
    assert_eq!(Campaign::load(saved, Execution::Grouped).unwrap().reading().authored_site("shrine").unwrap().0, 1000);
}

#[test]
fn changed_preview_and_late_native_attachment_refuse_without_any_host_commit() {
    let setup = setup_fixture::setup(Execution::Individuals);
    let record = fixture::record();
    let expected = NativeWorld::from_campaign(&Campaign::from_setup(&setup, &record, 16).unwrap().campaign).unwrap();
    let mut host = HostSession::new(origin());
    let before = host.state().clone(); let before_hash = host.log_hash(); let before_seq = host.seq();
    let mut changed = setup.clone(); changed.founding.seed ^= 1;
    let error = host.commit_native_setup(&changed, &expected, &record, Some(TokenId(1)))
        .expect_err("uninspected native preview was published");
    assert!(error.contains("preview changed"));
    assert_eq!(host.state(), &before); assert_eq!(host.log_hash(), before_hash); assert_eq!(host.seq(), before_seq);
    let mut invalid = setup.clone();
    invalid.attachments.insert("shrine".into(), isocosm::site_authoring::Attachment::Existing(0));
    let error = host.commit_native_setup(&invalid, &expected, &record, Some(TokenId(1))).unwrap_err();
    assert!(error.contains("another authored place"));
    assert_eq!(host.state(), &before, "late attachment refusal mutated host snapshot");
    assert_eq!(host.log_hash(), before_hash); assert_eq!(host.seq(), before_seq);
    assert!(host.campaign().secret("secret-oath").is_none(), "refused founding leaked private content into host state");
}
