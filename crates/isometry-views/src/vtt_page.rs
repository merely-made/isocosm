//! Generic renderer for the VTT endpoint's disclosed projection cards.

use cambium::{TextInput, caret_text_field, clickable, el, lens, text};

use crate::board::UiChild;
use crate::state::{UiState, VttProjectionView};

pub(crate) const CSS: &str = r#"
.vtt-page { position: absolute; left: 232px; top: 36px; width: 480px; max-height: 78%; overflow-y: auto; background-color: #1b1e27; border: 1px solid #2c3347; border-radius: 4px; padding: 10px 10px 12px; box-shadow: 0 8px 28px rgba(0,0,0,0.55); }
.vtt-family .vtt-page { left: 16px; top: 16px; }
.vtt-tabs { display: flex; flex-wrap: wrap; gap: 4px; margin-bottom: 8px; }
.vtt-tab { font-size: 12px; color: #aeb8cf; background-color: #232734; padding: 4px 8px; border-radius: 3px; cursor: pointer; }
.vtt-tab-selected { color: #fff; background-color: #31527a; }
.vtt-card { padding: 8px; margin: 6px 0; border: 1px solid #343c52; border-radius: 3px; background-color: #151923; }
.vtt-card-title { font-size: 14px; font-weight: bold; color: #e8ebf2; margin-bottom: 5px; }
.vtt-value { font-size: 12px; color: #cfd3dd; margin: 2px 0; }
.vtt-badge { display: inline-block; color: #d6b36a; font-size: 11px; margin-right: 6px; }
.vtt-actions { display: flex; flex-wrap: wrap; gap: 5px; margin-top: 7px; }
.vtt-action { display: inline-block; background-color: #2c6e49; color: #eaffea; padding: 5px 9px; border-radius: 3px; cursor: pointer; font-size: 12px; }
.vtt-action-help { color: #8a90a0; font-size: 11px; margin-top: 3px; }
.vtt-status { color: #9fd48a; font-size: 12px; margin-top: 10px; min-height: 14px; }
.vtt-payload { display: flex; align-items: center; min-height: 28px; padding: 4px 8px; margin: 8px 0; background-color: #232734; border: 1px solid #343c52; }
.vtt-payload [role="textbox"] { flex: 1; color: #e8ebf2; background-color: transparent; border: 0; font-size: 12px; }
"#;

fn tab(ui: &UiState, projection: &VttProjectionView) -> UiChild {
    let id = projection.id.clone();
    let class = if id == ui.vtt_selected_projection {
        "vtt-tab vtt-tab-selected"
    } else {
        "vtt-tab"
    };
    Box::new(clickable(
        el("div", text(projection.label.clone())).attr("class", class),
        move |ui: &mut UiState, _| ui.select_vtt_projection(&id),
    ))
}

fn card(projection: &str, item: &crate::state::VttCardView) -> UiChild {
    let mut children: Vec<UiChild> = vec![Box::new(
        el("div", text(item.title.clone())).attr("class", "vtt-card-title"),
    )];
    children.extend(item.badges.iter().map(|badge| {
        Box::new(el("span", text(badge.clone())).attr("class", "vtt-badge")) as UiChild
    }));
    children.extend(item.values.iter().map(|value| {
        Box::new(
            el(
                "div",
                (
                    text(format!("{}: ", value.label)),
                    text(value.value.clone()),
                ),
            )
            .attr("class", "vtt-value"),
        ) as UiChild
    }));
    let actions: Vec<UiChild> = item
        .actions
        .iter()
        .map(|action| {
            let projection = projection.to_owned();
            let item_id = item.id.clone();
            let action_id = action.id.clone();
            Box::new(el(
                "div",
                (
                    clickable(
                        el("span", text(action.label.clone())).attr("class", "vtt-action"),
                        move |ui: &mut UiState, _| {
                            ui.request_vtt_action(&projection, &item_id, &action_id)
                        },
                    ),
                    el("div", text(action.explanation.clone())).attr("class", "vtt-action-help"),
                ),
            )) as UiChild
        })
        .collect();
    if !actions.is_empty() {
        children.push(Box::new(el("div", actions).attr("class", "vtt-actions")));
    }
    Box::new(el("div", children).attr("class", "vtt-card"))
}

pub(crate) fn overlay(ui: &UiState) -> Option<UiChild> {
    if ui.vtt_projections.is_empty() {
        return None;
    }
    let selected = ui
        .vtt_projections
        .iter()
        .find(|view| view.id == ui.vtt_selected_projection)
        .unwrap_or(&ui.vtt_projections[0]);
    let tabs: Vec<UiChild> = ui
        .vtt_projections
        .iter()
        .map(|view| tab(ui, view))
        .collect();
    let mut body = vec![Box::new(el("div", tabs).attr("class", "vtt-tabs")) as UiChild];
    let requires_input = selected
        .cards
        .iter()
        .flat_map(|item| &item.actions)
        .any(|action| action.requires_payload());
    if requires_input {
        let label = if selected.id == "vtt:authoring" {
            "Operation or assertions (JSON)"
        } else {
            "Action details (JSON)"
        };
        body.push(Box::new(
            el(
                "div",
                text(label),
            )
            .attr("class", "vtt-action-help"),
        ));
        body.push(Box::new(
            el(
                "div",
                lens(
                    |field: &mut TextInput| caret_text_field(field, &[]),
                    |ui: &mut UiState| &mut ui.vtt_action_payload,
                ),
            )
            .attr("class", "vtt-payload"),
        ));
    }
    body.extend(selected.cards.iter().map(|item| card(&selected.id, item)));
    body.push(Box::new(
        el("div", text(ui.status.clone())).attr("class", "vtt-status"),
    ));
    Some(crate::widgets::overlay_panel(
        "vtt-page",
        selected.label.clone(),
        Vec::new(),
        body,
    ))
}
