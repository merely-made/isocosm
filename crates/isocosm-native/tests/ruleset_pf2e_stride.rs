#![cfg(feature = "endpoint")]
//! Stride (case 1042): an admitted, placed character spends one action and
//! its native cell batch moves it in the same transaction, ruled by a GM seat
//! with no scale, measured against Speed with one.
use chirograph::{IntentInvocation, IntentResult};
use cosmograph::Execution;
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_native::{
    AUTHORING, SHEETS, VttEndpoint,
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
    assert_eq!(click(&mut gm, SHEETS, "pf2e-table", "vtt.pf2e-start-encounter", &[]), IntentResult::Accepted);
    // Ruled: with no scale, the GM seat rules its own character's Stride.
    let player_card = format!("sheet:{}", characters[0]);
    if state(&host).encounter.unwrap().current().unwrap().token == characters[0] as u32 {
        assert_eq!(click(&mut seat, SHEETS, &player_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    }
    let gm_card = format!("sheet:{}", characters[1]);
    let before = at(&host, characters[1]);
    stride(&mut gm, &gm_card, &["1-cells", "4-cells"]);
    assert_ne!(at(&host, characters[1]), before, "the native position moved");
    assert_eq!(state(&host).encounter.unwrap().current().unwrap().actions, 2);
    assert!(state(&host).log.last().unwrap().note.contains("GM ruled the move within Speed"));
    assert_eq!(click(&mut gm, SHEETS, &gm_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    // Measured: a scale set, the player Strides on their own turn.
    assert_eq!(click(&mut gm, AUTHORING, "ruleset-scale", "vtt.ruleset-scale", br#"{"cells_per_square":4}"#),
        IntentResult::Accepted);
    if state(&host).encounter.unwrap().current().unwrap().token != characters[0] as u32 {
        assert_eq!(click(&mut gm, SHEETS, &gm_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    }
    let before = at(&host, characters[0]);
    stride(&mut seat, &player_card, &["15-feet", "5-feet"]);
    let after = at(&host, characters[0]);
    assert_ne!(after, before);
    let span = (0..3).map(|k| after[k].abs_diff(before[k])).max().unwrap();
    assert!(span.div_ceil(4) * 5 <= 15, "within the armoured dwarf's 15-foot Speed");
    assert!(state(&host).log.last().unwrap().note.contains("measured at 4 cells per square"));
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    for record in host.drain_records() {
        replica.apply(record).unwrap();
    }
    assert_eq!(replica.snapshot().unwrap().save, host.world().save().unwrap());
}
