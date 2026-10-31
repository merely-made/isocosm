// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Typed requests use the real saved host or transport endpoint binding. The
//! existing authority publishes native history and the table result atomically.

use super::*;
use cosmograph_overlay::{Intent, IntentEnvelope, ParticipantHandle, Tick,
    vtt::{TimeIntent, VttIntent, VttIntentEnvelope}};
use isonetry::native_table::TableSender;

pub(crate) fn envelope(snapshot:&GameSnapshot, sender:TableSender, intent:TimeIntent)
    -> Result<VttIntentEnvelope, String> {
    let world = snapshot.native.as_ref().ok_or("native time needs an accepted world")?;
    let table = snapshot.native_table.as_ref().ok_or("native time needs explicit player bindings")?;
    let campaign = world.load(cosmograph::Execution::Grouped)?;
    table.validate_history(campaign.table_session())?;
    Ok(IntentEnvelope { participant:ParticipantHandle(table.participant(sender)?),
        tick:Tick(campaign.table_session().sim.state().tick), intent:Intent::Game(VttIntent::Time(intent)) })
}

impl App {
    pub(crate) fn pump_native_time(&mut self, ctx:&mut Ctx<'_>) {
        let mut requests = Vec::new();
        ctx.runner.update(|ui| requests = std::mem::take(&mut ui.native.time_requests));
        if requests.is_empty() { return; }
        if let Some(net) = &mut self.net {
            for intent in requests {
                if net.native_time(intent).is_none() {
                    ctx.runner.update(|ui| ui.status = "native time authority is unavailable".into());
                }
            }
            return;
        }
        let before = self.snapshot_of(ctx.runner.state());
        self.ensure_history_origin(&before);
        let mut authority = HostSession::with_history(before, self.campaign.clone(), self.history.clone());
        let mut refused = Vec::new();
        for intent in requests {
            let result = (|| {
                let input = envelope(authority.state(), TableSender::Host, intent)?;
                let roster = authority.state().native_table.as_ref().ok_or("no admitted table")?.time_roster()?;
                authority.commit_native_time(TableSender::Host, &input, &roster)
            })();
            if let Err(reason) = result { refused.push(reason); }
        }
        let accepted = authority.state().clone();
        self.history = authority.history().clone();
        self.journal = accepted.journal.clone();
        ctx.runner.update(|ui| {
            ui.apply_snapshot(accepted);
            ui.status = if refused.is_empty() { "accepted native time decision".into() } else { refused.join("\n") };
        });
    }
}
