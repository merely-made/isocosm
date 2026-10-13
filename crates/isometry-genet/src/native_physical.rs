// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The complete supplied author file reaches the same native publication door.

use super::*;
use isometry_views::{PhysicalRequest, PhysicalOutcome};
use isometry_campaign::native::physical_map::physical_source;

fn install(ui: &mut UiState, outcome: PhysicalOutcome) -> Result<(), String> {
    if !ui.can_edit_inventory { return Err("physical map authoring requires the actual host".into()); }
    match outcome {
        PhysicalOutcome::Applied => {
            ui.native.physical_quote = None;
            ui.status = "accepted physical map edit and native matter disposition".into();
        },
        PhysicalOutcome::Quote { input, preview } => {
            let world = ui.native.world.as_ref().ok_or("physical quote has no current native world")?;
            let table = ui.native.table.as_ref().ok_or("physical quote has no admitted native table")?;
            let campaign = world.load(isocosm::Execution::Grouped)?;
            let session = campaign.table_session();
            if input.expected_source != physical_source(session) || input.expected_game != isocosm::digest(&table.game)
                || input.expected_terrain != session.sim.terrain_revision()
                || session.sim.plays(table.host_participant) != Some(input.actor) {
                return Err("physical quote names a changed native source, game or actual DM body".into());
            }
            ui.status = if preview.complete { "quoted complete supplied physical edit" }
                else { "read measured physical quantities; supply exact native debit selections in the file" }.into();
            ui.native.physical_quote = Some((input, preview));
        },
    }
    Ok(())
}

impl App {
    pub(crate) fn pump_native_physical(&mut self, ctx: &mut Ctx<'_>) {
        let mut request = None;
        ctx.runner.update(|ui| request = ui.native.physical_request.take());
        if let Some(net) = &mut self.net {
            for result in net.take_physical_results() {
                ctx.runner.update(|ui| {
                    if let Err(why) = result.value.and_then(|outcome| install(ui, outcome)) { ui.status = why; }
                });
            }
            let Some(request) = request else { return; };
            if !self.net_is_host || !ctx.runner.state().can_edit_inventory {
                ctx.runner.update(|ui| ui.status = "physical map authoring requires the actual host".into());
            } else if net.native_physical(request).is_none() {
                ctx.runner.update(|ui| ui.status = "physical map authority actor is unavailable".into());
            }
            return;
        }
        let Some(request) = request else { return; };
        if !ctx.runner.state().can_edit_inventory {
            ctx.runner.update(|ui| ui.status = "physical map authoring requires the actual host".into());
            return;
        }
        let snapshot = self.snapshot_of(ctx.runner.state());
        if let PhysicalRequest::Quote(input) = &request {
            let authority = HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
            let result = authority.quote_native_physical(input)
                .map(|preview| PhysicalOutcome::Quote { input: input.clone(), preview });
            ctx.runner.update(|ui| {
                if let Err(why) = result.and_then(|outcome| install(ui, outcome)) { ui.status = why; }
            });
            return;
        }
        self.ensure_history_origin(&snapshot);
        let mut authority = HostSession::with_history(snapshot, self.campaign.clone(), self.history.clone());
        let result = match request {
            PhysicalRequest::Apply(input) => authority.native_physical(&input),
            PhysicalRequest::Quote(_) => unreachable!(),
        };
        if result.is_ok() {
            let snapshot = authority.state().clone();
            self.history = authority.history().clone();
            self.journal = snapshot.journal.clone();
            ctx.runner.update(|ui| {
                ui.apply_snapshot(snapshot);
                if let Err(why) = install(ui, PhysicalOutcome::Applied) { ui.status = why; }
            });
        } else if let Err(why) = result {
            ctx.runner.update(|ui| ui.status = why);
        }
    }
}
