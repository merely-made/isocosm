// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Existing accepted native attention and knowing; reading adds no world acts.

use super::*;
use isonetry::native_table::{TableSender,table_digest};
use isocosm_overlay::Seat;
use isometry_views::NativeEventStream;

fn signature(ui:&UiState,sender:TableSender)->Result<String,String> {
    let world=ui.native.world.as_ref().ok_or("native event stream has no accepted world")?;
    let caller=match sender {TableSender::Host=>(0u8,None),TableSender::Endpoint(key)=>(1u8,Some(key))};
    Ok(isocosm::digest(&(world.cursor()?,table_digest(ui.native.table.as_ref()),caller)))
}

fn read(ui:&UiState,sender:TableSender)->Result<NativeEventStream,String> {
    let world=ui.native.world.as_ref().ok_or("native event stream has no accepted world")?;
    let table=ui.native.table.as_ref().ok_or("native event stream needs its explicit participant binding")?;
    let campaign=world.load(isocosm::Execution::Grouped)?;
    let session=campaign.table_session();
    table.validate_history(session)?;
    let participant=table.participant(sender)?;
    let subject=session.sim.plays(participant);
    let seat=if matches!(sender,TableSender::Host) {Seat::Dm} else {Seat::Player};
    let mode=if seat==Seat::Player {isocosm::directing::readings::Mode::Survival}
        else {isocosm::directing::readings::Mode::Creative};
    let visible=isocosm::directing::readings::view(&session.sim,subject.unwrap_or(u64::MAX),mode);
    let events=session.sim.attended_events(participant,mode)?.into_iter()
        .filter(|event|visible.events.contains(&event.id)).collect();
    Ok(NativeEventStream {cursor:world.cursor()?,table:table_digest(Some(table)),participant,subject,seat,events})
}

impl App {
    pub(crate) fn refresh_native_events(&mut self,ctx:&mut Ctx<'_>) {
        if ctx.runner.state().native.world.is_none() {
            if ctx.runner.state().native.events.signature.is_some() || ctx.runner.state().native.events.reading.is_some()
                || ctx.runner.state().native.events.refusal.is_some() {
                ctx.runner.update(|ui|ui.native.events=Default::default());
            }
            return;
        }
        let sender=match &self.net {
            Some(net)=>net.local_sender(),
            None if ctx.runner.state().net_mode==NetMode::Local=>Some(TableSender::Host),
            None=>None,
        };
        let source=sender.ok_or_else(||"native event stream has no current authenticated local actor".to_string())
            .and_then(|sender|signature(ctx.runner.state(),sender).map(|source|(sender,source)));
        let (sender,source)=match source {
            Ok(source)=>source,
            Err(why)=>{
                if ctx.runner.state().native.events.reading.is_some()
                    || ctx.runner.state().native.events.refusal.as_ref()!=Some(&why) {
                    ctx.runner.update(|ui| {
                        ui.native.events.signature=None;ui.native.events.reading=None;ui.native.events.refusal=Some(why);
                    });
                }
                return;
            },
        };
        if ctx.runner.state().native.events.signature.as_ref()==Some(&source) {return;}
        let result=read(ctx.runner.state(),sender);
        ctx.runner.update(|ui| {
            ui.native.events.signature=Some(source);
            ui.native.events.reading=None;ui.native.events.refusal=None;
            match result {Ok(read)=>ui.native.events.reading=Some(read),Err(why)=>ui.native.events.refusal=Some(why)}
        });
    }
}
