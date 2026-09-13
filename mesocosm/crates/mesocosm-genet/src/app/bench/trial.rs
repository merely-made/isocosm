// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native controls and presentation for a runtime-owned disposable world.
use super::{
    state::{Bench, Specimen},
    view::Child,
};
use crate::section::{GlyphOrientation, SpatialGlyph};
use cambium::{clickable, el, focusable, text};
use mesocosm_core::{World, effect_experiment::Glyph, history::Event};
use mesocosm_runtime::{MAX_TRIAL_STEPS, Trial, TrialActivity};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};

pub(super) struct WorldTrial {
    pub driver: Trial,
    pub playing: bool,
    pub marks: Vec<SpatialGlyph>,
    pub dirty: BTreeSet<[i16; 3]>,
    pub moved: u64,
    pub fed: u64,
    pub marker_height: f32,
    pub marker_size: f32,
    pub show_marks: bool,
    recent: Vec<TrialActivity>,
    last: Instant,
}
impl WorldTrial {
    fn new(source: &World) -> Result<Self, String> {
        Ok(Self {
            driver: Trial::new(source)?,
            playing: false,
            marks: Vec::new(),
            dirty: BTreeSet::new(),
            moved: 0,
            fed: 0,
            marker_height: 4.0,
            marker_size: 1.4,
            show_marks: true,
            recent: Vec::new(),
            last: Instant::now(),
        })
    }
    fn step(&mut self) -> bool {
        if !self.driver.step() {
            self.playing = false;
            return false;
        }
        for activity in self.driver.activities() {
            match activity.event {
                Event::Moved { .. } => self.moved += 1,
                Event::Fed { .. } => self.fed += 1,
                _ => {},
            }
            self.recent.push(activity.clone());
        }
        let tick = self.driver.world().tick;
        self.recent.retain(|a| tick.saturating_sub(a.tick) < 8);
        if self.recent.len() > 128 {
            self.recent.drain(..self.recent.len() - 128);
        }
        self.refresh_marks();
        self.dirty.extend(self.driver.drain_ground_dirty());
        if self.driver.steps() >= MAX_TRIAL_STEPS || self.driver.checkpoint().is_some() {
            self.playing = false;
        }
        true
    }
    fn refresh_marks(&mut self) {
        if !self.show_marks {
            self.marks.clear();
            return;
        }
        let tick = self.driver.world().tick;
        self.marks = self
            .recent
            .iter()
            .map(|a| {
                let age = tick.saturating_sub(a.tick) as f32 / 8.;
                let mut centre =
                    [0, 1, 2].map(|i| a.from[i] as f32 * (1. - age) + a.to[i] as f32 * age);
                centre[1] += self.marker_height;
                let glyph = if matches!(a.event, Event::Moved { .. }) {
                    Glyph::Slashes
                } else {
                    Glyph::Quotes
                };
                SpatialGlyph {
                    centre,
                    size: self.marker_size * (1. - age * 0.5),
                    angle: age * 0.6,
                    glyph,
                    orientation: GlyphOrientation::CameraFacing,
                    color: if glyph == Glyph::Slashes {
                        [0.45, 0.95, 0.8, 1.]
                    } else {
                        [1., 0.75, 0.3, 1.]
                    },
                }
            })
            .collect();
    }
    fn advance(&mut self) -> bool {
        if !self.playing {
            return false;
        }
        let count = (self.last.elapsed().as_millis() / 100).min(8) as u32;
        self.last += Duration::from_millis(u64::from(count) * 100);
        let mut changed = false;
        for _ in 0..count {
            if !self.step() {
                break;
            }
            changed = true;
        }
        changed
    }
    fn reset(&mut self) {
        self.driver.reset();
        self.playing = false;
        self.marks.clear();
        self.dirty.clear();
        self.moved = 0;
        self.fed = 0;
        self.recent.clear();
        self.last = Instant::now();
    }
    pub fn probe_fields(&self) -> Vec<(&'static str, String)> {
        vec![
            ("trial-active", "true".into()),
            ("trial-playing", self.playing.to_string()),
            ("trial-steps", self.driver.steps().to_string()),
            ("trial-tick", self.driver.world().tick.to_string()),
            (
                "trial-source-hash",
                format!("{:016x}", self.driver.baseline_hash()),
            ),
            ("trial-hash", format!("{:016x}", self.driver.state_hash())),
            ("trial-event-count", (self.moved + self.fed).to_string()),
            ("trial-total-moved", self.moved.to_string()),
            ("trial-total-fed", self.fed.to_string()),
            ("trial-visible-marks", self.marks.len().to_string()),
            ("trial-marker-height", self.marker_height.to_string()),
            ("trial-marker-size", self.marker_size.to_string()),
            ("trial-show-marks", self.show_marks.to_string()),
            (
                "trial-checkpoint",
                self.driver.checkpoint().is_some().to_string(),
            ),
            (
                "trial-activity",
                serde_json::to_string(&self.recent).expect("activity serializes"),
            ),
        ]
    }
}
impl Specimen {
    fn trial_replaced(&mut self) {
        self.selected = None;
        self.epoch = self.epoch.checked_add(1).expect("bench epoch exhausted");
        self.changed();
    }
    pub fn trial_playing(&self) -> bool {
        self.trial.as_ref().is_some_and(|t| t.playing)
    }
    pub fn pause_trial(&mut self) {
        if let Some(trial) = &mut self.trial {
            trial.playing = false;
        }
    }
    pub fn advance_trial(&mut self) {
        if self.trial.as_mut().is_some_and(WorldTrial::advance) {
            self.selected = None;
            self.changed();
        }
    }
}
impl Bench {
    pub fn start_trial(&mut self) {
        let mut model = self.model.borrow_mut();
        if model.creator.pending {
            self.notice = "Wait for generation before starting a world trial.".into();
            return;
        }
        match WorldTrial::new(model.source_world()) {
            Ok(trial) => {
                model.trial = Some(trial);
                model.spatial.playing = false;
                model.trial_replaced();
                self.generation.open = false;
                self.notice = "Disposable world trial. Saved generation remains unchanged.".into();
            },
            Err(why) => self.notice = why,
        }
    }
    fn step_trial(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.playing = false;
            if t.step() {
                m.selected = None;
                m.changed();
            }
        }
    }
    fn play_trial(&mut self) {
        if let Some(t) = &mut self.model.borrow_mut().trial {
            if t.driver.steps() < MAX_TRIAL_STEPS && t.driver.checkpoint().is_none() {
                t.playing = true;
                t.last = Instant::now();
            }
        }
    }
    fn marker_height(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.marker_height = if t.marker_height >= 8.0 {
                0.0
            } else {
                t.marker_height + 2.0
            };
            t.refresh_marks();
            m.changed();
        }
    }
    fn marker_size(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.marker_size = if t.marker_size >= 2.8 {
                0.7
            } else {
                t.marker_size * 2.0
            };
            t.refresh_marks();
            m.changed();
        }
    }
    fn reset_trial(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.reset();
            m.trial_replaced();
        }
    }
    fn toggle_trial_marks(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.show_marks = !t.show_marks;
            t.refresh_marks();
            m.changed();
        }
    }
    fn exit_trial(&mut self) {
        let mut m = self.model.borrow_mut();
        m.trial = None;
        m.trial_replaced();
        self.notice = "Returned to the unchanged generation.".into();
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
    let Some(trial) = &model.trial else {
        return Box::new(el("div", ()));
    };

    let status = if trial.driver.checkpoint().is_some() {
        "Stopped at a world checkpoint"
    } else if trial.driver.steps() >= MAX_TRIAL_STEPS {
        "Trial limit reached"
    } else if trial.playing {
        "Playing"
    } else {
        "Paused"
    };
    Box::new(el("section",(
        el("div",vec![button("Step world",Bench::step_trial),button("Play world",Bench::play_trial),button("Pause world",|s|s.model.borrow_mut().pause_trial()),button(if trial.show_marks { "Hide activity" } else { "Show activity" },Bench::toggle_trial_marks),button("Mark height",Bench::marker_height),button("Mark size",Bench::marker_size),button("Reset world",Bench::reset_trial),button("Exit world trial",Bench::exit_trial)]).attr("class","toolbar"),
        el("p",text(format!("{status} / {} of {MAX_TRIAL_STEPS} ticks / {} movements, {} meals / {} activity marks / height {} / size {}",trial.driver.steps(),trial.moved,trial.fed,trial.marks.len(),trial.marker_height,trial.marker_size))),
        el("p",text("Recorded movement and feeding, with adjustable raised markers. The trial leaves saved generation unchanged.")),
    )).attr("class","world-trial"))
}
