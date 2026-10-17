// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Derived local event stream, separate from deliberate party disclosures.

use crate::{UiState,UiChild};
use cambium::{el,text};
use isocosm_vtt::founding::NativeCursor;
use isocosm_overlay::Seat;

#[derive(Clone,Debug)]
pub struct NativeEventStream {
    pub cursor:NativeCursor,pub table:Option<String>,
    pub participant:u64,pub subject:Option<u64>,pub seat:Seat,
    pub events:Vec<isocosm::schema::Event>,
}

/// Unsaved UI projection. A changed actor/prefix withdraws the old read.
#[derive(Clone,Debug,Default)]
pub struct NativeEventUi {
    pub signature:Option<String>,pub reading:Option<NativeEventStream>,pub refusal:Option<String>,
}

pub(crate) fn panel(ui:&UiState)->UiChild {
    let mut rows:Vec<UiChild>=Vec::new();
    if ui.native.world.is_none() {return Box::new(el("div",rows));}
    rows.push(Box::new(el("div",text("Native event stream")).attr("class","side-heading")));
    let current=ui.native.events.reading.as_ref().filter(|read|
        ui.native.world.as_ref().is_some_and(|world|world.cursor().is_ok_and(|cursor|cursor==read.cursor))
            && isonetry::native_table::table_digest(ui.native.table.as_ref())==read.table);
    if let Some(read)=current {
        rows.push(Box::new(el("div",text(format!("Participant {} · played {:?} · {:?} · native tick {}",
            read.participant,read.subject,read.seat,read.cursor.tick))).attr("class","side-line")));
        if read.events.is_empty() {
            rows.push(Box::new(el("div",text("No events currently reach this participant's admitted attention."))
                .attr("class","side-hint")));
        }
        for event in &read.events {
            rows.push(Box::new(el("div",text(format!("{} · {} · {} · subject {} · object {:?}",
                event.tick,event.id,event.process,event.subject,event.object))).attr("class","side-line")));
            if let Some(authored)=&event.authored {
                rows.push(Box::new(el("div",text(authored.text.clone())).attr("class","side-hint")));
            }
        }
    } else {
        rows.push(Box::new(el("div",text(ui.native.events.refusal.clone()
            .unwrap_or_else(||"Native event reading needs its current authenticated participant.".into())))
            .attr("class","side-hint")));
    }
    Box::new(el("div",rows).attr("class","native-events"))
}
