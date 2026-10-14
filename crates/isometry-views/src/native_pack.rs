// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{TextInput, caret_text_field, clickable, el, lens, text};
use crate::{board::UiChild, state::UiState};
use isometry_campaign::{GenerationRecord, RoleSlot, StoryletRequirements};
use isometry_campaign::native::pack::{PackInput, PackView, ActiveContent};

#[derive(Clone, Debug)]
pub enum PackRequest {
    Read,
    Submit(PackInput),
    Generated { record: GenerationRecord, requirements: StoryletRequirements, roles: Vec<RoleSlot> },
    CurrentContent,
}

#[derive(Clone, Debug)]
pub struct PackReading {
    pub cursor: isometry_campaign::founding::NativeCursor,
    pub source: String, pub table: Option<String>, pub packs: Vec<PackView>, pub active: ActiveContent,
}

#[derive(Clone, Debug, Default)]
pub struct PackUi {
    pub definition: TextInput, pub requirements: TextInput, pub roles: TextInput,
    pub request: Option<PackRequest>, pub reading: Option<PackReading>,
}

impl UiState {
    pub fn request_native_pack(&mut self, request: PackRequest) {
        if !self.can_edit_inventory || self.native.world.is_none() {
            self.status = "adventure pack authoring requires the native campaign DM".into();
            return;
        }
        self.native.packs.request = Some(request);
    }

    fn register_pack_json(&mut self) {
        let input = serde_json::from_str(self.native.packs.definition.text()).map(|definition| PackInput::Register { definition });
        match input {
            Ok(intent) => self.request_native_pack(PackRequest::Submit(intent)),
            Err(why) => self.status = format!("explicit pack definition: {why}"),
        }
    }

    fn register_generated_pack(&mut self) {
        let result = (|| {
            let record = self.generator_preview.clone().ok_or("generate an adventure proposal first")?;
            let requirements = serde_json::from_str(self.native.packs.requirements.text()).map_err(|e| e.to_string())?;
            let roles = serde_json::from_str(self.native.packs.roles.text()).map_err(|e| e.to_string())?;
            Ok::<_,String>(PackRequest::Generated { record,requirements,roles })
        })();
        match result {
            Ok(request) => self.request_native_pack(request), Err(why) => self.status = why,
        }
    }
}

