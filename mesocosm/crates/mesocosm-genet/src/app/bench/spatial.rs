// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{state::Bench, view::Child};
use crate::section::{CameraMode, SpatialGlyph};
use cambium::{clickable, el, focusable, text};
use mesocosm_core::effect_experiment::Glyph;
use std::time::Instant;

pub(super) struct Spatial {
    pub enabled: bool,
    pub playing: bool,
    pub tick: u32,
    pub glyph: Glyph,
    last: Instant,
}
impl Default for Spatial {
    fn default() -> Self {
        Self {
            enabled: false,
            playing: false,
            tick: 0,
            glyph: Glyph::Quotes,
            last: Instant::now(),
        }
    }
}
impl Spatial {
    pub fn advance(&mut self) -> bool {
        let ticks = (self.last.elapsed().as_millis() / 33).min(120) as u32;
        if ticks == 0 {
            return false;
        }
        self.last += std::time::Duration::from_millis(u64::from(ticks) * 33);
        self.tick = (self.tick + ticks.min(120)) % 360;
        true
    }
    pub fn marks(&self, bounds: ([f32; 3], [f32; 3])) -> Vec<SpatialGlyph> {
        if !self.enabled {
            return Vec::new();
        }
        let (min, max) = bounds;
        let centre = [0, 1, 2].map(|i| (min[i] + max[i]) * 0.5);
        let radius = ((max[0] - min[0]).max(max[2] - min[2]) * 0.6).max(2.0);
        let height = (max[1] - min[1]).max(2.0);
        (0..18)
            .map(|id| {
                let phase = (id as f32 * 20.0 + self.tick as f32).to_radians();
                SpatialGlyph {
                    centre: [
                        centre[0] + radius * phase.cos(),
                        centre[1] + height * 0.22 * (phase * 2.0).sin(),
                        centre[2] + radius * phase.sin(),
                    ],
                    size: height * 0.18,
                    angle: phase * 0.25,
                    glyph: self.glyph,
                    color: if id % 2 == 0 {
                        [0.35, 0.95, 0.72, 1.0]
                    } else {
                        [1.0, 0.76, 0.3, 1.0]
                    },
                }
            })
            .collect()
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
        children.extend([
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
    Box::new(el("div", children).attr("class", "toolbar appearance"))
}
