// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm_views::{ItemsRequest,ItemsOutcome,ItemsFileAction};
use isonetry::native_table::TableSender;

fn read(path:&str)->Result<isocosm_native::native::items::ItemInput,String> {
    if path.trim().is_empty() {return Err("choose a complete item/custody/project input file".into());}
    let bytes=std::fs::read(path.trim()).map_err(|error|format!("item input file: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error|format!("item input JSON: {error}"))
}

fn install(ui:&mut UiState,outcome:ItemsOutcome)->Result<(),String> {
    match outcome {
        ItemsOutcome::Quote {input,preview}=>{
            let world=ui.native.world.as_ref().ok_or("item quote has no current native world")?;
            let table=ui.native.table.as_ref().ok_or("item quote has no current admitted table")?;
            let campaign=world.load(cosmograph::Execution::Grouped)?;
            if input.expected_native!=isocosm_native::native::items::source(campaign.table_session())
                || input.expected_game!=cosmograph::digest(&table.game) {
                return Err("item quote names a changed native or product cut".into());
            }
            ui.status=if preview.complete {"quoted complete native item/custody/project input"}
                else {"native item quote requires complete supplied bindings and debit selections"}.into();
            ui.native.items_quote=Some((input,preview));
        },
        ItemsOutcome::Applied=>{ui.native.items_quote=None;ui.status="accepted native item/custody/project input".into();},
        ItemsOutcome::Submitted=>{ui.native.items_quote=None;ui.status="submitted native item input; awaiting host admission".into();},
    }
    Ok(())
}

impl App {
    pub(crate) fn pump_native_items(&mut self,ctx:&mut Ctx<'_>) {
        ctx.runner.update(|ui| {
            let Some(action)=ui.native.items_file_request.take() else {return;};
            ui.native.items_quote=None;
            match read(ui.native.items_path.text()) {
                Ok(input)=>ui.native.items_request=Some(match action {
                    ItemsFileAction::Quote=>ItemsRequest::Quote(input),ItemsFileAction::Apply=>ItemsRequest::Apply(input)}),
                Err(why)=>ui.status=why,
            }
        });
        let mut request=None;
        ctx.runner.update(|ui|request=ui.native.items_request.take());
        if let Some(net)=&mut self.net {
            for result in net.take_items_results() {ctx.runner.update(|ui| {
                if let Err(why)=result.value.and_then(|outcome|install(ui,outcome)) {ui.status=why;}
            });}
            if let Some(request)=request {if net.native_items(request).is_none() {
                ctx.runner.update(|ui|ui.status="native item authority actor is unavailable".into());
            }}
            return;
        }
        let Some(request)=request else {return;};
        let snapshot=self.snapshot_of(ctx.runner.state());
        if let ItemsRequest::Quote(input)=&request {
            let authority=HostSession::with_history(snapshot,self.campaign.clone(),self.history.clone());
            let result=authority.quote_native_items(TableSender::Host,input)
                .map(|preview|ItemsOutcome::Quote {input:input.clone(),preview});
            ctx.runner.update(|ui| {if let Err(why)=result.and_then(|outcome|install(ui,outcome)) {ui.status=why;}});
            return;
        }
        self.ensure_history_origin(&snapshot);
        let mut authority=HostSession::with_history(snapshot,self.campaign.clone(),self.history.clone());
        let result=match request {ItemsRequest::Apply(input)=>authority.native_items(TableSender::Host,&input),
            ItemsRequest::Quote(_)=>unreachable!()};
        match result {
            Ok(_)=>{
                let snapshot=authority.state().clone();
                self.history=authority.history().clone();self.journal=snapshot.journal.clone();
                ctx.runner.update(|ui| {ui.apply_snapshot(snapshot);
                    if let Err(why)=install(ui,ItemsOutcome::Applied) {ui.status=why;}});
            },
            Err(why)=>ctx.runner.update(|ui|ui.status=why),
        }
    }
}
