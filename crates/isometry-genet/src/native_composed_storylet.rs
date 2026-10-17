// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isometry_views::{StoryletPhysicalRequest, StoryletPhysicalOutcome, StoryletPhysicalFileAction};

fn read(path: &str) -> Result<isocosm_vtt::native::storylets::ComposedStoryletInput, String> {
    if path.trim().is_empty() { return Err("choose a complete physical storylet input file".into()); }
    let bytes = std::fs::read(path.trim()).map_err(|error| format!("storylet input file: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("storylet input JSON: {error}"))
}

fn install(ui: &mut UiState, outcome: StoryletPhysicalOutcome) -> Result<(), String> {
    if !ui.can_edit_inventory { return Err("physical storylet quote belongs to the actual host".into()); }
    match outcome {
        StoryletPhysicalOutcome::Applied => {
            ui.native.storylet_physical_quote = None; ui.status = "accepted physical storylet inputs".into();
        },
        StoryletPhysicalOutcome::Quote { input, result } => {
            let world = ui.native.world.as_ref().ok_or("storylet quote has no native world")?;
            let table = ui.native.table.as_ref().ok_or("storylet quote has no admitted table")?;
            let campaign = world.load(isocosm::Execution::Grouped)?;
            if input.storylet.expected_native != isocosm::digest(&campaign.table_session().save())
                || input.expected_game != isocosm::digest(&table.game) || result.input != input
                || result.expected_table != isonetry::native_table::table_digest(Some(table))
                || result.item_owner != ui.selected_token
                || !read(ui.native.storylet_physical_path.text()).as_ref().is_ok_and(|current| current == &input) {
                return Err("physical storylet quote names a changed native or product cut".into());
            }
            ui.native.storylet_physical_quote = Some((input, result));
            ui.status = "quoted complete supplied physical storylet".into();
        },
    }
    Ok(())
}

impl App {
    pub(crate) fn pump_native_composed_storylet(&mut self, ctx: &mut Ctx<'_>) {
        ctx.runner.update(|ui| {
            if let Some((input, result)) = &ui.native.storylet_physical_quote {
                let changed = ui.native.world.as_ref().and_then(|world| world.load(isocosm::Execution::Grouped).ok())
                    .is_none_or(|campaign| input.storylet.expected_native != isocosm::digest(&campaign.table_session().save()))
                    || ui.native.table.as_ref().is_none_or(|table| input.expected_game != isocosm::digest(&table.game)
                        || result.expected_table != isonetry::native_table::table_digest(Some(table)))
                    || result.item_owner != ui.selected_token
                    || !read(ui.native.storylet_physical_path.text()).as_ref().is_ok_and(|current| current == input);
                if changed { ui.native.storylet_physical_quote = None; }
            }
            let Some(action) = ui.native.storylet_physical_file_request.take() else { return; };
            ui.native.storylet_physical_quote = None;
            match read(ui.native.storylet_physical_path.text()) {
                Ok(input) => ui.native.storylet_physical_request = Some(match action {
                    StoryletPhysicalFileAction::Quote => StoryletPhysicalRequest::Quote { input, item_owner: ui.selected_token },
                    StoryletPhysicalFileAction::Apply => StoryletPhysicalRequest::Apply { input, item_owner: ui.selected_token },
                }),
                Err(why) => ui.status = why,
            }
        });
        let mut request = None;
        ctx.runner.update(|ui| request = ui.native.storylet_physical_request.take());
        if let Some(net) = &mut self.net {
            for result in net.take_storylet_physical_results() { ctx.runner.update(|ui| {
                if let Err(why) = result.value.and_then(|outcome| install(ui, outcome)) { ui.status = why; }
            }); }
            if let Some(request) = request {
                if !self.net_is_host || !ctx.runner.state().can_edit_inventory {
                    ctx.runner.update(|ui| ui.status = "physical storylet authoring requires the actual host".into());
                } else if net.native_composed_storylet(request).is_none() {
                    ctx.runner.update(|ui| ui.status = "storylet authority actor is unavailable".into());
                }
            }
            return;
        }
        let Some(request) = request else { return; };
        if !ctx.runner.state().can_edit_inventory { return; }
        let snapshot = self.snapshot_of(ctx.runner.state());
        if let StoryletPhysicalRequest::Quote { input, item_owner } = &request {
            let authority = HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
            let outcome = authority.quote_composed_storylet(input, *item_owner)
                .map(|result| StoryletPhysicalOutcome::Quote { input: input.clone(), result });
            ctx.runner.update(|ui| { if let Err(why) = outcome.and_then(|outcome| install(ui, outcome)) { ui.status = why; } });
            return;
        }
        self.ensure_history_origin(&snapshot);
        let mut authority = HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
        let StoryletPhysicalRequest::Apply { input, item_owner } = request else { return; };
        match authority.commit_composed_storylet(&input, item_owner) {
            Ok(_) => {
                let snapshot = authority.state().clone();
                self.history = authority.history().clone(); self.journal = snapshot.journal.clone();
                ctx.runner.update(|ui| { ui.apply_snapshot(snapshot);
                    if let Err(why) = install(ui, StoryletPhysicalOutcome::Applied) { ui.status = why; } });
            },
            Err(why) => ctx.runner.update(|ui| ui.status = why),
        }
    }
}
