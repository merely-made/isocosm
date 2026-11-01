//! Write a family ledger founded under Ars Magica on its pinned pack, for the
//! desktop trial: two admitted characters (who live through a season), an
//! empty saved roster, and a covenant summa on Corpus. `--ready` also drafts
//! and finishes the GM's character as the sample Bonisagus magus.
//! `--inspect` reopens a saved ledger and prints what it replayed.
#[path = "../tests/native_founding/fixture.rs"]
mod content_fixture;
#[path = "../tests/support/table.rs"]
mod table;
#[path = "../tests/support/admitted.rs"]
mod admitted;
use chirograph::{IntentInvocation, IntentResult};
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_native::{
    SHEETS, TABLE, VttEndpoint,
    founding::{Content, PackSet, TableMetadata},
    native::GameBindings,
};
use isocosm_system::ars::sample::BONISAGUS_MAGUS;
use isonetry::family::{FamilyHost, FamilyLedgerLimits, FamilyLimits, SessionId};

fn click(endpoint: &mut VttEndpoint, projection: &str, id: &str, intent: &str, payload: &[u8]) -> Result<(), String> {
    let offer = endpoint.describe().projections.into_iter()
        .find(|o| o.request.session.0 == projection).ok_or("missing projection")?;
    let snap = endpoint.snapshot(offer.request)?;
    let target = snap.scene.active_items_in_order().into_iter()
        .find(|(_, item)| snap.scene.tables.sources[item.source.0 as usize].as_ref().is_some_and(|s| s.id == id))
        .ok_or("missing card")?.0;
    match endpoint.invoke(IntentInvocation { session: snap.session, target, observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision, intent: intent.into(), payload: payload.to_vec() })? {
        IntentResult::Accepted => Ok(()),
        other => Err(format!("fixture action {intent} refused: {other:?}")),
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("supply a new .family ledger path")?;
    if path == "--inspect" {
        return inspect(&args.next().ok_or("supply the saved .family ledger path")?);
    }
    let ready = args.next().as_deref() == Some("--ready");
    let a = admitted::admitted_table();
    let (session, mut sections) = a.world.into_parts()?;
    let draft = content_fixture::draft();
    let content = Content {
        id: "ars-bench".into(), name: "Ars Bench".into(), maps: draft.maps, storylets: draft.world.storylets,
        rewards: vec![], starting_map: draft.starting_map, final_storylet: draft.final_storylet,
        table: TableMetadata::default(), edition: PackSet::bundled("ars5")?,
    };
    content.validate().map_err(|e| e.to_string())?;
    sections.entry("vtt".into()).or_insert(GameBindings::decode(&[])?.encode()?);
    sections.insert("isocosm-native:campaign".into(),
        serde_json::to_vec(&serde_json::json!({"content": content, "origin": null})).map_err(|e| e.to_string())?);
    sections.insert("isocosm-native:table-access".into(),
        serde_json::to_vec(&serde_json::json!({"host_participant": a.dm.0, "peers": []})).map_err(|e| e.to_string())?);
    let host = FamilyHost::create_file(&path, SessionId([85; 16]), World::with_sections(session, sections),
        FamilyLimits::default(), FamilyLedgerLimits::default())?;
    let mut gm = VttEndpoint::open(host.world(), a.dm, Seat::Dm)?;
    click(&mut gm, TABLE, "roster", "vtt.roster-empty", &[])?;
    click(&mut gm, SHEETS, "ars-covenant", "vtt.ars-add-summa", br#"{"subject":"corpus","level":12,"quality":10}"#)?;
    let body = a.characters[1];
    if ready {
        let card = format!("sheet:{body}");
        click(&mut gm, SHEETS, &card, "vtt.sheet-create", &[])?;
        for (_, value) in BONISAGUS_MAGUS {
            click(&mut gm, SHEETS, &card, &format!("vtt.draft-{value}"), &[])?;
        }
        click(&mut gm, SHEETS, &card, "vtt.draft-finish", &[])?;
    }
    println!("Ars Bench ledger {path}; gm body={body}; sequence={}", host.sequence());
    Ok(())
}

fn inspect(path: &str) -> Result<(), String> {
    let host = FamilyHost::resume_file(path, cosmograph::Execution::Grouped, FamilyLimits::default(),
        FamilyLedgerLimits::default())?;
    let report = host.read(|session, sections| -> Result<_, String> {
        let state = isocosm_native::ruleset::State::read(sections)?.ok_or("no saved edition state")?;
        Ok(serde_json::json!({
            "edition": isocosm_native::ruleset::edition(sections)?,
            "tick": session.sim.state().tick,
            "sheets": state.sheets.iter().map(|(id, s)| serde_json::json!({"body": id.0, "system": s.system,
                "corpus": s.int("ars:corpus"), "corpus_xp": s.int("ars:xp:corpus")})).collect::<Vec<_>>(),
            "log": state.log.iter().rev().take(6).map(|e| e.note.clone()).collect::<Vec<_>>(),
        }))
    })??;
    println!("{}", serde_json::to_string_pretty(&serde_json::json!({"sequence": host.sequence(), "reopened": report}))
        .map_err(|e| e.to_string())?);
    Ok(())
}
