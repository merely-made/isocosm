// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A shared recorded map is a separate view; it never edits a member's knowing.

use cambium::{TextInput, caret_text_field, clickable, el, lens, text,
    graph_canvas_swatch_with_drag_and_relations};
use isometry_campaign::{Overmap, OvermapNode, founding::NativeCursor};
use isocosm::{party::SharedMap, schema::Id};
use crate::{UiState, UiChild};

#[derive(Clone, Debug)]
pub enum PartyRequest {
    Read { party: String }, Join { party: String, member: Id },
    Leave { party: String, member: Id }, Disclose { party: String, member: Id },
}

#[derive(Clone, Debug)]
pub struct PartyView {
    pub cursor: NativeCursor, pub source: String, pub table: Option<String>, pub participant: Id,
    pub subject: Option<Id>, pub shared: bool, pub map: SharedMap, pub here: Option<Id>,
    /// Only exact accepted disclosure keys are resolved into native facts.
    pub events: std::collections::BTreeMap<String, Option<isocosm::schema::Event>>,
}

impl PartyView {
    /// Only the legacy graph's node layout is reused. Native route labels keep
    /// their full u64 travel values in the Swatch, outside its u32 edge type.
    pub fn layout(&self) -> Overmap {
        let nodes = self.map.places.values().map(|place| OvermapNode {
            id: format!("site:{}", place.site), name: place.authored.as_ref().map(|p| p.name.clone())
                .unwrap_or_else(|| format!("Site {}", place.site)),
            // A presentation anchor only. It drives neither geometry nor travel.
            at: place.authored.as_ref().and_then(|p| p.position).unwrap_or((0, 0)),
            site: place.authored.as_ref().and_then(|p| p.map.clone()),
        }).collect();
        Overmap { name: self.map.party.clone(), nodes, edges: Vec::new() }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartyUi { pub handle: TextInput, pub request: Option<PartyRequest>, pub reading: Option<PartyView> }

impl UiState {
    pub fn request_party(&mut self, request: PartyRequest) {
        self.native.party.request = Some(request);
    }
    pub fn native_party_reading(&self) -> Option<&PartyView> {
        self.native.party.reading.as_ref().filter(|read| read.map.party == self.native.party.handle.text().trim())
    }
}

pub(crate) fn overlay(ui: &UiState) -> Option<UiChild> {
    let party = ui.native.party.handle.text().trim().to_owned();
    let mut actions: Vec<UiChild> = vec![Box::new(clickable(
        el::<_, UiState, ()>("span", text("close")).attr("class", "btn btn-mini"),
        |ui: &mut UiState, _| ui.close_overmap()))];
    let mut requests = vec![("Read party context/map", PartyRequest::Read { party: party.clone() })];
    if let Some(member) = ui.native_party_reading().and_then(|read| read.subject) {
        requests.extend([
            ("Opt this played body in", PartyRequest::Join { party: party.clone(), member }),
            ("Disclose this body's known map", PartyRequest::Disclose { party: party.clone(), member }),
        ]);
    }
    for (label, request) in requests {
        actions.push(Box::new(clickable(el("button", text(label)).attr("class", "btn btn-mini"),
            move |ui: &mut UiState, _| ui.request_party(request.clone()))));
    }
    let mut body: Vec<UiChild> = vec![Box::new(el("label", (text("Authored party handle "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]), |ui: &mut UiState| &mut ui.native.party.handle))))];
    if let Some(read) = ui.native_party_reading() {
        if !read.shared {
            body.push(Box::new(el("div", text("Opt in before reading the shared party map."))
                .attr("class", "side-hint")));
        }
        body.push(Box::new(el("div", text(format!("{} disclosed sites · {} events · {} recorded disclosures",
            read.map.places.len(), read.map.events.len(), read.map.records.len()))).attr("class", "side-line")));
        for member in read.map.members.iter().filter(|m| m.participant == read.participant) {
            let request = PartyRequest::Leave { party: party.clone(), member: member.member };
            body.push(Box::new(clickable(el("button", text(format!("Withdraw my body {}", member.member)))
                .attr("class", "btn"), move |ui: &mut UiState, _| ui.request_party(request.clone()))));
        }
        if let Some(swatch) = crate::overmap::overmap_swatch(ui) {
            body.push(Box::new(graph_canvas_swatch_with_drag_and_relations(&swatch,
                |ui: &mut UiState, id| ui.status = format!("Disclosed {id}; collective travel is not selected"),
                |ui: &mut UiState, id| ui.hover_overmap(id),
                |ui: &mut UiState, drag| { ui.drag_overmap_node(drag); },
                |ui: &mut UiState, id| ui.select_overmap_relation(id),
                |ui: &mut UiState, id| ui.hover_overmap_relation(id),
                |_ui: &mut UiState| {})));
        }
        for (key, event) in &read.events {
            let Some(event) = event else {
                body.push(Box::new(el("div", text(format!("{key}: disclosed event is no longer available")))
                    .attr("class", "side-line")));
                continue;
            };
            let place = match event.place {
                isocosm::schema::EventPlace::Site(site) if site == isocosm::directing::PLACELESS => "no site".into(),
                isocosm::schema::EventPlace::Site(site) => format!("site {site}"),
                isocosm::schema::EventPlace::Journey(j) => format!("in transit {} to {} ({}–{})",
                    j.from, j.to, j.departed, j.arrives),
            };
            let toward = event.object.map(|id| format!(" toward {id}")).unwrap_or_default();
            body.push(Box::new(el("div", text(format!("{key} · tick {} · {}{} · {} · {place}",
                event.tick, event.subject, toward, event.process))).attr("class", "side-line")));
            if let Some(history) = &event.authored {
                body.push(Box::new(el("div", text(format!("{}: {}", history.kind, history.text)))
                    .attr("class", "side-line")));
            }
        }
    } else {
        body.push(Box::new(el("div", text("Choose an explicitly declared party and read its shared map."))
            .attr("class", "side-hint")));
    }
    body.push(Box::new(el("div", text("Only deliberately disclosed facts enter this map. Membership grants no control."))
        .attr("class", "side-hint")));
    Some(crate::widgets::overlay_panel("overmap", "Shared party map".into(), actions, body))
}
