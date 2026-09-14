// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation-only workloads for the integrated document cost receipt.
#[cfg(test)]
mod completion;
pub(super) mod model;
#[cfg(test)]
mod oracle;
pub(super) mod renderer;

use super::{LEAF_KEY, state::Bench, view::Child};
use cambium::{clickable, custom_leaf, el, focusable, text};

pub fn load(path: &std::path::Path) -> Result<model::Workload, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let config: model::Config = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    model::Workload::new(config)
}
fn button(label: &'static str, action: fn(&mut Bench)) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |s: &mut Bench, _| action(s),
    )))
}
pub fn view(state: &Bench) -> Child {
    let model = state.model.borrow();
    let population = model.population.as_ref().expect("population view");
    Box::new(
        el(
            "div",
            (
                el("h1", text("Specimen bench / Population")),
                el(
                    "p",
                    text(format!(
                        "{} bodies / {} designs / {} / seed {}",
                        population.config.bodies,
                        population.config.designs,
                        population.config.kind.label(),
                        population.config.seed
                    )),
                ),
                el(
                    "p",
                    text(format!(
                        "Spacing {} / view half-height {} / {} depth layers / {} submission",
                        population.config.grid_spacing,
                        population.config.camera_extent,
                        population.config.depth_layers,
                        if population.config.reverse_instances {
                            "reverse"
                        } else {
                            "forward"
                        }
                    )),
                ),
                custom_leaf::<Bench, ()>(LEAF_KEY, 640, 400)
                    .attr("id", "specimen-viewport")
                    .attr(
                        "class",
                        match state.tint {
                            1 => "viewport warm",
                            2 => "viewport cool",
                            _ => "viewport",
                        },
                    )
                    .attr("role", "img")
                    .attr("aria-label", "Specimen population"),
                el(
                    "div",
                    vec![
                        button("Turn left", |s| s.turn(-0.2617994)),
                        button("Turn right", |s| s.turn(0.2617994)),
                        button("Reset pose", |s| {
                            let mut m = s.model.borrow_mut();
                            m.yaw = 0.;
                            m.changed();
                        }),
                        button("Natural", |s| s.tint = 0),
                        button("Warm", |s| s.tint = 1),
                        button("Cool", |s| s.tint = 2),
                    ],
                )
                .attr("class", "toolbar"),
                el(
                    "p",
                    text(
                        state
                            .published_error
                            .clone()
                            .unwrap_or_else(|| "Ready for population inspection.".into()),
                    ),
                )
                .attr("role", "status"),
            ),
        )
        .attr("class", "bench population"),
    )
}
