// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use cambium::{TextInput, caret_text_field, clickable, el, lens, text};
use crate::{UiState, board::UiChild};
use isocosm_vtt::native::storylets::ComposedStoryletInput;
use isometry_core::TokenId;

#[derive(Clone, Copy, Debug)]
pub enum StoryletPhysicalFileAction { Quote, Apply }
#[derive(Clone, Debug)]
pub enum StoryletPhysicalRequest {
    Quote { input: ComposedStoryletInput, item_owner: Option<TokenId> },
    Apply { input: ComposedStoryletInput, item_owner: Option<TokenId> },
}
#[derive(Clone, Debug)]
pub enum StoryletPhysicalOutcome {
    Quote { input: ComposedStoryletInput, result: isonetry::NativeComposedStoryletResolved },
    Applied,
}

impl UiState {
    pub fn request_storylet_physical_file(&mut self, action: StoryletPhysicalFileAction) {
        if !self.can_edit_inventory || self.native.world.is_none() || self.native.table.is_none() {
            self.status = "physical storylet authoring requires the actual host and admitted table".into(); return;
        }
        self.native.storylet_physical_quote = None;
        self.native.storylet_physical_file_request = Some(action);
    }
}

pub(crate) fn controls(ui: &UiState) -> Vec<UiChild> {
    if !ui.can_edit_inventory || ui.native.world.is_none() { return Vec::new(); }
    let mut rows: Vec<UiChild> = vec![Box::new(el("h4", text("Physical storylet inputs")))];
    rows.push(Box::new(el("div", (text("Complete storylet JSON file "), lens(
        |field: &mut TextInput| caret_text_field(field, &[]),
        |ui: &mut UiState| &mut ui.native.storylet_physical_path,
    ))).attr("class", "sheet-row")));
    rows.push(Box::new(el("div", (
        clickable(el("span", text("Quote supplied input")).attr("class", "btn"),
            |ui: &mut UiState, _| ui.request_storylet_physical_file(StoryletPhysicalFileAction::Quote)),
        clickable(el("span", text("Apply supplied input")).attr("class", "btn"),
            |ui: &mut UiState, _| ui.request_storylet_physical_file(StoryletPhysicalFileAction::Apply)),
    )).attr("class", "btn-row")));
    if let Some((input, result)) = &ui.native.storylet_physical_quote {
        rows.push(Box::new(el("p", text(format!("{}: {} supplied operation steps", input.storylet.key, input.steps.len())))));
        for effect in &result.public.effects {
            rows.push(Box::new(el("p", text(serde_json::to_string(effect).unwrap_or_else(|error| error.to_string())))));
        }
    }
    rows
}
