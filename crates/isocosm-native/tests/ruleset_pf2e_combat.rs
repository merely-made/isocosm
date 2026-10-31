#![cfg(feature = "endpoint")]
//! Ordinary combat over admitted, measured characters (cases 1047, 1050):
//! ranged Strikes spend ammunition, a GM-ruled off-guard target takes Sneak
//! Attack, a hit on a raised shield waits for Shield Block, and Battle
//! Medicine heals once a day.
use chirograph::{IntentInvocation, IntentResult};
use cosmograph::Execution;
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_native::{
    AUTHORING, SHEETS, VttEndpoint,
    founding::{Content, TableMetadata},
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

const HUMAN_ROGUE: &[&str] = &[
    "human", "dex", "int", "skilled-human", "stealth", "street-urchin", "dex", "con", "rogue", "thief",
    "athletics", "acrobatics", "dex", "dex", "con", "wis", "cha", "arcana", "crafting", "diplomacy",
    "medicine", "nature", "occultism", "religion", "society", "Draconic", "Elven", "natural-ambition",
    "plant-evidence", "battle-medicine", "tumble-behind", "rapier", "shortbow", "arrows", "leather-armor",
    "healers-toolkit", "done",
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

fn state(host: &FamilyHost) -> State {
    host.read(|_, sections| State::read(sections).unwrap().unwrap()).unwrap()
}

fn ready(host: &FamilyHost, endpoint: &mut VttEndpoint, body: u64, names: &[&str]) {
    let items: Vec<(String, String)> = host.read(|_, sections| {
        let state = State::read(sections).unwrap().unwrap();
        state.inventories[&isocosm_native::ruleset::token(body).unwrap()].items.iter()
            .map(|(id, item)| (id.0.clone(), item.name.clone())).collect()
    }).unwrap();
    for (id, name) in items {
        if names.contains(&name.as_str()) {
            assert_eq!(click(endpoint, SHEETS, &format!("sheet:{body}"), &format!("vtt.pf2e-equip-{id}"), &[]),
                IntentResult::Accepted, "{name}");
        }
    }
}

fn current(host: &FamilyHost) -> u64 {
    u64::from(state(host).encounter.unwrap().current().unwrap().token)
}

fn arrows(host: &FamilyHost, body: u64) -> u32 {
    let state = state(host);
    state.inventories[&isocosm_native::ruleset::token(body).unwrap()].items.values()
        .find(|i| i.template == "pf2e:arrows").map_or(0, |i| i.quantity)
}

#[test]
fn ranged_strikes_sneak_attack_shield_block_and_battle_medicine() {
    let admitted::Admitted { world, dm, player, characters } = admitted::admitted_table();
    founded(&world);
    let mut host = FamilyHost::new(SessionId([86; 16]), world).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    host.drain_records();
    let (rogue, fighter) = (characters[0], characters[1]);
    let mut gm = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let mut seat = VttEndpoint::open(host.world(), player, Seat::Player).unwrap();
    for (endpoint, body, script) in [(&mut seat, rogue, HUMAN_ROGUE), (&mut gm, fighter, DWARF_FIGHTER)] {
        let card = format!("sheet:{body}");
        assert_eq!(click(endpoint, SHEETS, &card, "vtt.sheet-create", &[]), IntentResult::Accepted);
        for value in script {
            assert_eq!(click(endpoint, SHEETS, &card, &format!("vtt.draft-{value}"), &[]), IntentResult::Accepted, "{value}");
        }
        assert_eq!(click(endpoint, SHEETS, &card, "vtt.draft-finish", &[]), IntentResult::Accepted);
    }
    ready(&host, &mut seat, rogue, &["Shortbow", "Leather Armor"]);
    ready(&host, &mut gm, fighter, &["Longsword", "Steel Shield", "Scale Mail"]);
    assert_eq!(click(&mut gm, AUTHORING, "ruleset-scale", "vtt.ruleset-scale", br#"{"cells_per_square":4}"#),
        IntentResult::Accepted);
    let (rogue_card, fighter_card) = (format!("sheet:{rogue}"), format!("sheet:{fighter}"));
    assert_eq!(click(&mut gm, SHEETS, &fighter_card, "vtt.pf2e-off-guard-on", &[]), IntentResult::Accepted);
    assert_eq!(click(&mut gm, SHEETS, "pf2e-table", "vtt.pf2e-start-encounter", &[]), IntentResult::Accepted);
    let before = arrows(&host, rogue);
    let mut strikes = 0;
    let mut sneak = false;
    let mut blocked = false;
    for _ in 0..12 {
        if current(&host) == fighter {
            if !blocked {
                let _ = click(&mut gm, SHEETS, &fighter_card, "vtt.pf2e-raise-shield", &[]);
            }
            assert_eq!(click(&mut gm, SHEETS, &fighter_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
            continue;
        }
        while state(&host).encounter.unwrap().current().unwrap().actions > 0 && state(&host).pending.is_none() {
            assert_eq!(click(&mut seat, SHEETS, &rogue_card, &format!("vtt.pf2e-strike-{fighter}"), &[]),
                IntentResult::Accepted);
            strikes += 1;
            let last = state(&host).log.last().unwrap().clone();
            sneak |= last.rolls.iter().any(|r| r.expr.contains("Sneak Attack"));
            assert!(last.rolls[0].expr.contains("off-guard"), "{:?}", last.rolls[0]);
        }
        if state(&host).pending.is_some() {
            assert_eq!(click(&mut gm, SHEETS, &fighter_card, "vtt.pf2e-shield-block", &[]), IntentResult::Accepted);
            assert!(state(&host).log.last().unwrap().note.contains("Shield Block"));
            blocked = true;
        }
        if sneak && blocked {
            break;
        }
        if current(&host) == rogue {
            assert_eq!(click(&mut seat, SHEETS, &rogue_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
        }
    }
    assert!(sneak, "an off-guard target took Sneak Attack within {strikes} Strikes");
    assert!(blocked, "a hit on the raised shield waited for Shield Block");
    assert_eq!(arrows(&host, rogue), before - strikes, "each ranged Strike spent an arrow");
    // Battle Medicine once a day: the second attempt on the same patient is refused.
    if current(&host) != rogue {
        assert_eq!(click(&mut gm, SHEETS, &fighter_card, "vtt.pf2e-end-turn", &[]), IntentResult::Accepted);
    }
    if current(&host) == rogue {
        let heal = format!("vtt.pf2e-battle-medicine-{rogue}");
        assert_eq!(click(&mut seat, SHEETS, &rogue_card, &heal, &[]), IntentResult::Accepted);
        assert_ne!(click(&mut seat, SHEETS, &rogue_card, &heal, &[]), IntentResult::Accepted);
    }
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    for record in host.drain_records() {
        replica.apply(record).unwrap();
    }
    assert_eq!(replica.snapshot().unwrap().save, host.world().save().unwrap());
}
