// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit source authoring, individual learning/telling and selected sharing.

use super::*;
use isocosm_overlay::vtt::PartyAction;

pub(super) fn panel(ui: &UiState, read: &PartyView) -> Vec<UiChild> {
    let mut body: Vec<UiChild> = Vec::new();
    if read.dm {
        body.push(Box::new(el("label", (text("Authored native Location JSON "), lens(
            |field: &mut TextInput| caret_text_field(field, &[]),
            |ui: &mut UiState| &mut ui.native.party.location)))));
        body.push(Box::new(clickable(el("button", text("Admit supplied Location")).attr("class", "btn"),
            |ui: &mut UiState, _| match serde_json::from_str(ui.native.party.location.text()) {
                Ok(input) => ui.request_party(PartyRequest::AuthorLocation(input)),
                Err(error) => ui.status = format!("Location input: {error}"),
            })));
        for (id,location) in &read.location_authoring {
            body.push(Box::new(el("div",text(format!("Actual native Location {id}: {location:?}; this author read grants no body knowing")))
                .attr("class","side-line")));
        }
        body.push(Box::new(el("label", (text("Authored native statement JSON "), lens(
            |field: &mut TextInput| caret_text_field(field, &[]),
            |ui: &mut UiState| &mut ui.native.party.statement)))));
        body.push(Box::new(clickable(el("button", text("Admit supplied statement source")).attr("class", "btn"),
            |ui: &mut UiState, _| match serde_json::from_str(ui.native.party.statement.text()) {
                Ok(input) => ui.request_party(PartyRequest::AuthorStatement(input)),
                Err(error) => ui.status = format!("statement input: {error}"),
            })));
    }
    for (entry,details) in &read.map.details {
        for held in &details.statements {
            body.push(Box::new(el("div",text(format!("Disclosure {entry}: {:?} · {:?} · original source {} · revision {}",
                held.statement,held.meaning,held.source.hash,held.source.revision))).attr("class","side-line")));
        }
        for held in &details.terrain {
            let named=&held.source.observation.named;
            body.push(Box::new(el("div",text(format!("Disclosure {entry}: terrain {} {:?}..{:?} · {} observed cells; other cells unknown · {:?} · revision {} · hash {}",
                named.name,named.min,named.max,held.source.cells.len(),held.meaning,
                held.source.observation.eye.terrain_revision,held.hash))).attr("class","side-line")));
        }
    }
    let Some(known) = &read.knowledge else {
        body.push(Box::new(el("div", text("Learning or telling needs an actual played native body."))
            .attr("class", "side-hint")));
        return body;
    };
    body.push(Box::new(el("div", text(format!("Body {} · authored source cut {} · game {}",
        known.holder,known.native,known.game))).attr("class", "side-line")));
    for (id, statement, source) in &known.sources {
        body.push(Box::new(el("div", text(format!("Admitted source {id}: {statement:?} · hash {} · revision {}",
            source.hash,source.revision))).attr("class", "side-line")));
        let request = PartyRequest::Decision { party: read.map.party.clone(), member: known.holder,
            action: PartyAction::ObserveStatement { source_note: id.clone() } };
        body.push(Box::new(clickable(el("button",text("Learn this statement source")).attr("class","btn btn-mini"),
            move |ui: &mut UiState, _| ui.request_party(request.clone()))));
    }
    for held in &known.held.statements {
        body.push(Box::new(el("div", text(format!("Held {}: {:?} · {:?} · source {} · revision {}",
            held.id,held.statement,held.meaning,held.source.hash,held.source.revision))).attr("class", "side-line")));
    }
    for held in &known.held.terrain {
        let named = &held.source.observation.named;
        body.push(Box::new(el("div", text(format!("Held {}: terrain {} {:?}..{:?} · {} known cells · {:?} · revision {}",
            held.id,named.name,named.min,named.max,held.source.cells.len(),held.meaning,
            held.source.observation.eye.terrain_revision))).attr("class", "side-line")));
    }
    if read.shared {
        for id in known.held.statements.iter().map(|held| &held.id).chain(known.held.terrain.iter().map(|held| &held.id)) {
            let request = PartyRequest::Decision { party: read.map.party.clone(), member: known.holder,
                action: PartyAction::DiscloseDetails { sites: Vec::new(), events: Vec::new(), held: vec![id.clone()] } };
            body.push(Box::new(clickable(el("button", text(format!("Disclose held {id}"))).attr("class", "btn btn-mini"),
                move |ui: &mut UiState, _| ui.request_party(request.clone()))));
        }
    }
    for (id, location) in &known.locations {
        body.push(Box::new(el("div", text(format!("Known Location {id} · {} identity sources; {} member and {} extent statements",
            location.identity.len(),location.members.len(),location.extent.len()))).attr("class", "side-line")));
    }
    body.push(Box::new(el("label", (text("Exact telling, named terrain window or selected disclosure JSON "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]), |ui: &mut UiState| &mut ui.native.party.details)))));
    let party = read.map.party.clone();
    let member = known.holder;
    body.push(Box::new(clickable(el("button",text("Submit supplied knowledge action")).attr("class","btn"),
        move |ui: &mut UiState, _| match serde_json::from_str(ui.native.party.details.text()) {
            Ok(action) => ui.request_party(PartyRequest::Decision { party: party.clone(), member, action }),
            Err(error) => ui.status = format!("knowledge action: {error}"),
        })));
    body.push(Box::new(el("div", text("Statements may be wrong. Retained sources never become live terrain, claims or undisclosed Location boundaries."))
        .attr("class", "side-hint")));
    body
}
