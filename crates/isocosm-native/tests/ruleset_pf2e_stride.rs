#![cfg(feature = "endpoint")]
//! Stride (case 1042): an admitted, placed character spends one action and
//! its native cell batch moves it in the same transaction, ruled by a GM seat
//! with no scale, measured against Speed with one.
use chirograph::{IntentInvocation, IntentResult, PortableCardV1, ResourceRequest};
use cosmograph::Execution;
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use isocosm_core::encounter_board::{BADGE, VALUE, EncounterBoard};
use isocosm_native::{
    AUTHORING, SHEETS, TABLE, VttEndpoint,
    founding::{Content, TableMetadata},
    native::GameBindings,
    ruleset::State,
};
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};
#[path = "support/admitted.rs"]
mod admitted;
#[path = "native_founding/fixture.rs"]
mod content_fixture;
#[path = "support/table.rs"]
mod table;

const DWARF_FIGHTER: &[&str] = &[
    "dwarf", "listed", "str", "rock-dwarf", "warrior", "str", "con", "fighter", "athletics", "str",
    "dex", "con", "wis", "int", "acrobatics", "medicine", "survival", "crafting", "Gnomish",
    "dwarven-lore", "society", "vicious-swing", "longsword", "steel-shield", "scale-mail", "done",
];

fn founded(world: &World) {
    let draft = content_fixture::draft();
    let content = Content { id: "pf2e-stride".into(), name: "Stride Bench".into(), maps: draft.maps,
        storylets: draft.world.storylets, rewards: vec![], starting_map: draft.starting_map,
        final_storylet: draft.final_storylet, table: TableMetadata::default(), edition: "pf2e-srd".into() };
    let bytes = serde_json::to_vec(&serde_json::json!({"content": content, "origin": null})).unwrap();
    world.put_section("isocosm-native:campaign", bytes).unwrap();
}

fn click(endpoint: &mut VttEndpoint, projection: &str, card: &str, intent: &str, payload: &[u8]) -> IntentResult {
    let offer = endpoint.describe().projections.into_iter().find(|o| o.request.session.0 == projection).unwrap();
    let snap = endpoint.snapshot(offer.request).unwrap();
    let target = snap.scene.active_items_in_order().into_iter()
        .find(|(_, item)| snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == card)
        .unwrap().0;
    endpoint.invoke(IntentInvocation { session: snap.session, target, observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision, intent: intent.into(), payload: payload.to_vec() }).unwrap()
}

/// Read precisely the card and advertised intents available to this window.
fn boards(endpoint: &mut VttEndpoint) -> Vec<(String, EncounterBoard, Vec<String>)> {
    let request = endpoint.describe().projections.into_iter().find(|p| p.request.session.0 == TABLE).unwrap().request;
    let snapshot = endpoint.snapshot(request).unwrap();
    snapshot.scene.active_items_in_order().into_iter().filter_map(|(instance, item)| {
        let source = snapshot.scene.tables.sources[item.source.0 as usize].as_ref().unwrap();
        let offer = &snapshot.presentation.offers_for(instance).unwrap()[0];
        let bytes = endpoint.resource(ResourceRequest { session: snapshot.session.clone(), resource: offer.resource }).unwrap().bytes;
        let card: PortableCardV1 = serde_json::from_slice(&bytes).unwrap();
        if !card.badges.iter().any(|b| b == BADGE) { return None; }
        let data = card.values.iter().find(|field| field.label == VALUE).unwrap();
        let actions = offer.semantics.actions.iter().map(|a| a.intent.0.clone()).collect();
        Some((source.id.clone(), serde_json::from_str(&data.value).unwrap(), actions))
    }).collect()
}

/// Click the first Stride the card offers among the compass steps.
fn stride(endpoint: &mut VttEndpoint, card: &str, steps: &[&str]) -> String {
    for step in steps {
        for way in ["east", "west", "south", "north"] {
            let intent = format!("vtt.pf2e-stride-{step}-{way}");
            if click(endpoint, SHEETS, card, &intent, &[]) == IntentResult::Accepted {
                return intent;
            }
        }
    }
    panic!("no Stride offered on {card}");
}

fn state(host: &FamilyHost) -> State {
    host.read(|_, sections| State::read(sections).unwrap().unwrap()).unwrap()
}

fn at(host: &FamilyHost, body: u64) -> [i64; 3] {
    host.read(|session, sections| {
        let game = GameBindings::decode(&sections["vtt"]).unwrap();
        game.positions().values().find(|p| p.entity == body).unwrap().current(session).unwrap()
    }).unwrap()
}

