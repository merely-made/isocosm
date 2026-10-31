//! Player controls for local terrain memory and the current omitted count.

use super::*;

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
            ),
        )
        .attr("class", "terrain-settings"),
    )
}
