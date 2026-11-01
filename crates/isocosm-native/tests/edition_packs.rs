#![cfg(feature = "endpoint")]
//! A campaign founded on a pinned pack set (cases 1064, 1067) plays its
//! edition as an unpinned one does, survives the save, and a pin the held
//! pack no longer matches is refused rather than read.
use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_native::{
    SHEETS, VttEndpoint,
    founding::{Content, PackSet, TableMetadata},
    ruleset::{self, State},
};
use isonetry::family::{FamilyHost, SessionId};
#[path = "native_founding/fixture.rs"]
mod content_fixture;
#[path = "support/table.rs"]
mod table;

fn content(edition: PackSet) -> Content {
    let draft = content_fixture::draft();
    Content {
        id: "pinned".into(),
        name: "Pinned".into(),
        maps: draft.maps,
        storylets: draft.world.storylets,
        rewards: vec![],
        starting_map: draft.starting_map,
        final_storylet: draft.final_storylet,
        table: TableMetadata::default(),
        edition,
    }
}

fn click(endpoint: &mut VttEndpoint, card: &str, intent: &str) -> IntentResult {
    let offer = endpoint.describe().projections.into_iter()
        .find(|o| o.request.session.0 == SHEETS).unwrap();
    let snap: ProjectionSnapshot = endpoint.snapshot(offer.request).unwrap();
    let target = snap.scene.active_items_in_order().into_iter()
        .find(|(_, item)| snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id == card)
        .unwrap().0;
    endpoint.invoke(IntentInvocation { session: snap.session, target,
        observed_epoch: snap.scene.epoch, observed_revision: snap.scene.revision,
        intent: intent.into(), payload: vec![] }).unwrap()
}

#[test]
fn a_pinned_founding_plays_its_edition_and_a_stale_pin_refuses() {
    let pinned = content(PackSet::bundled("pf2e-srd").unwrap());
    pinned.validate().unwrap();
    let saved = serde_json::to_vec(&serde_json::json!({"content": pinned, "origin": null})).unwrap();
    let text = String::from_utf8(saved.clone()).unwrap();
    assert!(text.contains("\"hash\""), "the founding record carries the pin");

    let table::Table { world, dm, bodies, .. } = table::table_with_sites(1);
    world.put_section("isocosm-native:campaign", saved).unwrap();
    let host = FamilyHost::new(SessionId([83; 16]), world).unwrap();
    let sections = host.world().save().unwrap().sections;
    assert_eq!(ruleset::edition(&sections).unwrap(), "pf2e-srd");

    let mut gm = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let card = format!("sheet:{}", bodies[0]);
    assert_eq!(click(&mut gm, &card, "vtt.sheet-create"), IntentResult::Accepted);
    let state = host.read(|_, sections| State::read(sections).unwrap().unwrap()).unwrap();
    assert_eq!(state.drafts.len(), 1, "a Pathfinder draft starts under the pinned set");

    let mut stale = PackSet::bundled("pf2e-srd").unwrap();
    stale.packs[0].hash = "0".repeat(64);
    let refused = content(stale).validate().unwrap_err().to_string();
    assert!(refused.contains("pinned at"), "{refused}");
}
