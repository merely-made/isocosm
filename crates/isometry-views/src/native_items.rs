// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{TextInput,caret_text_field,clickable,el,lens,text};
use crate::{board::UiChild,state::UiState,ItemsFileAction};

pub(crate) fn controls(ui:&UiState)->Vec<UiChild> {
    if ui.native.world.is_none() {return Vec::new();}
    let mut rows:Vec<UiChild>=vec![Box::new(el("h4",text("Native items and projects")))];
    rows.push(Box::new(el("div",(text("Complete item/project JSON file "),lens(
        |field:&mut TextInput|caret_text_field(field,&[]),|ui:&mut UiState|&mut ui.native.items_path
    ))).attr("class","sheet-row")));
    rows.push(Box::new(el("div",(
        clickable(el("span",text("Quote supplied input")).attr("class","btn"),
            |ui:&mut UiState,_|ui.request_native_items_file(ItemsFileAction::Quote)),
        clickable(el("span",text("Submit supplied input")).attr("class","btn"),
            |ui:&mut UiState,_|ui.request_native_items_file(ItemsFileAction::Apply)),
    )).attr("class","btn-row")));
    if let Some((input,preview))=&ui.native.items_quote {
        rows.push(Box::new(el("div",text(format!("Actor {} · complete {}",input.actor,preview.complete)))
            .attr("class","side-line")));
        if let Some(acted)=&preview.project {
            rows.push(Box::new(el("div",text(format!("Native project {:?} · actual output {:?}",acted.project,acted.output)))
                .attr("class","side-line")));
        }
        if let Some(debits)=&preview.debits {for request in &debits.requests {
            rows.push(Box::new(el("div",text(format!("Native debit: {request:?}"))).attr("class","side-line")));
        }}
        if let Some(why)=&preview.refusal {rows.push(Box::new(el("div",text(why.clone())).attr("class","side-line")));}
    }
    rows
}
