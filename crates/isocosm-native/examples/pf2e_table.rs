//! Write a fresh Bench family ledger founded under Pathfinder 2e, for the
//! desktop creation trial; `--inspect` reopens a saved one and prints what the
//! family host replayed.
#[path = "../tests/native_founding/fixture.rs"]
mod content_fixture;
#[path = "../tests/support/table.rs"]
mod table;
use chirograph::{IntentInvocation, IntentResult};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use cosmograph_host::world::World;
use cosmograph_overlay::Seat;
use isocosm_native::{
    SHEETS, TABLE, VttEndpoint,
    founding::{Content, TableMetadata},
    native::GameBindings,
};
use isonetry::family::{FamilyHost, FamilyLedgerLimits, FamilyLimits, SessionId};

fn click(
    endpoint: &mut VttEndpoint,
    projection: &str,
    id: &str,
    intent: &str,
) -> Result<(), String> {
    let offer = endpoint
        .describe()
        .projections
        .into_iter()
        .find(|o| o.request.session.0 == projection)
        .ok_or("missing projection")?;
    let snap = endpoint.snapshot(offer.request)?;
    let target = snap
        .scene
        .active_items_in_order()
        .into_iter()
        .find(|(_, item)| {
            snap.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .is_some_and(|source| source.id == id)
        })
        .ok_or("missing card")?
        .0;
    match endpoint.invoke(IntentInvocation {
        session: snap.session,
        target,
        observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision,
        intent: intent.into(),
        payload: vec![],
    })? {
        IntentResult::Accepted => Ok(()),
        other => Err(format!("fixture action refused: {other:?}")),
    }
}

const DWARF_FIGHTER: &[&str] = &[
    "dwarf", "listed", "str", "rock-dwarf", "warrior", "str", "con", "fighter", "athletics", "str",
    "dex", "con", "wis", "int", "acrobatics", "medicine", "survival", "crafting", "Gnomish",
    "dwarven-lore", "society", "vicious-swing", "longsword", "steel-shield", "scale-mail", "done",
];

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("supply a new .family ledger path")?;
    if path == "--inspect" {
        return inspect(&args.next().ok_or("supply the saved .family ledger path")?);
    }
    let table::Table {
        world, dm, bodies, ..
    } = table::table_with_sites(1);
    let (session, mut sections) = world.into_parts()?;
    let draft = content_fixture::draft();
    let content = Content {
        id: "pathfinder-bench".into(),
        name: "Pathfinder Bench".into(),
        maps: draft.maps,
        storylets: draft.world.storylets,
        rewards: vec![],
        starting_map: draft.starting_map,
        final_storylet: draft.final_storylet,
        table: TableMetadata::default(),
        edition: "pf2e-srd".into(),
    };
    content.validate().map_err(|e| e.to_string())?;
    sections.insert("vtt".into(), GameBindings::decode(&[])?.encode()?);
    sections.insert(
        "isocosm-native:campaign".into(),
        serde_json::to_vec(&serde_json::json!({"content":content,"origin":null}))
            .map_err(|e| e.to_string())?,
    );
    sections.insert(
        "isocosm-native:table-access".into(),
        serde_json::to_vec(&serde_json::json!({"host_participant":dm.0,"peers":[]}))
            .map_err(|e| e.to_string())?,
    );
    let host = FamilyHost::create_file(
        &path,
        SessionId([83; 16]),
        World::with_sections(session, sections),
        FamilyLimits::default(),
        FamilyLedgerLimits::default(),
    )?;
    let mut endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm)?;
    click(&mut endpoint, TABLE, "roster", "vtt.roster-empty")?;
    // `--ready`: both bodies drafted as dwarf fighters through the same
    // sheet buttons, with longsword, steel shield and scale mail readied.
    if args.next().as_deref() == Some("--ready") {
        for body in bodies {
            let card = format!("sheet:{body}");
            click(&mut endpoint, SHEETS, &card, "vtt.sheet-create")?;
            for value in DWARF_FIGHTER {
                click(&mut endpoint, SHEETS, &card, &format!("vtt.draft-{value}"))?;
            }
            click(&mut endpoint, SHEETS, &card, "vtt.draft-finish")?;
            let items = host.read(|_, sections| -> Result<Vec<(String, String)>, String> {
                let state = isocosm_native::ruleset::State::read(sections)?.ok_or("no edition state")?;
                let inventory = &state.inventories[&isocosm_native::ruleset::token(body)?];
                Ok(inventory.items.iter().map(|(id, item)| (id.0.clone(), item.name.clone())).collect())
            })??;
            for (id, name) in items {
                if ["Longsword", "Steel Shield", "Scale Mail"].contains(&name.as_str()) {
                    click(&mut endpoint, SHEETS, &card, &format!("vtt.pf2e-equip-{id}"))?;
                }
            }
        }
    }
    println!(
        "Pathfinder Bench ledger {path}; bodies={bodies:?}; sequence={}",
        host.sequence()
    );
    Ok(())
}

/// Reopen the real saved ledger after the headed writer has closed. Opening
/// verifies the checkpoint and accepted suffix through the normal family host.
fn inspect(path: &str) -> Result<(), String> {
    let host = FamilyHost::resume_file(
        path,
        cosmograph::Execution::Grouped,
        FamilyLimits::default(),
        FamilyLedgerLimits::default(),
    )?;
    let report = host.read(|_, sections| -> Result<_, String> {
        let state = isocosm_native::ruleset::State::read(sections)?.ok_or("no saved edition state")?;
        Ok(serde_json::json!({
            "edition": isocosm_native::ruleset::edition(sections)?,
            "sheets": state.sheets.iter().map(|(id, sheet)| serde_json::json!({
                "body": id.0, "system": sheet.system, "hp": sheet.int("hp_current")
            })).collect::<Vec<_>>(),
            "drafts": state.drafts.iter().map(|(id, draft)| serde_json::json!({
                "body": id.0, "choices": draft.choices.len()
            })).collect::<Vec<_>>(),
            "log": state.log.iter().map(|entry| entry.note.clone()).collect::<Vec<_>>(),
        }))
    })??;
    println!(
        "{}",
        serde_json::to_string_pretty(
            &serde_json::json!({"sequence":host.sequence(),"reopened":report})
        )
        .map_err(|e| e.to_string())?
    );
    Ok(())
}
