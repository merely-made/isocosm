// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{state::Bench, view::Child};
use crate::section::CameraMode;
use cambium::{clickable, el, focusable, text};
use mesocosm_core::effect_experiment::Glyph;
use serde::{Deserialize, Serialize};
use std::time::Instant;
mod sampling;
mod saved;
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Form {
    Orbit,
    Surface,
    Tether,
    Emission,
}
impl Form {
    pub const ALL: [Self; 4] = [Self::Orbit, Self::Surface, Self::Tether, Self::Emission];
    pub fn label(self) -> &'static str {
        match self {
            Self::Orbit => "Orbit",
            Self::Surface => "Surface",
            Self::Tether => "Tether",
            Self::Emission => "Emission",
        }
    }
}

pub(super) struct Spatial {
    pub enabled: bool,
    pub playing: bool,
    pub tick: u32,
    pub glyph: Glyph,
    pub form: Form,
    pub seed: u64,
    pub count: u16,
    pub saved: Option<std::path::PathBuf>,
    last: Instant,
}
impl Default for Spatial {
    fn default() -> Self {
        Self {
            enabled: false,
            playing: false,
            tick: 0,
            glyph: Glyph::Quotes,
            form: Form::Orbit,
            seed: 7,
            count: 18,
            saved: None,
            last: Instant::now(),
        }
    }
}
impl Spatial {
    pub fn framing_bounds(
        &self,
        (mut min, mut max): ([f32; 3], [f32; 3]),
        anchors: &[crate::section::GlyphAnchor],
    ) -> ([f32; 3], [f32; 3]) {
        let height = (max[1] - min[1]).max(2.0);
        match self.form {
            Form::Orbit => {
                let radius =
                    ((max[0] - min[0]).max(max[2] - min[2]) * 0.6).max(2.0) + height * 0.18;
                for i in [0, 2] {
                    let c = (min[i] + max[i]) * 0.5;
                    min[i] = min[i].min(c - radius);
                    max[i] = max[i].max(c + radius);
                }
            },
            Form::Surface => {
                for i in 0..3 {
                    min[i] -= 0.02;
                    max[i] += 0.02;
                }
            },
            Form::Tether => {
                max[1] += height * 0.9;
                for i in 0..3 {
                    min[i] -= height * 0.1;
                    max[i] += height * 0.1;
                }
            },
            Form::Emission => {
                for anchor in anchors {
                    for i in 0..3 {
                        let end = anchor.centre[i] + anchor.normal[i] * (0.02 + height * 0.65);
                        let padding = anchor.right[i].abs() * height * 0.05 + height * 0.12;
                        min[i] = min[i]
                            .min(end - padding)
                            .min(anchor.centre[i] - height * 0.12);
                        max[i] = max[i]
                            .max(end + padding)
                            .max(anchor.centre[i] + height * 0.12);
                    }
                }
            },
        }
        (min, max)
    }

    pub fn advance(&mut self) -> bool {
        let ticks = (self.last.elapsed().as_millis() / 33).min(120) as u32;
        if ticks == 0 {
            return false;
        }
        self.last += std::time::Duration::from_millis(u64::from(ticks) * 33);
        self.tick = (self.tick + ticks.min(120)) % 360;
        true
    }
}
fn button(label: &'static str, action: fn(&mut Bench)) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |s: &mut Bench, _| action(s),
    )))
}
pub(super) fn view(state: &Bench) -> Child {
    let model = state.model.borrow();
    let mut children = vec![button("Spatial glyphs", |s| {
        let mut m = s.model.borrow_mut();
        m.spatial.enabled = !m.spatial.enabled;
        m.spatial.playing = false;
        m.changed();
    })];
    if model.spatial.enabled {
        for form in Form::ALL {
            children.push(Box::new(focusable(clickable(
                el("button", text(form.label()))
                    .attr("aria-label", form.label())
                    .attr("aria-pressed", (model.spatial.form == form).to_string()),
                move |s: &mut Bench, _| {
                    let mut m = s.model.borrow_mut();
                    m.spatial.form = form;
                    m.spatial.tick = 0;
                    m.spatial.playing = false;
                    m.changed();
                },
            ))));
        }
        children.extend([
            button("Reseed spatial", |s| {
                let mut m = s.model.borrow_mut();
                m.spatial.seed = m.spatial.seed.wrapping_add(1);
                m.changed();
            }),
            button("Glyph count", |s| {
                let mut m = s.model.borrow_mut();
                m.spatial.count = match m.spatial.count {
                    6 => 18,
                    18 => 64,
                    64 => 128,
                    _ => 6,
                };
                m.changed();
            }),
            button("Save spatial", Bench::save_spatial),
            button("Reopen spatial", Bench::reopen_spatial),
            button("Step orbit", |s| {
                let mut m = s.model.borrow_mut();
                m.spatial.tick = (m.spatial.tick + 15) % 360;
                m.changed();
            }),
            button(
                if model.spatial.playing {
                    "Pause orbit"
                } else {
                    "Play orbit"
                },
                |s| {
                    let mut m = s.model.borrow_mut();
                    m.spatial.playing = !m.spatial.playing;
                    m.spatial.last = Instant::now();
                },
            ),
            button("Reset orbit", |s| {
                let mut m = s.model.borrow_mut();
                m.spatial.tick = 0;
                m.spatial.playing = false;
                m.changed();
            }),
            button("Change glyph", |s| {
                let mut m = s.model.borrow_mut();
                m.spatial.glyph = match m.spatial.glyph {
                    Glyph::Quotes => Glyph::Slashes,
                    Glyph::Slashes => Glyph::Backticks,
                    Glyph::Backticks => Glyph::Quotes,
                };
                m.changed();
            }),
            button("View angle", |s| {
                let mut m = s.model.borrow_mut();
                m.camera = if m.camera == CameraMode::Oblique {
                    CameraMode::Across
                } else {
                    CameraMode::Oblique
                };
                m.epoch += 1;
                m.changed();
            }),
        ]);
    }
    if model.spatial.enabled && model.spatial.form == Form::Tether && model.selected.is_some() {
        children.push(Box::new(el(
            "span",
            text("Tethers need two parts. Clear selection to use body anchors."),
        )));
    }
    if model.spatial.enabled {
        children.push(Box::new(el(
            "span",
            text(format!(
                "{} / {} marks / seed {} / {}",
                model.spatial.form.label(),
                model.spatial.count,
                model.spatial.seed,
                if model.selected.is_some() {
                    "selected part"
                } else {
                    "body parts"
                }
            )),
        )));
    }
    Box::new(el("div", children).attr("class", "toolbar appearance"))
}