pub(crate) fn overlay(ui: &UiState) -> UiChild {
    let mut body: Vec<UiChild> = super::generator::founding::table_controls(ui);
    body.push(Box::new(el("div",text("Adventure packs remain dormant until preview and explicit DM acceptance."))));
    body.push(Box::new(clickable(el("button",text("Read saved catalogue")).attr("class","btn"),
        |ui: &mut UiState,_| ui.request_native_pack(PackRequest::Read))));
    body.push(Box::new(clickable(el("button",text("Register current campaign offers")).attr("class","btn"),
        |ui: &mut UiState,_| ui.request_native_pack(PackRequest::CurrentContent))));
    body.push(Box::new(el("label",(text("Explicit public pack definition JSON "),lens(
        |field: &mut TextInput| caret_text_field(field,&[]),|ui: &mut UiState| &mut ui.native.packs.definition)))));
    body.push(Box::new(clickable(el("button",text("Register dormant definition")).attr("class","btn"),
        |ui: &mut UiState,_| ui.register_pack_json())));
    if let Some(choice) = ui.selected_generator() {
        body.push(Box::new(el("div",text(format!("Generator: {} ({})",choice.name,choice.id)))));
        body.push(Box::new(clickable(el("button",text("Next generator")).attr("class","btn"),
            |ui: &mut UiState,_| ui.cycle_generator())));
        body.push(Box::new(clickable(el("button",text("Generate proposal")).attr("class","btn"),
            |ui: &mut UiState,_| ui.request_generation())));
    }
    body.push(Box::new(el("label",(text("Authored requirements JSON "),lens(
        |field: &mut TextInput| caret_text_field(field,&[]),|ui: &mut UiState| &mut ui.native.packs.requirements)))));
    body.push(Box::new(el("label",(text("Authored role slots JSON "),lens(
        |field: &mut TextInput| caret_text_field(field,&[]),|ui: &mut UiState| &mut ui.native.packs.roles)))));
    if let Some(record) = &ui.generator_preview {
        body.push(Box::new(el("div",text(format!("Generated {} · entropy {}",record.request.generator,record.entropy)))));
        if let isometry_campaign::GenValue::Campaign { campaign } = &record.proposal {
            body.push(Box::new(el("div",text(format!("{} · {} maps · {} GM-only secrets",campaign.name,campaign.maps.len(),campaign.secrets.len())))));
            body.push(Box::new(clickable(el("button",text("Register generated adventure as dormant")).attr("class","btn"),
                |ui: &mut UiState,_| ui.register_generated_pack())));
        }
    }
    if let Some(read) = &ui.native.packs.reading {
        for view in &read.packs {
            let key = &view.pack.definition.key;
            let status = if view.pack.activation.is_some() { "active" }
                else if view.readiness.compatible { "compatible, dormant" } else { "dormant, unmet requirements" };
            body.push(Box::new(el("div",text(format!("{} ({key}) · {status}",view.pack.definition.name)))));
            for why in [&view.readiness.condition_refusal,&view.readiness.native_refusal].into_iter().flatten() {
                body.push(Box::new(el("div",text(why.clone())).attr("class","side-hint")));
            }
            for (role,character) in &view.readiness.cast { body.push(Box::new(el("div",text(format!("{role}: {character}"))))); }
            if view.pack.activation.is_none() {
                for (label,force) in [("Preview acceptance",false),("Preview explicit force",true)] {
                    let request = PackRequest::Submit(PackInput::Preview { key:key.clone(),force });
                    body.push(Box::new(clickable(el("button",text(label)).attr("class","btn"),
                        move |ui: &mut UiState,_| ui.request_native_pack(request.clone()))));
                }
                if view.preview_current {
                  if let Some(preview) = view.pack.previews.last() {
                    body.push(Box::new(el("div",text(format!("Preview: {} native assertions · {}",
                        view.pack.definition.assertions.len(),serde_json::to_string(&preview.prepared.probe)
                            .unwrap_or_else(|why| format!("preview display: {why}")))))));
                    for assertion in &view.pack.definition.assertions {
                        body.push(Box::new(el("pre",text(serde_json::to_string_pretty(assertion)
                            .unwrap_or_else(|why| format!("assertion display: {why}"))))));
                    }
                    if let Some(content) = &view.pack.definition.content {
                        for map in &content.maps {
                            body.push(Box::new(el("div",text(format!("Authored board {}: {} · {}x{}",
                                map.map.id,map.map.name,map.map.width,map.map.height)))));
                        }
                        for (key,storylet) in &content.storylets {
                            body.push(Box::new(el("div",text(format!("Storylet offer {key}: {}",storylet.entry)))));
                        }
                        for reward in &content.rewards {
                            body.push(Box::new(el("div",text(format!("Reward offer: {} ({})",reward.name,reward.template)))));
                        }
                        body.push(Box::new(el("div",text("Boards and reward offers supply no body, stock, placement or control admission."))));
                    }
                    if preview.prepared.probe.native_refusal.is_none()
                        && (preview.prepared.forced || preview.prepared.probe.compatible) {
                        let intent = if preview.prepared.forced { PackInput::Force { key:key.clone(),preview:preview.witness.event } }
                            else { PackInput::Accept { key:key.clone(),preview:preview.witness.event } };
                        let label = if preview.prepared.forced { "Force previewed pack" } else { "Accept previewed pack" };
                        body.push(Box::new(clickable(el("button",text(label)).attr("class","btn"),
                            move |ui: &mut UiState,_| ui.request_native_pack(PackRequest::Submit(intent.clone())))));
                    }
                  }
                }
            }
        }
        body.push(Box::new(el("div",text(format!("{} active storylets · {} authored maps · {} reward offers",
            read.active.storylets.len(),read.active.maps.len(),read.active.rewards.len())))));
    }
    crate::widgets::overlay_panel("generator","Native table and adventure packs".into(),vec![
        Box::new(clickable(el::<_,UiState,()>("button",text("close")).attr("class","btn"),
            |ui: &mut UiState,_| ui.close_generator()))],body)
}
