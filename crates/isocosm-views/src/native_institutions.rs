// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit supplied constitutional operations, not table faction permissions.

use cambium::{TextInput,caret_text_field,clickable,el,lens,text};
use crate::{board::UiChild,state::UiState};
use isonetry::{NativeInstitutionInput,NativeInstitutionQuote};

#[derive(Clone,Debug,Default)]
pub struct InstitutionUi {
    pub path:TextInput, pub file_request:Option<InstitutionFileAction>,
    pub request:Option<InstitutionRequest>, pub quote:Option<NativeInstitutionQuote>,
}
#[derive(Clone,Copy,Debug)]
pub enum InstitutionFileAction { Quote,Apply }
#[derive(Clone,Debug)]
pub enum InstitutionRequest { Quote(NativeInstitutionInput),Apply(NativeInstitutionInput) }
#[derive(Clone,Debug)]
pub enum InstitutionOutcome { Quote(NativeInstitutionQuote),Applied,Submitted }

impl UiState {
    pub fn request_native_institution_file(&mut self,action:InstitutionFileAction) {
        if self.native.world.is_none() || self.native.table.is_none() {
            self.status="institution input needs an admitted native world and table".into();return;
        }
        self.native.institutions.quote=None;
        self.native.institutions.file_request=Some(action);
    }
}

pub(crate) fn controls(ui:&UiState)->Vec<UiChild> {
    if ui.native.world.is_none() {return Vec::new();}
    let mut rows:Vec<UiChild>=vec![Box::new(el("h4",text("Native institution operations")))];
    rows.push(Box::new(el("div",(text("Complete proposal, approval or application JSON file "),lens(
        |field:&mut TextInput|caret_text_field(field,&[]),|ui:&mut UiState|&mut ui.native.institutions.path
    ))).attr("class","sheet-row")));
    rows.push(Box::new(el("div",(
        clickable(el("span",text("Quote supplied input")).attr("class","btn"),
            |ui:&mut UiState,_|ui.request_native_institution_file(InstitutionFileAction::Quote)),
        clickable(el("span",text("Submit supplied input")).attr("class","btn"),
            |ui:&mut UiState,_|ui.request_native_institution_file(InstitutionFileAction::Apply)),
    )).attr("class","btn-row")));
    rows.push(Box::new(el("div",text("Native membership, deciding method, support and owned debits decide admission."))
        .attr("class","side-hint")));
    if let Some(quote)=&ui.native.institutions.quote {
        rows.push(Box::new(el("div",text(format!("Supplied native change: {:?}",quote.input.change)))
            .attr("class","side-line")));
        for request in &quote.preview.requests {
            rows.push(Box::new(el("div",text(format!("Native debit: {request:?}"))).attr("class","side-line")));
        }
        if let Some(receipt)=&quote.preview.receipt {
            rows.push(Box::new(el("div",text(format!("Quoted native receipt: {receipt:?}"))).attr("class","side-line")));
        }
        if let Some(why)=&quote.preview.refusal {
            rows.push(Box::new(el("div",text(why.clone())).attr("class","side-line")));
        }
    }
    rows
}
