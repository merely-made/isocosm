//! Write a fresh, opt-in Bench family ledger for the desktop casting receipt.
//! Setup itself uses the same advertised sheet actions as a DM.
#[path = "../tests/native_founding/fixture.rs"]
mod content_fixture;
#[path = "../tests/support/table.rs"]
mod table;
use chirograph::{IntentInvocation, IntentResult};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm_host::world::World;
use isocosm_overlay::Seat;
use isocosm_vtt::{
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
        id: "magic-weapon-bench".into(),
        name: "Magic Weapon Bench".into(),
        maps: draft.maps,
        storylets: draft.world.storylets,
        rewards: vec![],
        starting_map: draft.starting_map,
        final_storylet: draft.final_storylet,
        table: TableMetadata::default(),
    };
    content.validate().map_err(|e| e.to_string())?;
    sections.insert("vtt".into(), GameBindings::decode(&[])?.encode()?);
    sections.insert(
        "isocosm-vtt:campaign".into(),
        serde_json::to_vec(&serde_json::json!({"content":content,"origin":null}))
            .map_err(|e| e.to_string())?,
    );
    sections.insert(
        "isocosm-vtt:table-access".into(),
        serde_json::to_vec(&serde_json::json!({"host_participant":dm.0,"peers":[]}))
            .map_err(|e| e.to_string())?,
    );
    let host = FamilyHost::create_file(
        &path,
        SessionId([73; 16]),
        World::with_sections(session, sections),
        FamilyLimits::default(),
        FamilyLedgerLimits::default(),
    )?;
    let mut endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm)?;
    for id in bodies {
        click(
            &mut endpoint,
            SHEETS,
            &format!("sheet:{id}"),
            "vtt.sheet-create",
        )?;
    }
    let caster = bodies[0];
    for action in ["vtt.sheet-learn", "vtt.sheet-give-longsword"] {
        click(&mut endpoint, SHEETS, &format!("sheet:{caster}"), action)?;
    }
    click(
        &mut endpoint,
        SHEETS,
        &format!("weapon:{caster}:ruleset:weapon:1"),
        "vtt.weapon-equip",
    )?;
    for _ in 0..2 {
        click(
            &mut endpoint,
            SHEETS,
            &format!("slots:{caster}:2"),
            "vtt.slot-2-1",
        )?;
    }
    click(&mut endpoint, TABLE, "roster", "vtt.roster-empty")?;
    println!(
        "Bench ledger {path}; caster={caster}, target={}; sequence={}",
        bodies[1],
        host.sequence()
    );
    Ok(())
}

/// Reopen the real saved ledger after the headed writer has closed. Opening
/// verifies the checkpoint and accepted suffix through the normal family host.
fn inspect(path: &str) -> Result<(), String> {
    let host = FamilyHost::resume_file(
        path,
        isocosm::Execution::Grouped,
        FamilyLimits::default(),
        FamilyLedgerLimits::default(),
    )?;
    let report = host.read(|session, sections| -> Result<_, String> {
        let state=isocosm_vtt::ruleset::State::read(sections)?.ok_or("no saved edition state")?;
        let map=state.map();
        if map.system_data != state.current_map(session)?.system_data {
            return Err("saved effect lifecycle differs from the accepted native clock".into());
        }
        let effects=isometry_system::active_effects::Records::read(&map)?.ok_or("no saved casts")?;
        Ok(serde_json::json!({
            "native_tick":session.sim.state().tick,
            "sheets":state.sheets.iter().map(|(id,sheet)|serde_json::json!({
                "body":id.0,"hp":sheet.int("hp_current"),"slot_2":sheet.int("slot:2")
            })).collect::<Vec<_>>(),
            "effects":effects.effects.values().map(|effect|serde_json::json!({
                "id":effect.id,"active":effect.applies(effects.tick),"expires":effect.expires,"ended":effect.ended
            })).collect::<Vec<_>>(),
            "log_entries":state.log.len(),
            "rolls":state.log.iter().flat_map(|entry|&entry.rolls).collect::<Vec<_>>(),
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