#[test]
fn stride_carries_the_native_move_ruled_then_measured() {
    let admitted::Admitted { world, dm, player, characters } = admitted::admitted_table();
    founded(&world);
    let mut host = FamilyHost::new(SessionId([85; 16]), world).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    host.drain_records();
    let mut gm = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let mut seat = VttEndpoint::open(host.world(), player, Seat::Player).unwrap();
    for (endpoint, body) in [(&mut seat, characters[0]), (&mut gm, characters[1])] {
        let card = format!("sheet:{body}");
        assert_eq!(click(endpoint, SHEETS, &card, "vtt.sheet-create", &[]), IntentResult::Accepted, "{card}");
        for value in DWARF_FIGHTER {
            assert_eq!(click(endpoint, SHEETS, &card, &format!("vtt.draft-{value}"), &[]), IntentResult::Accepted);
        }
        assert_eq!(click(endpoint, SHEETS, &card, "vtt.draft-finish", &[]), IntentResult::Accepted);
    }
    let (site_card, _, start) = boards(&mut gm).pop().unwrap();
    assert!(start.iter().any(|id| id == "vtt.pf2e-start-encounter"));
    assert_ne!(click(&mut seat, TABLE, &site_card, "vtt.pf2e-start-encounter", &[]), IntentResult::Accepted);
    assert_eq!(click(&mut gm, TABLE, &site_card, "vtt.pf2e-start-encounter", &[]), IntentResult::Accepted);
    // Ruled: with no scale, the GM seat rules its own character's Stride.
    if state(&host).encounter.unwrap().current().unwrap().token == characters[0] as u32 {
        assert_eq!(click(&mut seat, TABLE, &site_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    }
    let gm_card = format!("sheet:{}", characters[1]);
    let before = at(&host, characters[1]);
    stride(&mut gm, &gm_card, &["1-cells", "4-cells"]);
    assert_ne!(at(&host, characters[1]), before, "the native position moved");
    assert_eq!(state(&host).encounter.unwrap().current().unwrap().actions, 2);
    assert!(state(&host).log.last().unwrap().note.contains("GM ruled the move within Speed"));
    assert_eq!(click(&mut gm, TABLE, &site_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    // Measured: a scale set, the player Strides on their own turn.
    assert_eq!(click(&mut gm, AUTHORING, "ruleset-scale", "vtt.ruleset-scale", br#"{"cells_per_square":4}"#),
        IntentResult::Accepted);
    if state(&host).encounter.unwrap().current().unwrap().token != characters[0] as u32 {
        assert_eq!(click(&mut gm, TABLE, &site_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    }
    let before = at(&host, characters[0]);
    let (card, board, offers) = boards(&mut seat).pop().unwrap();
    assert_eq!(board.cells_per_square, 4);
    assert!(board.tokens.iter().any(|t| t.body == characters[0] && t.at == before && t.active));
    // The draft purchased scale mail but never readied it. This dwarf has
    // the ancestry's 20-foot Speed; the legacy sheet offered a full Stride.
    let distance = |at| isocosm_system::pf2e::combat::feet(before, at, board.cells_per_square);
    let landings = board.cells.iter().filter(|cell| cell.stride.is_some());
    assert!(landings.clone().all(|cell| distance(cell.at) <= 20));
    let destination = landings.max_by_key(|cell| distance(cell.at)).unwrap();
    assert_eq!(distance(destination.at), 20);
    let stride = destination.stride.as_deref().unwrap();
    assert!(offers.iter().any(|offered| offered == stride));
    assert!(offers.iter().any(|id| id == "vtt.pf2e-end-turn"));
    // Even a GM who sees this character does not play it.
    let (_, gm_board, gm_offers) = boards(&mut gm).pop().unwrap();
    assert!(gm_board.cells.iter().all(|cell| cell.stride.is_none()) && !gm_offers.iter().any(|id| id == stride));
    assert_ne!(click(&mut gm, TABLE, &card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    assert_ne!(click(&mut gm, TABLE, &card, stride, &[]), IntentResult::Accepted);
    assert_eq!(at(&host, characters[0]), before);
    assert_eq!(click(&mut seat, TABLE, &card, stride, &[]), IntentResult::Accepted);
    assert_eq!(at(&host, characters[0]), destination.at);
    assert_eq!(state(&host).encounter.unwrap().current().unwrap().actions, 2);
    assert!(boards(&mut seat).pop().unwrap().1.tokens.iter().any(|t| t.body == characters[0] && t.at == destination.at));
    let after = at(&host, characters[0]);
    assert_ne!(after, before);
    host.read(|session, sections| {
        let game = GameBindings::decode(&sections["vtt"]).unwrap();
        let p = &game.positions().values().find(|p| p.entity == characters[0]).unwrap().placement;
        let v = session.sim.volume(p.site, p.window.min, p.window.max, p.window.depth).unwrap();
        assert_eq!(v.surface(after[0], after[2]), Some(after[1] - 1), "the offered Stride landed on the floor");
        assert!(before[1] > v.surface(before[0], before[2]).unwrap() + 1, "the original elevated binding remained readable");
    }).unwrap();
    assert!(state(&host).log.last().unwrap().note.contains("measured at 4 cells per square"));
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    for record in host.drain_records() {
        replica.apply(record).unwrap();
    }
    assert_eq!(replica.snapshot().unwrap().save, host.world().save().unwrap());
}


#[test]
fn board_uses_the_selected_native_eye_for_tokens_and_terrain() {
    let admitted::Admitted { world, dm, player, characters } = admitted::sighted_table([true, false], 0);
    founded(&world);
    let mut gm = VttEndpoint::open(&world, dm, Seat::Dm).unwrap();
    let mut seat = VttEndpoint::open(&world, player, Seat::Player).unwrap();
    let (_, truth, _) = boards(&mut gm).pop().unwrap();
    assert_eq!(truth.tokens.len(), 2);
    assert!(truth.cells.len() > 2);
    assert_ne!(truth.tokens[0].at, truth.tokens[1].at, "the hidden token stands in another column");
    assert!(boards(&mut seat).is_empty(), "the original, unmarked observer has no admitted eye");
    assert_eq!(click(&mut seat, TABLE, &format!("body:{}", characters[0]), "vtt.view-as", &[]), IntentResult::Accepted);
    let (_, cut, offers) = boards(&mut seat).pop().unwrap();
    assert_eq!(cut.tokens.len(), 1);
    assert_eq!(cut.tokens[0].body, characters[0]);
    assert_eq!(cut.cells.len(), 1);
    assert_eq!(cut.cells[0].at, cut.tokens[0].at);
    assert!(cut.cells[0].stride.is_none() && offers.is_empty());
}
