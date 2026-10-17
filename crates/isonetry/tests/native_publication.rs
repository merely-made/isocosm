// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "../../isocosm-vtt/tests/native_founding/fixture.rs"]
mod fixture;
use isocosm::{Command, Execution, Founding, rules::{Causation, Effect, Process}};
use isocosm_vtt::founding::{Campaign, NativeWorld};
use isometry_core::{Facing, MapDocument, Token, TokenId, TurnList};
use isonetry::{
    GameEvent, GameSnapshot, HostSession, NativePublication, NativeTravelResolved,
    NetMessage, PeerId, RequestId, TravelTableResult, apply_game,
};
use std::collections::{BTreeMap, BTreeSet};

fn origin() -> GameSnapshot {
    let mut map = MapDocument::new("origin", 4, 4);
    map.tokens.push(Token {
        id: TokenId(1), at: (0, 0), facing: Facing::South,
        sprite: "observer".into(), owner: Some("A".into()),
    });
    GameSnapshot {
        map, turns: TurnList::new(), roll_log: Vec::new(), journal: Vec::new(),
        inventories: BTreeMap::new(), generations: Vec::new(), maps: BTreeMap::new(),
        active_map: None, world: Default::default(), native: None,
        native_table: None,
        clocks: BTreeMap::new(), party_cap: 4, last_beats: Vec::new(),
        beat_seq: 0, applied_actions: BTreeSet::new(),
    }
}

fn founded() -> HostSession {
    let profile = Founding { seed: 59, sites: 2, population: 4, lineages: 2,
        cohort_size: 2, ..Default::default() };
    let mut host = HostSession::new(origin());
    host.commit_native_campaign_for(
        &profile,
        &fixture::record(),
        Some(TokenId(1)),
        Execution::Grouped,
        isocosm_host::founding::Purpose::Bench,
        None,
    )
    .unwrap();
    host
}

#[test]
fn native_publication_is_the_host_snapshot_and_wire_replays_the_native_hash() {
    let host = founded();
    let native = host.state().native.as_ref().unwrap();
    let campaign = native.load(Execution::Individuals).unwrap();
    assert_eq!(campaign.reading().state_hash(), native.cursor().unwrap().state_hash);
    assert!(host.state().world.assertions().is_empty(), "legacy fold retained native assertions");
    assert_eq!(host.campaign().secret("secret-oath").unwrap().text, "HOST PRIVATE OATH");
    let message = host.on_connect(PeerId(1)).pop().unwrap().1;
    let bytes = postcard::to_allocvec(&message).unwrap();
    let decoded: NetMessage = postcard::from_bytes(&bytes).unwrap();
    let mut client = isonetry::ClientSession::new();
    client.on_message(decoded);
    assert_eq!(client.state(), Some(host.state()), "native snapshot wire receiver diverged");
    let public = serde_json::to_string(&client.state().unwrap().native.as_ref().unwrap()
        .load(Execution::Grouped).unwrap().save()).unwrap();
    assert!(!public.contains("HOST PRIVATE OATH"));
    assert_eq!(client.log_hash(), host.log_hash());
}

#[test]
fn native_publication_refuses_stale_parent_and_late_board_failure_atomically() {
    let host = founded();
    let before = host.state().clone();
    let previous = before.native.as_ref().unwrap();
    let old = previous.load(Execution::Grouped).unwrap();
    let admitted = old.with_commands(&[], 1).unwrap();
    let mut publication = NativePublication::update(previous, &admitted).unwrap();
    publication.expected.as_mut().unwrap().state_hash ^= 1;
    let mut receiver = before.clone();
    let error = apply_game(&mut receiver, &GameEvent::NativePublished(publication.clone()))
        .expect_err("stale native parent was accepted");
    assert!(format!("{error:?}").contains("stale parent"), "stale native parent was accepted");
    assert_eq!(receiver, before, "stale publication mutated receiver");
    publication.expected = Some(previous.cursor().unwrap());
    publication.activate = Some("missing-board".into());
    assert!(apply_game(&mut receiver, &GameEvent::NativePublished(publication)).is_err());
    assert_eq!(receiver, before, "late publication refusal mutated receiver");
}

#[test]
fn native_wire_refuses_a_forged_saved_hash_and_legacy_world_fallback() {
    let host = founded();
    let native = host.state().native.as_ref().unwrap();
    let bytes: Vec<u8> = serde_json::from_value(serde_json::to_value(native).unwrap()).unwrap();
    let mut body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let hash = body["native"]["state_hash"].as_u64().unwrap();
    body["native"]["state_hash"] = (hash ^ 1).into();
    let forged = serde_json::to_vec(&body).unwrap();
    let error = serde_json::from_value::<NativeWorld>(serde_json::to_value(forged).unwrap())
        .unwrap_err().to_string();
    assert!(error.contains("save state hash mismatch"), "forged native witness was accepted");
    let mut receiver = host.state().clone();
    let before = receiver.clone();
    let error = apply_game(&mut receiver, &GameEvent::World(
        isometry_campaign::WorldEvent::PartyMoved {
            party: "forged".into(), node: "ford".into(),
        })).unwrap_err();
    assert!(format!("{error:?}").contains("legacy world mutation"));
    assert_eq!(receiver, before, "legacy fallback mutated native campaign");
}

