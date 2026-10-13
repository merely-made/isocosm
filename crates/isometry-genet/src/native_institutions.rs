// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isometry_views::{InstitutionRequest,InstitutionOutcome,InstitutionFileAction};
use isonetry::native_table::{TableSender,table_digest};

fn read(path:&str)->Result<isonetry::NativeInstitutionInput,String> {
    if path.trim().is_empty() {return Err("choose a complete native institution input file".into());}
    let bytes=std::fs::read(path.trim()).map_err(|error|format!("institution input file: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error|format!("institution input JSON: {error}"))
}

fn install(ui:&mut UiState,outcome:InstitutionOutcome)->Result<(),String> {
    match outcome {
        InstitutionOutcome::Quote(quote)=>{
            let world=ui.native.world.as_ref().ok_or("institution quote has no current native world")?;
            let table=ui.native.table.as_ref().ok_or("institution quote has no current admitted table")?;
            let campaign=world.load(isocosm::Execution::Grouped)?;
            table.validate_history(campaign.table_session())?;
            if table_digest(Some(table)).as_deref()!=Some(quote.expected_table.as_str())
                || quote.input.expected_native!=isometry_campaign::native::institutions::source(campaign.table_session())
                || quote.input.tick!=campaign.table_session().sim.state().tick {
                return Err("institution quote names a changed native/product cut".into());
            }
            ui.status=if quote.preview.refusal.is_some() {"native institution quote refused"}
                else {"quoted supplied native institution input"}.into();
            ui.native.institutions.quote=Some(quote);
        },
        InstitutionOutcome::Applied=>{ui.native.institutions.quote=None;ui.status="accepted native institution input".into();},
        InstitutionOutcome::Submitted=>{ui.native.institutions.quote=None;ui.status="submitted institution input; awaiting host admission".into();},
    }
    Ok(())
}

impl App {
    pub(crate) fn pump_native_institutions(&mut self,ctx:&mut Ctx<'_>) {
        ctx.runner.update(|ui| {
            let Some(action)=ui.native.institutions.file_request.take() else {return;};
            ui.native.institutions.quote=None;
            match read(ui.native.institutions.path.text()) {
                Ok(input)=>ui.native.institutions.request=Some(match action {
                    InstitutionFileAction::Quote=>InstitutionRequest::Quote(input),
                    InstitutionFileAction::Apply=>InstitutionRequest::Apply(input)}),
                Err(why)=>ui.status=why,
            }
        });
        let mut request=None;
        ctx.runner.update(|ui|request=ui.native.institutions.request.take());
        if let Some(net)=&mut self.net {
            for result in net.take_institution_results() {ctx.runner.update(|ui| {
                if let Err(why)=result.value.and_then(|outcome|install(ui,outcome)) {ui.status=why;}
            });}
            if let Some(request)=request {if net.native_institution(request).is_none() {
                ctx.runner.update(|ui|ui.status="native institution authority actor is unavailable".into());
            }}
            return;
        }
        let Some(request)=request else {return;};
        let snapshot=self.snapshot_of(ctx.runner.state());
        if let InstitutionRequest::Quote(input)=&request {
            let authority=HostSession::with_history(snapshot,self.campaign.clone(),self.history.clone());
            let result=authority.quote_native_institution(TableSender::Host,input).map(InstitutionOutcome::Quote);
            ctx.runner.update(|ui| {if let Err(why)=result.and_then(|outcome|install(ui,outcome)) {ui.status=why;}});
            return;
        }
        self.ensure_history_origin(&snapshot);
        let mut authority=HostSession::with_history(snapshot,self.campaign.clone(),self.history.clone());
        let result=match request {InstitutionRequest::Apply(input)=>authority.native_institution(TableSender::Host,&input),
            InstitutionRequest::Quote(_)=>unreachable!()};
        match result {
            Ok(_)=>{
                let snapshot=authority.state().clone();
                self.history=authority.history().clone();self.journal=snapshot.journal.clone();
                ctx.runner.update(|ui| {ui.apply_snapshot(snapshot);
                    if let Err(why)=install(ui,InstitutionOutcome::Applied) {ui.status=why;}});
            },
            Err(why)=>ctx.runner.update(|ui|ui.status=why),
        }
    }
}
