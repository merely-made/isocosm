// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{TextInput, caret_text_field, clickable, el, lens, text};
use crate::{board::UiChild, state::UiState};
use crate::state::native::TableFileAction;

pub(super) fn selection(ui: &UiState) -> Vec<UiChild> {
    let mut body: Vec<UiChild> = vec![Box::new(el("div", (
        clickable(el("span", text("Table only")).attr("class", "btn"),
            |ui: &mut UiState, _| ui.choose_native_world(false)),
        clickable(el("span", text("Native world")).attr("class", "btn"),
            |ui: &mut UiState, _| ui.choose_native_world(true)),
    )).attr("class", "btn-row"))];
    if !ui.native.enabled { return body; }
    body.push(Box::new(el("div", (text("World profile file "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]),
        |ui: &mut UiState| &mut ui.native.profile_path,
    ))).attr("class", "sheet-row")));
    body.push(Box::new(el("div", (text("Site attachments file (optional override) "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]),
        |ui: &mut UiState| &mut ui.native.attachment_path,
    ))).attr("class", "sheet-row")));
    body.push(Box::new(clickable(el("div", text("Preview world")).attr("class", "btn"),
        |ui: &mut UiState, _| ui.preview_native_world())));
    if let Some(selected) = &ui.native.selection {
        body.push(Box::new(el("div", text(format!("{}: {} sites, {} living members; {} site attachments",
            selected.setup.name, selected.setup.founding.sites, selected.setup.founding.population,
            selected.setup.attachments.len()))).attr("class", "side-line")));
        for site in &selected.sites {
            body.push(Box::new(el("div", text(site.clone())).attr("class", "side-line")));
        }
    }
    body
}

pub(crate) fn table_controls(ui: &UiState) -> Vec<UiChild> {
    let mut body: Vec<UiChild> = vec![Box::new(el("div", text(
        "Native table: explicitly admit characters/control, then bind tokens, authenticated peers and authored cells."
    )).attr("class", "side-line"))];
    body.push(Box::new(el("div", (text("Admission JSON file "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]),
        |ui: &mut UiState| &mut ui.native.admission_path,
    ))).attr("class", "sheet-row")));
    body.push(Box::new(clickable(el("div", text("Admit native table input")).attr("class", "btn"),
        |ui: &mut UiState, _| ui.request_native_table_file(TableFileAction::Admit))));
    body.push(Box::new(el("div", (text("Binding JSON file "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]),
        |ui: &mut UiState| &mut ui.native.binding_path,
    ))).attr("class", "sheet-row")));
    body.push(Box::new(clickable(el("div", text("Apply explicit table bindings")).attr("class", "btn"),
        |ui: &mut UiState, _| ui.request_native_table_file(TableFileAction::Bind))));
    body.push(Box::new(el("div", (text("Authoring export file "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]),
        |ui: &mut UiState| &mut ui.native.export_path,
    ))).attr("class", "sheet-row")));
    body.push(Box::new(clickable(el("div", text("Export current native authoring context")).attr("class", "btn"),
        |ui: &mut UiState, _| ui.request_native_table_file(TableFileAction::Export))));
    if let Some(table) = &ui.native.table {
        body.push(Box::new(el("div", text(format!("Native host participant {}; {} peer bindings; {} board bindings",
            table.host_participant, table.peers.len(), table.boards.len()))).attr("class", "side-line")));
    }
    for endpoint in &ui.native.endpoints {
        let key: String = endpoint.iter().map(|byte| format!("{byte:02x}")).collect();
        body.push(Box::new(el("div", text(format!("Authenticated endpoint: {key}"))).attr("class", "side-line")));
    }
    body
}