fn travel_host() -> (HostSession, u64, u64) {
    let mut genesis = fixture::genesis();
    let actor = 0;
    let from = genesis.population.get(actor).unwrap().place;
    let to = genesis.sites[&from].routes[0].to;
    genesis.rules.processes.insert("test:travel".into(), Process {
        port_feeding: None, goal_directives: Vec::new(),
        id: "test:travel".into(), causation: Causation::Choice, requires: Vec::new(),
        commitments: Vec::new(), effects: vec![Effect::Move { destination: to }],
        risk: None, target: None, period: None, priority: 0, need_account: None,
        need_below: 0, glyphs: BTreeSet::new(), invariants: BTreeSet::new(), note: false,
    });
    let campaign = Campaign::from_draft_bench(genesis, &fixture::draft(), Execution::Grouped)
        .unwrap().campaign;
    let mut host = HostSession::new(origin());
    let publication = NativePublication {
        expected: None, world: NativeWorld::from_campaign(&campaign).unwrap(),
        boards: campaign.content().lowered_maps().unwrap(), inventories: BTreeMap::new(),
        activate: Some(campaign.content().starting_map.clone()),
    };
    assert!(!host.local_event(GameEvent::NativePublished(publication)).is_empty());
    (host, actor, to)
}

fn command(actor: u64) -> Command {
    Command::Act { actor, target: None, process: "test:travel".into(), cause: None }
}

#[test]
fn native_travel_receivers_use_one_admitted_result_and_absolute_split_clocks() {
    let (host, actor, to) = travel_host();
    let campaign = host.state().native.as_ref().unwrap().load(Execution::Grouped).unwrap();
    let from = campaign.reading().state().population.get(actor).unwrap().resident_site().unwrap();
    let travel = campaign.reading().genesis().sites[&from].routes.iter()
        .find(|route| route.to == to).unwrap().travel;
    assert!(travel > 0);
    for ticks in [0, travel] { travel_receivers(ticks); }
}

fn travel_receivers(ticks: u64) {
    let (mut host, actor, to) = travel_host();
    let before = host.state().clone();
    let mut boards = before.maps.clone();
    boards.get_mut("river-map").unwrap().document.set_condition(TokenId(1), "exhaustion", 7);
    let table = TravelTableResult {
        map: boards["river-map"].document.clone(), turns: before.turns.clone(),
        boards, clocks: BTreeMap::from([("river-map".into(), 80), ("elsewhere".into(), 47)]),
        active: Some("river-map".into()), inventories: before.inventories.clone(), rolls: Vec::new(),
    };
    let out = host.commit_native_travel(RequestId::host(77), &[command(actor)], ticks,
        &BTreeSet::from([actor]), |_| Ok(table.clone())).unwrap();
    let NetMessage::Applied { event: GameEvent::NativeTravelResolved(result), .. } = &out[0].1 else {
        panic!("host did not publish one native travel result");
    };
    let resident = (ticks > 0).then_some(to);
    assert_eq!(result.members[&actor].resident, resident);
    let mut receiver = before.clone();
    apply_game(&mut receiver, &GameEvent::NativeTravelResolved(result.clone())).unwrap();
    assert_eq!(&receiver, host.state(), "native travel receiver diverged");
    assert_eq!(receiver.clocks, table.clocks, "travel receiver derived clocks");
    assert_eq!(receiver.map.condition_value(TokenId(1), "exhaustion"), 7);
    let once = receiver.clone();
    apply_game(&mut receiver, &GameEvent::NativeTravelResolved(result.clone())).unwrap();
    assert_eq!(receiver, once, "native travel double-applied");
    let event = GameEvent::NativeTravelResolved(result.clone());
    let decoded: GameEvent = postcard::from_bytes(&postcard::to_allocvec(&event).unwrap()).unwrap();
    let mut wire_receiver = before;
    apply_game(&mut wire_receiver, &decoded).unwrap();
    assert_eq!(wire_receiver, once);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let replay = once.native.as_ref().unwrap().load(mode).unwrap();
        let member = replay.reading().state().population.get(actor).unwrap();
        assert_eq!(member.resident_site(), resident);
        assert_eq!(member.journey.is_some(), ticks == 0);
        assert_eq!(replay.reading().state_hash(), result.publication.world.cursor().unwrap().state_hash);
    }
}

#[test]
fn native_travel_refuses_a_late_native_command_and_forged_member_without_mutation() {
    let (mut host, actor, _) = travel_host();
    let before = host.state().clone();
    let hash = host.log_hash();
    let commands = [command(actor), Command::Act {
        actor, target: None, process: "missing:process".into(), cause: None,
    }];
    let error = host.commit_native_travel(RequestId::host(78), &commands, 0,
        &BTreeSet::from([actor]), |_| panic!("refused native command reached table resolution"))
        .unwrap_err();
    assert!(error.contains("Command { index: 1 }") && error.contains("unknown process"),
        "native refused receipt was admitted");
    assert_eq!(host.state(), &before);
    assert_eq!(host.log_hash(), hash);
    let previous = before.native.as_ref().unwrap();
    let next = previous.load(Execution::Grouped).unwrap().with_commands(&[command(actor)], 0).unwrap();
    let result = NativeTravelResolved {
        request: RequestId::host(79), publication: NativePublication::update(previous, &next).unwrap(),
        members: BTreeMap::from([(actor, isonetry::TravelMember {
            resident: Some(u64::MAX), arrived: 0, alive: true,
            accounts: BTreeMap::new(), traits: BTreeSet::new(),
        })]), table: TravelTableResult { map: before.map.clone(), turns: before.turns.clone(),
            boards: before.maps.clone(), clocks: before.clocks.clone(),
            active: before.active_map.clone(), inventories: before.inventories.clone(), rolls: Vec::new() },
    };
    let mut receiver = before.clone();
    let error = apply_game(&mut receiver, &GameEvent::NativeTravelResolved(result)).unwrap_err();
    assert!(format!("{error:?}").contains("travel members differ"));
    assert_eq!(receiver, before, "forged member mutated receiver");
}
