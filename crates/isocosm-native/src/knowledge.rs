// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Held map knowledge: deliberate single-record learning and sharing by the
//! played body, at the player's own cut.

use chirograph::IntentEffect;
use cosmograph_host::endpoint::disclosure::{Item, NULL, action, card, value};
use cosmograph_overlay::{EntityHandle, Seat, vtt::{PartyAction, PartyIntent, VttIntent}};
use crate::native::party;

use super::{Act, Built, Cx, VttEndpoint};

pub(crate) fn build(endpoint: &VttEndpoint, cx: &Cx) -> Result<Built, String> {
    let mut built = Built::new();
    let note = |built: &mut Built, text: String| built.push(Item { id: "knowledge".into(),
        card: card("Held map knowledge", vec![value("Note", text)], &["note"]), actions: Vec::new() }, Vec::new());
    let access = cx.cut.access;
    if access.seat != Seat::Player {
        note(&mut built, "The player's body learns and discloses these records.".into());
        return Ok(built);
    }
    let Some(chosen) = cx.party else {
        note(&mut built, "Choose an authored party to share map knowledge.".into());
        return Ok(built);
    };
    let reading = endpoint.read(|session, game| party::read_knowledge(session, game, access)?
        .ok_or_else(|| "No currently played body holds these records.".to_string()));
    let reading = match reading {
        Ok(reading) => reading,
        Err(reason) => { note(&mut built, reason); return Ok(built); },
    };
    let intent = |action| VttIntent::Party(PartyIntent { party: chosen.to_owned(), member: EntityHandle(reading.holder),
        action });
    for (index, (source, statement, revision)) in reading.sources.iter().enumerate() {
        built.push(Item { id: format!("statement:{index}"), card: card(format!("{statement:?}"),
            vec![value("Source", &revision.note.core.object), value("Revision", &revision.revision)], &["statement"]),
            actions: vec![action("vtt.learn", "Learn this statement", "The played body learns this statement.", NULL,
                IntentEffect::DomainTruth)] },
            vec![Act::Send(intent(PartyAction::ObserveStatement { source_note: source.clone() }))]);
    }
    match party::detail_keys(&reading.held) {
        Err(reason) => note(&mut built, reason),
        Ok(keys) => for held in keys {
            built.push(Item { id: format!("held:{held}"), card: card(format!("Held record: {held}"), Vec::new(), &["held"]),
                actions: vec![action("vtt.share", "Disclose this record to the party",
                    "Share this held record with the chosen party.", NULL, IntentEffect::DomainTruth)] },
                vec![Act::Send(intent(PartyAction::DiscloseDetails { sites: Vec::new(), events: Vec::new(),
                    held: vec![held] }))]);
        },
    }
    if built.items.is_empty() { note(&mut built, "The played body holds no statements or records to share.".into()); }
    Ok(built)
}
