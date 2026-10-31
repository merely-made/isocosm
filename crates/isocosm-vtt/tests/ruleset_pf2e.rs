#![cfg(feature = "endpoint")]
//! Pathfinder 2e creation at the table: the seat that controls a body drafts
//! its character through the endpoint, the draft and the finished sheet
//! survive family replay, and a 5e founding never admits a draft.
use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm::Execution;
use isocosm_host::world::World;
use isocosm_overlay::{ParticipantHandle, Seat};
use isocosm_vtt::{
    SHEETS, VttEndpoint,
    founding::{Content, TableMetadata},
    ruleset::{SECTION, State, token},
};
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};
#[path = "native_founding/fixture.rs"]
mod content_fixture;
#[path = "support/table.rs"]
mod table;

fn founded(world: &World, edition: &str) {
    let draft = content_fixture::draft();
    let content = Content {
        id: "pf2e-bench".into(),
        name: "Pathfinder Bench".into(),
        maps: draft.maps,
        storylets: draft.world.storylets,
        rewards: vec![],
        starting_map: draft.starting_map,
        final_storylet: draft.final_storylet,
        table: TableMetadata::default(),
        edition: edition.into(),
    };
    content.validate().unwrap();
    let bytes = serde_json::to_vec(&serde_json::json!({"content": content, "origin": null})).unwrap();
    world.put_section("isocosm-vtt:campaign", bytes).unwrap();
}

fn snapshot(endpoint: &mut VttEndpoint) -> ProjectionSnapshot {
    let offer = endpoint.describe().projections.into_iter()
        .find(|o| o.request.session.0 == SHEETS).unwrap();
    endpoint.snapshot(offer.request).unwrap()
}

fn click(endpoint: &mut VttEndpoint, card: &str, intent: &str) -> IntentResult {
    let snap = snapshot(endpoint);
    let target = snap.scene.active_items_in_order().into_iter()
        .find(|(_, item)| snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == card)
        .unwrap().0;
    endpoint.invoke(IntentInvocation { session: snap.session, target,
        observed_epoch: snap.scene.epoch, observed_revision: snap.scene.revision,
        intent: intent.into(), payload: vec![] }).unwrap()
}

fn state(host: &FamilyHost) -> State {
    host.read(|_, sections| State::read(sections).unwrap().unwrap()).unwrap()
}

/// A dwarf fighter, as the card's choice buttons spell it.
const DWARF_FIGHTER: &[&str] = &[
    "dwarf", "listed", "str", "rock-dwarf", "warrior", "str", "con", "fighter", "athletics", "str",
    "dex", "con", "wis", "int", "acrobatics", "medicine", "survival", "crafting",
    "dwarven-lore", "society", "vicious-swing",
];

#[test]
fn a_player_drafts_a_legal_character_and_it_survives_replay() {
    let table::Table { world, dm, player, bodies } = table::table_with_sites(1);
    founded(&world, "pf2e-srd");
    let mut host = FamilyHost::new(SessionId([81; 16]), world).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    host.drain_records();
    let mut seat = VttEndpoint::open(host.world(), player, Seat::Player).unwrap();
    let card = format!("sheet:{}", bodies[1]);
    assert_eq!(click(&mut seat, &card, "vtt.sheet-create"), IntentResult::Accepted);
    for value in DWARF_FIGHTER {
        let intent = format!("vtt.draft-{value}");
        assert_eq!(click(&mut seat, &card, &intent), IntentResult::Accepted, "{value}");
    }
    // Taking back a choice and making it again is part of the record.
    assert_eq!(click(&mut seat, &card, "vtt.draft-undo"), IntentResult::Accepted);
    assert_eq!(click(&mut seat, &card, "vtt.draft-vicious-swing"), IntentResult::Accepted);
    assert_eq!(click(&mut seat, &card, "vtt.draft-finish"), IntentResult::Accepted);
    let finished = state(&host);
    let id = token(bodies[1]).unwrap();
    assert_eq!(finished.sheets[&id].system, "pf2e-srd");
    assert_eq!(finished.sheets[&id].int("hp_current"), Some(23));
    assert_eq!(finished.drafts[&id].choices.len(), DWARF_FIGHTER.len());
    // The player sees no control over the other body.
    let other = format!("sheet:{}", bodies[0]);
    let snap = snapshot(&mut seat);
    assert!(!snap.scene.active_items_in_order().iter().any(|(_, item)|
        snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == other));
    // Any seat with GM authority may discard a draft it did not start.
    let mut gm = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    assert_eq!(click(&mut gm, &other, "vtt.sheet-create"), IntentResult::Accepted);
    assert_eq!(click(&mut gm, &other, "vtt.draft-human"), IntentResult::Accepted);
    let mut second = VttEndpoint::open(host.world(), ParticipantHandle(player.0), Seat::Dm).unwrap();
    assert_eq!(click(&mut second, &other, "vtt.draft-discard"), IntentResult::Accepted);
    assert!(!state(&host).drafts.contains_key(&token(bodies[0]).unwrap()));
    // Save and reopen: replay from the checkpoint reaches the same sections.
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    for record in host.drain_records() {
        replica.apply(record).unwrap();
    }
    let saved = host.world().save().unwrap();
    assert_eq!(replica.snapshot().unwrap().save, saved);
    let reopened = State::read(&saved.sections).unwrap().unwrap();
    assert_eq!(reopened.drafts, state(&host).drafts);
}

#[test]
fn a_5e_founding_admits_no_draft() {
    let table::Table { world, dm, bodies, .. } = table::table_with_sites(1);
    founded(&world, "pf2e-srd");
    let host = FamilyHost::new(SessionId([82; 16]), world).unwrap();
    let mut gm = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let card = format!("sheet:{}", bodies[0]);
    assert_eq!(click(&mut gm, &card, "vtt.sheet-create"), IntentResult::Accepted);
    let mut sections = host.world().save().unwrap().sections;
    assert!(State::read(&sections).is_ok());
    let draft = content_fixture::draft();
    let content = serde_json::json!({"content": {"id": "x", "name": "x", "maps": draft.maps,
        "storylets": draft.world.storylets, "rewards": [], "starting_map": draft.starting_map,
        "final_storylet": draft.final_storylet, "table": TableMetadata::default()}, "origin": null});
    sections.insert("isocosm-vtt:campaign".into(), serde_json::to_vec(&content).unwrap());
    assert!(State::read(&sections).is_err(), "drafts under a 5e founding are refused");
    assert!(sections.contains_key(SECTION));
}
