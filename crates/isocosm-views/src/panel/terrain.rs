//! Player controls for local scene quality and terrain memory.

use super::*;
use crate::scene::BoardBodyQuality;

pub(super) fn controls(ui: &UiState) -> UiChild {
    if !ui.scene_board {
        return Box::new(el("div", ()));
    }
    let settings = &ui.terrain_settings;
    let effective = settings.device_budget.map_or(settings.budget_mib, |_| {
        settings.budget_mib.min(settings.max_mib())
    });
    Box::new(
        el(
            "div",
            (
                el("div", text(format!("Terrain memory: {effective} MiB")))
                    .attr("class", "side-line"),
                el(
                    "div",
                    vec![
                        map_button(
                            "−",
                            settings.device_budget.is_some() && effective > 1,
                            |ui| ui.terrain_settings.step(-1),
                        ),
                        map_button(
                            "+",
                            settings.device_budget.is_some() && effective < settings.max_mib(),
                            |ui| ui.terrain_settings.step(1),
                        ),
                    ],
                )
                .attr("class", "btn-row"),
                el(
                    "div",
                    text(format!("{} terrain bricks omitted", settings.omitted)),
                )
                .attr("class", "side-line"),
                el("div", text("Token detail")).attr("class", "side-line"),
                el(
                    "div",
                    vec![
                        map_button(
                            "Live",
                            ui.scene_body_quality != BoardBodyQuality::Live,
                            |ui| {
                                ui.scene_body_quality = BoardBodyQuality::Live;
                            },
                        ),
                        map_button(
                            "Hybrid",
                            ui.scene_body_quality != BoardBodyQuality::Hybrid,
                            |ui| {
                                ui.scene_body_quality = BoardBodyQuality::Hybrid;
                            },
                        ),
                    ],
                )
                .attr("class", "btn-row"),
            ),
        )
        .attr("class", "terrain-settings"),
    )
}
