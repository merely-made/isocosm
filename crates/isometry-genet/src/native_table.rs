// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! One actual request path for standalone/solo native table bindings and moves.

use super::*;
use isometry_views::TableFileAction;
use std::io::Write;

fn read_input<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    if path.trim().is_empty() { return Err("choose an explicit native table input file".into()); }
    let bytes = std::fs::read(path.trim()).map_err(|error| format!("native table file: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("native table input: {error}"))
}

/// Read-only authoring context, including incomplete templates. A null
/// participant is deliberately not an admitted Existing/Join input.
fn export_context(ui: &UiState) -> Result<(), String> {
    let path = ui.native.export_path.text().trim();
    if path.is_empty() { return Err("choose an explicit authoring export file".into()); }
    let world = ui.native.world.as_ref().ok_or("no accepted native world to export")?;
    let campaign = world.load(isocosm::Execution::Grouped)?;
    let session = campaign.table_session();
    let state = session.sim.state();
    let participants: Vec<_> = state.population.groups.iter()
        .filter(|(_, group)| isocosm::directing::is_participant(&group.entity))
        .map(|(first, group)| serde_json::json!({"first": first, "count": group.count})).collect();
    let characters: Vec<_> = state.population.groups.iter().filter_map(|(first, group)|
        group.entity.authored.as_ref().map(|fill| serde_json::json!({
            "first": first, "count": group.count, "fill": fill, "alive": group.entity.alive,
            "resident_site": group.entity.resident_site(), "arrived": group.entity.arrived,
            "body_revision": group.entity.body_revision,
        }))).collect();
    let expected_native = world.cursor()?;
    let expected_table = isonetry::native_table::table_digest(ui.native.table.as_ref());
    let binding = ui.native.table.as_ref().map(|table| isonetry::NativeTableBindingInput {
        expected_native: expected_native.clone(), expected_table: expected_table.clone(), table: table.clone(),
    });
    let accepted_game = ui.native.table.as_ref().map(|table|
        isometry_campaign::native::GameBindings::decode(&table.game)).transpose()?;
    let boards: Vec<_> = ui.campaign_maps.iter().map(|(key, map)| serde_json::json!({
        "map": key, "ground_digest": isonetry::native_table::ground_digest(
            if ui.active_map.as_ref() == Some(key) { &ui.map } else { &map.document }),
        "tokens": if ui.active_map.as_ref() == Some(key) { &ui.map.tokens } else { &map.document.tokens },
    })).collect();
    let value = serde_json::json!({
        "purpose": "Author explicit inputs; null participant and missing cell mappings are incomplete, not defaults.",
        "admission_template": {"expected_native": expected_native, "expected_table": expected_table,
            "participant": null, "batch": {"version": 2, "assertions": [], "characters": [],
                "placements": [], "controls": []}},
        "binding_input": binding, "accepted_game": accepted_game, "participants": participants,
        "authored_characters": characters, "sites": state.sites, "lineages": state.lineages,
        "accounts": session.sim.genesis().rules.accounts, "boards": boards,
        "authenticated_endpoints": ui.native.endpoints,
    });
    let bytes = serde_json::to_vec_pretty(&value).map_err(|error| error.to_string())?;
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("authoring export (existing files are preserved): {error}"))?;
    file.write_all(&bytes).map_err(|error| format!("authoring export write: {error}"))
}

impl App {
    pub(crate) fn pump_native_table(&mut self, ctx: &mut Ctx<'_>) {
        ctx.runner.update(|ui| {
            let Some(action) = ui.native.file_request.take() else { return; };
            let result = match action {
                TableFileAction::Admit => read_input(ui.native.admission_path.text()).map(|input|
                    ui.native.admission_request = Some(input)),
                TableFileAction::Bind => read_input(ui.native.binding_path.text()).map(|input|
                    ui.native.binding_request = Some(input)),
                TableFileAction::Export => export_context(ui),
            };
            ui.status = match result {
                Ok(()) => if matches!(action, TableFileAction::Export) { "exported native authoring context" }
                    else { "requested native table input" }.into(),
                Err(reason) => reason,
            };
        });
        let mut request = None;
        let mut admission = None;
        ctx.runner.update(|ui| {
            request = ui.native.binding_request.take();
            admission = ui.native.admission_request.take();
        });
        if self.net.is_some() {
            if let Some(input) = admission {
                if !self.net_is_host {
                    ctx.runner.update(|ui| ui.status = "native table admission requires the host".into());
                } else if let Some(net) = &mut self.net {
                    if net.commit_native_table_admission(input).is_none() {
                        ctx.runner.update(|ui| ui.status = "native table authority is unavailable".into());
                    }
                }
            }
            if let Some(input) = request {
                if !self.net_is_host {
                    ctx.runner.update(|ui| ui.status = "native table binding requires the host".into());
                } else if let Some(net) = &mut self.net {
                    if net.commit_native_table_binding(input).is_none() {
                        ctx.runner.update(|ui| ui.status = "native table authority is unavailable".into());
                    }
                }
            }
            return;
        }
        if ctx.runner.state().native.world.is_none() && request.is_none() && admission.is_none() { return; }
        let mut events = Vec::new();
        ctx.runner.update(|ui| events = std::mem::take(&mut ui.net_outbox));
        if request.is_none() && admission.is_none() && events.is_empty() { return; }
        let before = self.snapshot_of(ctx.runner.state());
        self.ensure_history_origin(&before);
        let mut authority = HostSession::with_history(before, self.campaign.clone(), self.history.clone());
        let mut refused = Vec::new();
        if let Some(input) = admission {
            if let Err(reason) = authority.commit_native_table_admission(&input) { refused.push(reason); }
        }
        if let Some(input) = request {
            if let Err(reason) = authority.commit_native_table_binding(&input) { refused.push(reason); }
        }
        for event in events {
            if let Err(reason) = authority.try_local_event(event) { refused.push(reason); }
        }
        let accepted = authority.state().clone();
        self.history = authority.history().clone();
        self.journal = accepted.journal.clone();
        ctx.runner.update(|ui| {
            ui.apply_snapshot(accepted);
            ui.status = if refused.is_empty() { "accepted native table update".into() }
                else { refused.join("\n") };
        });
    }
}
