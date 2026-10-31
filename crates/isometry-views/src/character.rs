//! The bounded first-character panel.
//!
//! It asks only for map-visible identity: name, look, and optional token
//! owner. The host chooses the loaded system and mints its default sheet.

use cambium::{TextInput, caret_text_field, clickable, el, lens, request_focus, text};

use crate::board::UiChild;
use crate::state::UiState;

/// The creation overlay, omitted until a local author opens it.
pub fn character_overlay(ui: &UiState) -> Option<UiChild> {
    if !ui.character_open {
        return None;
    }
    let name: UiChild = Box::new(
        el(
            "div",
            lens(
                |draft: &mut TextInput| request_focus(caret_text_field(draft, &[]), true),
                |ui: &mut UiState| &mut ui.character_name,
            ),
        )
        .attr("class", "character-name"),
    );
    let owner: UiChild = Box::new(
        el(
            "div",
            lens(
                |draft: &mut TextInput| caret_text_field(draft, &[]),
                |ui: &mut UiState| &mut ui.character_owner,
            ),
        )
        .attr("class", "character-owner"),
    );
    let actions: Vec<UiChild> = vec![
        Box::new(clickable(
            el("span", text("cancel")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.close_character(),
        )),
        Box::new(clickable(
            el("span", text("create")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.create_character(),
        )),
    ];
    let mut body: Vec<UiChild> = vec![
        Box::new(el("div", text("Uses the current system's default sheet. Place on the selected tile, or the next free tile.")).attr("class", "sheet-row")),
        Box::new(el("div", (text("Name "), name)).attr("class", "sheet-row character-field")),
        Box::new(el("div", (text("Owner "), owner)).attr("class", "sheet-row character-field")),
        Box::new(el("div", text("Leave owner empty for a DM token.")).attr("class", "sheet-row")),
    ];
    body.extend(crate::look::look_rows(ui));
    Some(crate::widgets::overlay_panel(
        "sheet",
        "Create character".to_owned(),
        actions,
        body,
    ))
}
