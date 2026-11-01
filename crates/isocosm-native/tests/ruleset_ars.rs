#![cfg(feature = "endpoint")]
//! Ars Magica at the table (cases 1063, 1068): a magus drafted through the
//! sheet's buttons, a season of study from a covenant summa across consented
//! native downtime, a formulaic cast through F11 under the native sky, and a
//! Wound that wounds the native body; replay reaches the same world.
use chirograph::{IntentInvocation, IntentResult};
use cosmograph::Execution;
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_native::{
    SHEETS, TABLE, VttEndpoint,
    founding::{Content, PackSet, TableMetadata},
    ruleset::{State, token},
};
use isocosm_system::ars::sample::BONISAGUS_MAGUS;
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};
#[path = "native_founding/fixture.rs"]
mod content_fixture;
#[path = "support/table.rs"]
mod table;
#[path = "support/admitted.rs"]
mod admitted;

fn founded(world: &World) {
    let draft = content_fixture::draft();
    let content = Content {
        id: "ars-bench".into(), name: "Ars Bench".into(), maps: draft.maps, storylets: draft.world.storylets,
        rewards: vec![], starting_map: draft.starting_map, final_storylet: draft.final_storylet,
        table: TableMetadata::default(), edition: PackSet::bundled("ars5").unwrap(),
    };
    content.validate().unwrap();
    let bytes = serde_json::to_vec(&serde_json::json!({"content": content, "origin": null})).unwrap();
    world.put_section("isocosm-native:campaign", bytes).unwrap();
}

/// Invoke `intent` on the card whose id passes `card`, in `projection`.
fn click(endpoint: &mut VttEndpoint, projection: &str, card: impl Fn(&str) -> bool, intent: &str, payload: &[u8]) -> IntentResult {
    let offer = endpoint.describe().projections.into_iter().find(|o| o.request.session.0 == projection).unwrap();
    let snap = endpoint.snapshot(offer.request).unwrap();
    let target = snap.scene.active_items_in_order().into_iter()
        .find(|(_, item)| card(&snap.scene.tables.sources[item.source.0 as usize].as_ref().unwrap().id))
        .unwrap_or_else(|| panic!("no card for {intent}")).0;
    endpoint.invoke(IntentInvocation { session: snap.session, target, observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision, intent: intent.into(), payload: payload.to_vec() }).unwrap()
}

fn state(host: &FamilyHost) -> State {
    host.read(|_, sections| State::read(sections).unwrap().unwrap()).unwrap()
}

#[test]
fn a_magus_is_drafted_studies_a_season_casts_and_is_wounded() {
    // Admitted characters live through a season; the bench fixture's
    // founded population ages out within its first native day.
    let admitted::Admitted { world, dm, characters, .. } = admitted::admitted_table();
    founded(&world);
    let mut host = FamilyHost::new(SessionId([84; 16]), world).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    host.drain_records();
    let mut gm = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let body = characters[1];
    let card = format!("sheet:{body}");
    let on = |id: &str| id == card;
    let ok = IntentResult::Accepted;
    assert_eq!(click(&mut gm, SHEETS, on, "vtt.sheet-create", &[]), ok);
    for (_, value) in BONISAGUS_MAGUS {
        assert_eq!(click(&mut gm, SHEETS, on, &format!("vtt.draft-{value}"), &[]), ok, "{value}");
    }
    assert_eq!(click(&mut gm, SHEETS, on, "vtt.draft-finish", &[]), ok);
    let id = token(body).unwrap();
    let sheet = state(&host).sheets[&id].clone();
    assert_eq!(sheet.system, "ars5");
    assert_eq!(sheet.int("ars:muto"), Some(10));
    let corpus_xp = sheet.int("ars:xp:corpus").unwrap();
    // The birth is a dated native history line about the character, before
    // the founding; the native body itself stays born at tick 0 (case 1069).
    let (time, subject, year) = host.read(|session, _| {
        let event = &session.sim.state().events[&format!("line:ars-birth-{body}")];
        (event.authored.as_ref().unwrap().time, event.subject,
            cosmograph::rules::year_ticks(session.sim.genesis().rules.tick_microseconds()))
    }).unwrap();
    assert_eq!(subject, body);
    assert_eq!(time, -25 * i64::try_from(year).unwrap(), "born 25 native years before tick 0");

    let summa = br#"{"subject":"corpus","level":12,"quality":10}"#;
    assert_eq!(click(&mut gm, SHEETS, |c| c == "ars-covenant", "vtt.ars-add-summa", summa), ok);
    assert_eq!(click(&mut gm, SHEETS, on, "vtt.ars-study-0", &[]), ok);
    assert_ne!(click(&mut gm, SHEETS, on, "vtt.ars-complete", &[]), ok, "the season has not passed");
    assert_eq!(click(&mut gm, TABLE, |c| c == "roster", "vtt.roster-empty", &[]), ok);
    assert_eq!(click(&mut gm, SHEETS, on, "vtt.ars-propose-season", &[]), ok);
    assert_eq!(click(&mut gm, TABLE, |c| c.starts_with("proposal:"), "vtt.advance", &[]), ok);
    assert_eq!(click(&mut gm, SHEETS, on, "vtt.ars-complete", &[]), ok);
    let studied = state(&host);
    assert_eq!(studied.sheets[&id].int("ars:xp:corpus"), Some(corpus_xp + 10));
    assert!(studied.log.last().unwrap().note.contains("completes a season of study"));

    assert_eq!(click(&mut gm, SHEETS, on, "vtt.ars-cast-eyes-of-the-cat", &[]), ok);
    assert!(state(&host).log.last().unwrap().note.contains("casting total"));

    let lost = |host: &FamilyHost| host.read(|session, _| session.sim.state().population.get(body)
        .map_or(0, |b| b.parts.values().map(|p| p.lost.len()).sum::<usize>())).unwrap();
    let before = lost(&host);
    assert_eq!(click(&mut gm, SHEETS, on, "vtt.sheet-damage-15", &[]), ok);
    let wounded = state(&host);
    assert_eq!(wounded.sheets[&id].int("ars:wounds:heavy"), Some(1), "15 against Soak 1 is a Heavy Wound");
    assert_eq!(lost(&host), before + 3, "a Heavy Wound takes three native cells");

    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    for record in host.drain_records() {
        replica.apply(record).unwrap();
    }
    assert_eq!(replica.snapshot().unwrap().save, host.world().save().unwrap());
}
