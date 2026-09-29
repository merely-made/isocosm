// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Retained SP3 terrain inspection in the existing specimen bench.
pub(super) mod render;
mod terrain;

use super::{LEAF_KEY, state::Bench, view::Child};
use cambium::{clickable, custom_leaf, el, focusable, text};
pub use terrain::Landscape;

impl Bench {
    pub fn open_spine(&mut self) {
        let seed = self.model.borrow().creator.request.seed;
        self.set_spine(seed, false, false);
    }
    fn set_spine(&mut self, seed: u64, overview: bool, perturbed: bool) {
        match terrain::draw(seed, overview, perturbed) {
            Ok(landscape) => {
                self.sim.open = false;
                self.sim.playing = false;
                self.effects.open = false;
                self.effects.playing = false;
                self.visible = true;
                let mut model = self.model.borrow_mut();
                model.pause_trial();
                model.spatial.playing = false;
                model.spine = Some(landscape);
                model.changed();
                self.notice = "World terrain ready.".into();
            },
            Err(why) => self.notice = format!("World terrain refused: {why}"),
        }
    }
    fn spine_view(&mut self, overview: bool) {
        let params = self
            .model
            .borrow()
            .spine
            .as_ref()
            .map(|s| (s.receipt.seed, s.receipt.perturbed));
        if let Some((seed, control)) = params {
            self.set_spine(seed, overview, control);
        }
    }
    fn spine_control(&mut self) {
        let params = self
            .model
            .borrow()
            .spine
            .as_ref()
            .map(|s| (s.receipt.seed, s.receipt.level > 0, !s.receipt.perturbed));
        if let Some((seed, overview, control)) = params {
            self.set_spine(seed, overview, control);
        }
    }
    fn spine_next(&mut self) {
        let params = self
            .model
            .borrow()
            .spine
            .as_ref()
            .map(|s| (s.receipt.seed.wrapping_add(1), s.receipt.level > 0));
        if let Some((seed, overview)) = params {
            self.set_spine(seed, overview, false);
        }
    }
}

fn button(label: &'static str, action: fn(&mut Bench)) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |s: &mut Bench, _| action(s),
    )))
}

pub fn view(state: &Bench) -> Child {
    let model = state.model.borrow();
    let r = &model.spine.as_ref().expect("terrain view").receipt;
    Box::new(
        el(
            "div",
            (
                el("h1", text("Specimen bench / World terrain")),
                el(
                    "p",
                    text(format!(
                        "Seed {} · {} · sites {} and {} · {} · cell {} · opaque water",
                        r.seed,
                        r.grid.shape,
                        r.sites[0],
                        r.sites[1],
                        if r.level == 0 {
                            "Border window"
                        } else {
                            "Two-site overview"
                        },
                        1u64 << r.level
                    )),
                ),
                custom_leaf::<Bench, ()>(LEAF_KEY, 640, 400)
                    .attr("id", "specimen-viewport")
                    .attr("class", "viewport")
                    .attr("role", "img")
                    .attr("aria-label", "Generated world terrain"),
                el(
                    "div",
                    vec![
                        button("Border window", |s| s.spine_view(false)),
                        button("Two-site overview", |s| s.spine_view(true)),
                        button("Perturb one side", Bench::spine_control),
                        button("Next world seed", Bench::spine_next),
                        button("Back to specimen", |s| {
                            let mut m = s.model.borrow_mut();
                            m.spine = None;
                            m.changed();
                        }),
                    ],
                )
                .attr("class", "toolbar"),
                el(
                    "p",
                    text(format!(
                        "Border {} / {} · {}",
                        r.border_digests[0],
                        r.border_digests[1],
                        if r.border_equal {
                            "equal"
                        } else {
                            "MISMATCH: perturbed-side control"
                        }
                    )),
                ),
                el(
                    "p",
                    text(state.published_error.as_deref().unwrap_or(&state.notice)),
                )
                .attr("role", "status"),
            ),
        )
        .attr("class", "bench"),
    )
}
