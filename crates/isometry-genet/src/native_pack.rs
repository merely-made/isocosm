// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isometry_views::{PackReading, PackRequest};
use isocosm_vtt::native::pack::{self, Choreography, PackEnvelope, PackInput};
use isonetry::native_table::{TableSender, table_digest};
use std::collections::BTreeSet;

pub(crate) fn read(snapshot: &GameSnapshot, private: &CampaignStore) -> Result<PackReading, String> {
    let (campaign,access) = crate::native_party::context(snapshot,TableSender::Host)?;
    let session = campaign.table_session();
    let table = snapshot.native_table.as_ref().ok_or("adventure catalogue has no admitted native table")?;
    let private_ids: BTreeSet<_> = private.secret_ids().map(String::from).collect();
    Ok(PackReading { cursor:snapshot.native.as_ref().ok_or("pack world absent")?.cursor()?,
        source:isocosm::digest(snapshot.native.as_ref().ok_or("pack world absent")?),table:table_digest(Some(table)),
        packs:pack::read_packs(session,&table.game,access,&private_ids)?,
        active:pack::active_content(session,&table.game,access)? })
}

pub(crate) fn envelope(snapshot: &GameSnapshot, intent: PackInput) -> Result<PackEnvelope, String> {
    let (campaign,access) = crate::native_party::context(snapshot,TableSender::Host)?;
    Ok(PackEnvelope { participant:access.participant.0,tick:campaign.table_session().sim.state().tick,intent })
}

fn install(ui: &mut UiState, read: PackReading) -> Result<(), String> {
    let world = ui.native.world.as_ref().ok_or("pack read has no native world")?;
    if !ui.can_edit_inventory || world.cursor()? != read.cursor || isocosm::digest(world) != read.source
        || table_digest(ui.native.table.as_ref()) != read.table {
        return Err("pack response names a changed native world, game bytes or DM binding".into());
    }
    ui.native.packs.reading = Some(read);
    Ok(())
}

impl App {
    pub(crate) fn sync_native_choreography(&mut self, ctx: &mut Ctx<'_>) {
        let reading = (|| {
            let ui = ctx.runner.state();
            let Some(world) = &ui.native.world else { return Ok(Vec::new()); };
            let Some(table) = &ui.native.table else { return Ok(Vec::new()); };
            let campaign = world.load(isocosm::Execution::Grouped)?;
            isocosm_vtt::native::pack::accepted_choreography(campaign.table_session(), &table.game)
        })();
        let beats = match reading {
            Ok(beats) => beats,
            Err(reason) => { ctx.runner.update(|ui| ui.status = reason); return; }
        };
        if beats == self.native_choreography { return; }
        let mut sheet = self.sheet.clone();
        let mut emotes: std::collections::BTreeMap<_, _> = self.pack_emotes.iter().cloned().collect();
        for beat in &beats {
            sheet.push('\n'); sheet.push_str(&beat.css);
            if let Some(label) = &beat.emote { emotes.insert(beat.name.clone(), label.clone()); }
        }
        *ctx.set_sheet = Some(sheet);
        ctx.runner.update(|ui| ui.emotes = emotes.into_iter().collect());
        self.native_choreography = beats;
    }

    pub(crate) fn pump_native_packs(&mut self, ctx: &mut Ctx<'_>) {
        let mut request = None;
        ctx.runner.update(|ui| request = ui.native.packs.request.take());
        if !ctx.runner.state().can_edit_inventory { return; }
        if self.net.is_some() {
            if let Some(net) = &mut self.net {
                for response in net.take_pack_readings() {
                    ctx.runner.update(|ui| {
                        ui.status = match response.value.and_then(|read| install(ui,read)) {
                            Ok(()) => "read saved native adventure catalogue".into(),Err(why) => why,
                        };
                    });
                }
            }
            let Some(request) = request else { return; };
            if !self.net_is_host { ctx.runner.update(|ui| ui.status = "pack decisions require the actual host".into()); return; }
            let choreography = match &request {
                PackRequest::Generated { record,.. } => self.generator_catalog.pack_choreography(&record.request.generator),
                _ => Ok(Vec::new()),
            };
            match choreography {
                Ok(beats) => if let Some(net) = &mut self.net {
                    if net.native_pack(request,beats.into_iter().map(|beat| Choreography {
                        name:beat.name,emote:beat.emote,css:beat.css }).collect()).is_none() {
                        ctx.runner.update(|ui| ui.status = "pack authority actor is unavailable".into());
                    }
                },
                Err(why) => ctx.runner.update(|ui| ui.status = why),
            }
            return;
        }
        let snapshot = self.snapshot_of(ctx.runner.state());
        let Some(request) = request else { return; };
        if matches!(&request,PackRequest::Read) {
            let result = read(&snapshot,&self.campaign);
            ctx.runner.update(|ui| ui.status = match result.and_then(|read| install(ui,read)) {
                Ok(()) => "read saved native adventure catalogue".into(),Err(why) => why,
            });
            return;
        }
        self.ensure_history_origin(&snapshot);
        let mut authority = HostSession::with_history(snapshot,self.campaign.clone(),self.history.clone());
        let result = match request {
            PackRequest::Generated { record,requirements,roles } => self.generator_catalog.pack_choreography(&record.request.generator)
                .and_then(|beats| authority.register_generated_pack(&record,requirements,roles,
                    beats.into_iter().map(|beat| Choreography { name:beat.name,emote:beat.emote,css:beat.css }).collect(),
                    GeneratorLimits::default().max_value_depth)),
            PackRequest::CurrentContent => (|| {
                let (campaign,_) = crate::native_party::context(authority.state(),TableSender::Host)?;
                let input = envelope(authority.state(),PackInput::Content { content:campaign.content().clone() })?;
                authority.native_pack(TableSender::Host,&input)
            })(),
            PackRequest::Submit(intent) => envelope(authority.state(),intent)
                .and_then(|input| authority.native_pack(TableSender::Host,&input)),
            PackRequest::Read => unreachable!(),
        };
        let snapshot = authority.state().clone();
        self.campaign = authority.campaign().clone();
        self.history = authority.history().clone();
        self.journal = snapshot.journal.clone();
        ctx.runner.update(|ui| {
            ui.apply_snapshot(snapshot);
            ui.native.packs.reading = None;
            if result.is_ok() { ui.native.packs.request = Some(PackRequest::Read); }
            ui.status = match result { Ok(_) => "accepted native pack decision; read the catalogue".into(),Err(why) => why };
        });
    }
}
