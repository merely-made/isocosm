// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authored departures, own-body approvals and selected DM publication.

use cambium::{TextInput,caret_text_field,clickable,el,lens,text};
use isocosm_vtt::native::joint_journey::{DepartureInput,DepartureResult};
use isocosm_overlay::EventHandle;
use crate::{UiChild,UiState,PartyRequest,PartyView};

#[derive(Clone,Debug)]
pub enum JourneyRequest {
    Quote { party:String,input:DepartureInput },
    Propose { party:String,departures:Vec<DepartureInput> },
    Approve { proposal:EventHandle },
    Apply { proposal:EventHandle },
}

#[derive(Clone,Debug,Default)]
pub struct JourneyUi { pub departures:TextInput,pub quote_input:TextInput }

#[derive(Clone,Debug)]
pub struct JourneyQuote { pub input:DepartureInput,pub native:isocosm::projects::DebitPreview }

pub(crate) fn panel(_ui: &UiState, read: &PartyView) -> Vec<UiChild> {
    let mut body: Vec<UiChild> = vec![Box::new(el("div",text(
        "Joint departure needs each listed player's own approval. Native clock consent is separate."))
        .attr("class","side-hint"))];
    for context in &read.journey_authoring {
        body.push(Box::new(el("div",text(context.clone())).attr("class","side-line")));
    }
    if read.dm {
        body.push(Box::new(el("label",(text("One explicit departure input JSON for native debit quote "),lens(
            |field:&mut TextInput| caret_text_field(field,&[]),
            |ui:&mut UiState| &mut ui.native.party.journey.quote_input)))));
        body.push(Box::new(clickable(el("button",text("Read native stock requests")).attr("class","btn"),
            |ui:&mut UiState,_| {
                match serde_json::from_str(ui.native.party.journey.quote_input.text()) {
                    Ok(input) => ui.request_party(PartyRequest::Journey(JourneyRequest::Quote {
                        party:ui.native.party.handle.text().trim().into(),input })),
                    Err(why) => ui.status = format!("departure input: {why}"),
                }
            })));
        body.push(Box::new(el("label",(text("Explicit ordered DepartureInput array JSON, including chosen lot portions "),lens(
            |field:&mut TextInput| caret_text_field(field,&[]),
            |ui:&mut UiState| &mut ui.native.party.journey.departures)))));
        body.push(Box::new(clickable(el("button",text("Propose this explicit group")).attr("class","btn"),
            |ui:&mut UiState,_| {
                match serde_json::from_str(ui.native.party.journey.departures.text()) {
                    Ok(departures) => ui.request_party(PartyRequest::Journey(JourneyRequest::Propose {
                        party:ui.native.party.handle.text().trim().into(),departures })),
                    Err(why) => ui.status = format!("departure group: {why}"),
                }
            })));
    }
    if let Some(quote) = &read.journey_quote {
        body.push(Box::new(el("div",text(format!("Native debit read for body {} · process {} · profile {}",
            quote.input.body,quote.input.process,quote.input.profile))).attr("class","side-line")));
        match serde_json::to_string(&quote.native) {
            Ok(json) => body.push(Box::new(el("pre",text(json)))),
            Err(why) => body.push(Box::new(el("div",text(format!("native quote unavailable: {why}"))))),
        }
        body.push(Box::new(el("div",text(
            "Choose exact source/account/amount/operation/ordinal and lot quantities/unowned; this read approves nothing."))
            .attr("class","side-hint")));
    }
    for view in &read.journeys {
        let proposal = &view.proposal;
        body.push(Box::new(el("div",text(format!("{} · native tick {} · proposal event {}",
            proposal.declaration.party,proposal.declaration.tick,proposal.witness.event.0))).attr("class","side-line")));
        for departure in &proposal.declaration.departures {
            let input = &departure.input;
            body.push(Box::new(el("div",text(format!("Player {} · body {} · {:?} {} to {} · process {} · profile {} · {} ticks",
                input.participant,input.body,departure.kind,input.from,input.to,input.process,input.profile,
                departure.journey.arrives - departure.journey.departed))).attr("class","side-line")));
            match serde_json::to_string(&input.debits) {
                Ok(json) => body.push(Box::new(el("pre",text(format!("Exact signed stocks: {json}"))))),
                Err(why) => body.push(Box::new(el("div",text(format!("stock terms unavailable: {why}"))))),
            }
        }
        for approval in &proposal.approvals {
            body.push(Box::new(el("div",text(format!("Player {} approved body {} · native entry {}",
                approval.participant,approval.body,approval.witness.entry))).attr("class","side-line")));
        }
        if let Some(body_id) = view.can_approve {
            let proposal = proposal.witness.event;
            body.push(Box::new(clickable(el("button",text(format!("Approve my current body {body_id}"))).attr("class","btn"),
                move |ui:&mut UiState,_| ui.request_party(PartyRequest::Journey(JourneyRequest::Approve { proposal })))));
        }
        if view.ready_for_dm {
            let proposal = proposal.witness.event;
            body.push(Box::new(clickable(el("button",text("Apply these approved native departures/goals")).attr("class","btn"),
                move |ui:&mut UiState,_| ui.request_party(PartyRequest::Journey(JourneyRequest::Apply { proposal })))));
        }
        if let Some(done) = &proposal.completed {
            for result in &done.departures {
                let label = match result {
                    DepartureResult::Departed { body,journey,.. } => format!("Body {body} actually departed {} to {} at {}, due {}",
                        journey.from,journey.to,journey.departed,journey.arrives),
                    DepartureResult::Goal { body,.. } => format!("Body {body} accepted a native travel goal; own future choice may split"),
                };
                body.push(Box::new(el("div",text(label)).attr("class","side-line")));
            }
        } else if let Some(why) = &view.refusal {
            body.push(Box::new(el("div",text(why.clone())).attr("class","side-hint")));
        }
    }
    body
}
