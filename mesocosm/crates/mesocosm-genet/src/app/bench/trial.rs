// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native controls and presentation for a runtime-owned disposable world.
use super::{
    state::{Bench, Specimen},
    view::Child,
};
use crate::section::{GlyphOrientation, SpatialGlyph};
use cambium::{clickable, el, focusable, text};
use isometer::GlyphAnchor;
use mesocosm_core::{PartId, World, effect_pack::Amount, history::Event};
use mesocosm_runtime::{MAX_TRIAL_STEPS, Trial, TrialActivity, TrialUptake};
mod boundary;
mod carving;
mod journey;
mod uptake;

use std::{
    cell::Cell,
    collections::BTreeSet,
    time::{Duration, Instant},
};

/// Base clearance along a face normal for a mark inscribed on it, in world
/// units, under the bench's own marker height. The spatial preview's attached
/// form uses the same base figure (`spatial/sampling.rs`).
const FACE_CLEARANCE: f32 = 0.01;

pub(super) struct WorldTrial {
    pub driver: Trial,
    carving: carving::Carving,
    journey: journey::Binding,
    boundary: boundary::Boundary,
    pub playing: bool,
    pub marks: Vec<SpatialGlyph>,
    /// The bearing part each mark asked to sit on, index-aligned with
    /// `marks`. `None` for a mark that is not an inscription on a body.
    mark_anchors: Vec<Option<PartId>>,
    /// Anchored marks the scene could not answer for, counted at the draw
    /// that asked. A cell because the anchors arrive while the model is
    /// borrowed for reading; nothing else about the trial moves there.
    anchor_fallbacks: Cell<usize>,
    pub dirty: BTreeSet<[i16; 3]>,
    pub moved: u64,
    pub fed: u64,
    pub marker_height: f32,
    pub marker_size: f32,
    pub show_marks: bool,
    show_uptake: bool,
    recent: Vec<TrialActivity>,
    uptake: Vec<TrialUptake>,
    pulses: uptake::Pulses,
    uptake_count: u64,
    uptake_mg: u64,
    marks_dropped: usize,
    marks_withheld: usize,
    uptake_pulses: usize,
    last: Instant,
}
impl WorldTrial {
    fn new(source: &World) -> Result<Self, String> {
        let mut driver = Trial::new(source)?;
        // The preset binds the journey before the first tick. Grants stay the
        // adapter's; the bench only ever reads what accepted history earned.
        let rules = journey::default_rules(source)
            .ok_or("specimen trial requires a controlled body to bind the journey")?;
        driver.enable_glyphs(rules)?;
        Ok(Self {
            driver,
            carving: carving::Carving::new(),
            journey: journey::Binding::new(),
            boundary: boundary::Boundary::new(),
            playing: false,
            marks: Vec::new(),
            mark_anchors: Vec::new(),
            anchor_fallbacks: Cell::new(0),
            dirty: BTreeSet::new(),
            moved: 0,
            fed: 0,
            marker_height: 4.0,
            marker_size: 1.4,
            show_marks: true,
            show_uptake: true,
            recent: Vec::new(),
            uptake: Vec::new(),
            pulses: uptake::Pulses::new(),
            uptake_count: 0,
            uptake_mg: 0,
            marks_dropped: 0,
            marks_withheld: 0,
            uptake_pulses: 0,
            last: Instant::now(),
        })
    }
    fn step(&mut self) -> bool {
        if !self.driver.step() {
            self.playing = false;
            return false;
        }
        self.consume_step();
        true
    }
    fn consume_step(&mut self) {
        self.boundary.note_step(&self.driver);
        for activity in self.driver.activities() {
            match activity.event {
                Event::Moved { .. } => self.moved += 1,
                Event::Fed { .. } => self.fed += 1,
                _ => {},
            }
            self.recent.push(activity.clone());
        }
        for uptake in self.driver.uptakes() {
            self.uptake_count += 1;
            self.uptake_mg += uptake.record.record.amount_mg;
            self.pulses.observe(uptake.organism, uptake.tick);
            self.uptake.push(uptake.clone());
        }
        let tick = self.driver.world().tick;
        self.carving.observe(self.driver.carves(), tick);
        self.pulses.retain(tick);
        self.uptake.retain(|a| tick.saturating_sub(a.tick) < 8);
        if self.uptake.len() > 128 {
            self.uptake.drain(..self.uptake.len() - 128);
        }
        self.recent.retain(|a| tick.saturating_sub(a.tick) < 8);
        if self.recent.len() > 128 {
            self.recent.drain(..self.recent.len() - 128);
        }
        self.refresh_marks();
        self.dirty.extend(self.driver.drain_ground_dirty());
        if self.driver.steps() >= MAX_TRIAL_STEPS || self.driver.checkpoint().is_some() {
            self.playing = false;
        }
    }
    /// Every mark now reads the bound body first and the journey second: the
    /// form comes from **what bears the glyph** — a living part expressing it,
    /// or the journey remembering it — and never from the mark's own kind. An
    /// effect the journey does not own still paints nothing and is counted
    /// instead. The bench never writes the journey, and only accepted history
    /// and flows reach the adapter.
    fn refresh_marks(&mut self) {
        self.marks_dropped = 0;
        self.marks_withheld = 0;
        self.uptake_pulses = 0;
        if !self.show_marks {
            self.marks.clear();
            self.mark_anchors.clear();
            return;
        }
        let tick = self.driver.world().tick;
        let mut marks: Vec<((u64, u8, u64), SpatialGlyph, Option<PartId>)> = Vec::new();
        for activity in &self.recent {
            // Amount stays the event's; form comes from what bears the glyph.
            let amount = match activity.event {
                Event::Moved { .. } => Amount::None,
                Event::Fed { mass_mg, .. } => Amount::MealMass(mass_mg),
                _ => continue,
            };
            match self.journey.resolve(
                &self.driver,
                activity.sequence,
                activity.from,
                Some(activity.to),
                amount,
            ) {
                Ok(request) => {
                    let life = f32::from(request.lifetime_ticks).max(1.);
                    let age = tick.saturating_sub(activity.tick) as f32 / life;
                    marks.push((
                        (activity.tick, 0u8, activity.sequence),
                        journey::glyph(&request, age, self.marker_height, self.marker_size),
                        journey::face_of(&request),
                    ));
                },
                Err(_) => self.marks_withheld += 1,
            }
        }
        // Continuous uptake refreshes one pulse per recipient, rather than
        // emitting a new particle every tick. Retained flow facts remain in
        // uptake; the pulse's age comes from the run anchor, not the newest
        // record, so unbroken flow still rises instead of standing still.
        // Uptake grants in its own right (ruling 5); its milligrams drive the
        // mark's size, and the form comes from what bears the glyph.
        if self.show_uptake {
            let mut recipients = BTreeSet::new();
            for record in self.uptake.iter().rev() {
                if !recipients.insert(record.organism) {
                    continue;
                }
                let (Some(at), Some(ticks)) = (record.at, self.pulses.age(record.organism, tick))
                else {
                    continue;
                };
                let amount = Amount::UptakeMass(record.record.record.amount_mg);
                match self
                    .journey
                    .resolve(&self.driver, record.sequence, at, None, amount)
                {
                    Ok(request) => {
                        marks.push((
                            (record.tick, 1u8, record.sequence),
                            uptake::pulse(&request, ticks, self.marker_height, self.marker_size),
                            journey::face_of(&request),
                        ));
                        self.uptake_pulses += 1;
                    },
                    Err(_) => self.marks_withheld += 1,
                }
            }
        }
        let (carved, withheld) = self.carving.marks(
            &self.journey,
            &self.driver,
            tick,
            self.marker_height,
            self.marker_size,
        );
        marks.extend(carved);
        self.marks_withheld += withheld;
        marks.sort_by_key(|(key, ..)| *key);
        self.marks_dropped = marks
            .len()
            .saturating_sub(crate::section::MAX_SPATIAL_GLYPHS);
        marks.drain(..self.marks_dropped);
        self.mark_anchors = marks.iter().map(|(_, _, part)| *part).collect();
        self.marks = marks.into_iter().map(|(_, mark, _)| mark).collect();
    }

    /// The marks as this draw should place them: an inscription borne by a
    /// part sits on **that part's own meshed face**, taking its centre and
    /// axes from the scene's anchor for it. A mark the scene cannot answer
    /// for keeps the placement the bench has always used and is counted.
    ///
    /// The anchors arrive from the draw rather than from the tick because
    /// only the scene knows where a posed part's face is; the reading that
    /// chose the part is still the body's, and nothing here can change it.
    pub fn anchored_marks(&self, anchors: &[GlyphAnchor]) -> Vec<SpatialGlyph> {
        let mut fallbacks = 0;
        let placed = self
            .marks
            .iter()
            .zip(&self.mark_anchors)
            .map(|(mark, part)| {
                let Some(part) = part else {
                    return mark.clone();
                };
                let Some(anchor) = anchors.iter().find(|a| a.part == *part) else {
                    fallbacks += 1;
                    return mark.clone();
                };
                // Cleared off the face along its own normal, then raised by
                // the bench's marker height exactly as every trial mark has
                // always been raised. A mark left in the face plane is the
                // depth renderer's to occlude and it occludes it; the raise
                // is display, and the face is what the mark now belongs to.
                let mut centre =
                    [0, 1, 2].map(|i| anchor.centre[i] + anchor.normal[i] * FACE_CLEARANCE);
                centre[1] += self.marker_height;
                SpatialGlyph {
                    centre,
                    orientation: GlyphOrientation::WorldPlane {
                        right: anchor.right,
                        up: anchor.up,
                    },
                    ..mark.clone()
                }
            })
            .collect();
        self.anchor_fallbacks.set(fallbacks);
        placed
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
    /// Arms an advance-to-boundary run at the trial's current epoch (D5).
    fn start_advance_to_boundary(&mut self) {
        self.boundary.start(&self.driver);
    }
    /// One frame's worth of an advance-to-boundary run: up to
    /// `boundary::STEP_BUDGET` ordinary idle steps, applied directly rather
    /// than gated by Play's wall-clock rate, so a 1,000-tick epoch finishes
    /// in a handful of frames. `Boundary::observe` is consulted after every
    /// attempted step, successful or refused, so the run stops the instant
    /// its epoch has risen or a step refuses, exactly as it would mid-chunk.
    fn advance_boundary_chunk(&mut self) -> bool {
        if !self.boundary.advancing() {
            return false;
        }
        let mut changed = false;
        for _ in 0..boundary::STEP_BUDGET {
            changed |= self.step();
            self.boundary.observe(&self.driver);
            if !self.boundary.advancing() {
                break;
            }
        }
        changed
    }
    fn reset(&mut self) {
        self.driver.reset();
        self.carving.reset();
        self.boundary.reset(&self.driver);
        self.playing = false;
        self.marks.clear();
        self.mark_anchors.clear();
        self.anchor_fallbacks.set(0);
        self.dirty.clear();
        self.moved = 0;
        self.fed = 0;
        self.recent.clear();
        self.uptake.clear();
        self.pulses.reset();
        self.uptake_count = 0;
        self.uptake_mg = 0;
        self.marks_dropped = 0;
        self.marks_withheld = 0;
        self.uptake_pulses = 0;
        self.last = Instant::now();
    }
    pub fn probe_fields(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![
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
            ("trial-uptake-count", self.uptake_count.to_string()),
            ("trial-uptake-mg", self.uptake_mg.to_string()),
            // Counted as the pulses are built. The stroke is the pack's now,
            // and it is the same shape the feeding mark uses.
            ("trial-uptake-pulses", self.uptake_pulses.to_string()),
            (
                "trial-uptake",
                serde_json::to_string(&self.uptake).expect("uptake serializes"),
            ),
            ("trial-mark-budget-dropped", self.marks_dropped.to_string()),
            // Marks that asked for a bearing part's face and did not get one,
            // counted at the last draw. Nonzero means the scene answered with
            // no anchor for that part and the mark kept its old placement.
            (
                "trial-anchor-fallbacks",
                self.anchor_fallbacks.get().to_string(),
            ),
            (
                "trial-journey",
                serde_json::to_string(&self.journey.reading(&self.driver))
                    .expect("journey reading serializes"),
            ),
            (
                "trial-grants",
                self.journey.grants(&self.driver).to_string(),
            ),
            // What bears the effect's glyph, and so which rule every mark of
            // it resolves through: a living part expressing it, the journey
            // remembering it, or nothing.
            (
                "trial-borne-by",
                journey::borne_label(self.journey.borne_by(&self.driver)).into(),
            ),
            // The glyphs the bound body embodies right now, and the living
            // parts bearing the preset's base glyph. Both are pure readings
            // over the phenotype; neither is a grant and neither is stored.
            (
                "trial-embodied",
                serde_json::to_string(&self.journey.embodied(&self.driver))
                    .expect("embodied glyphs serialize"),
            ),
            (
                "trial-expressing-parts",
                serde_json::to_string(
                    &self
                        .journey
                        .expressing_parts(&self.driver)
                        .iter()
                        .map(|part| part.0)
                        .collect::<Vec<_>>(),
                )
                .expect("expressing parts serialize"),
            ),
            ("trial-marks-withheld", self.marks_withheld.to_string()),
            ("trial-visible-marks", self.marks.len().to_string()),
            ("trial-marker-height", self.marker_height.to_string()),
            ("trial-marker-size", self.marker_size.to_string()),
            ("trial-show-marks", self.show_marks.to_string()),
            ("trial-show-uptake", self.show_uptake.to_string()),
            (
                "trial-checkpoint",
                self.driver.checkpoint().is_some().to_string(),
            ),
            (
                "trial-activity",
                serde_json::to_string(&self.recent).expect("activity serializes"),
            ),
        ];
        fields.extend(self.carving.probe_fields());
        fields.extend(self.boundary.probe_fields(&self.driver));
        fields.push((
            "trial-trace",
            serde_json::to_string(self.driver.trace()).expect("trace serializes"),
        ));
        fields
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
    pub fn trial_advancing(&self) -> bool {
        self.trial.as_ref().is_some_and(|t| t.boundary.advancing())
    }
    pub fn advance_trial_boundary(&mut self) {
        if self
            .trial
            .as_mut()
            .is_some_and(WorldTrial::advance_boundary_chunk)
        {
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
            // An advance-to-boundary run already owns the stepping this frame.
            if t.boundary.advancing() {
                return;
            }
            t.playing = false;
            if t.step() {
                m.selected = None;
                m.changed();
            }
        }
    }
    fn play_trial(&mut self) {
        if let Some(t) = &mut self.model.borrow_mut().trial {
            if !t.boundary.advancing()
                && t.driver.steps() < MAX_TRIAL_STEPS
                && t.driver.checkpoint().is_none()
            {
                t.playing = true;
                t.last = Instant::now();
            }
        }
    }
    fn advance_to_boundary(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.playing = false;
            t.start_advance_to_boundary();
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
    fn toggle_uptake_marks(&mut self) {
        let mut m = self.model.borrow_mut();
        if let Some(t) = &mut m.trial {
            t.show_uptake = !t.show_uptake;
            t.refresh_marks();
            m.changed();
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
    } else if trial.boundary.advancing() {
        "Advancing to boundary"
    } else if trial.playing {
        "Playing"
    } else {
        "Paused"
    };
    Box::new(el("section",(
        el("div",vec![button("Step world",Bench::step_trial),button("Play world",Bench::play_trial),button("Advance to boundary",Bench::advance_to_boundary),button("Pause world",|s|s.model.borrow_mut().pause_trial()),button(if trial.show_marks { "Hide activity" } else { "Show activity" },Bench::toggle_trial_marks),button(if trial.show_uptake { "Hide uptake" } else { "Show uptake" },Bench::toggle_uptake_marks),button("Mark height",Bench::marker_height),button("Mark size",Bench::marker_size),button("Reset world",Bench::reset_trial),button("Exit world trial",Bench::exit_trial)]).attr("class","toolbar"),
        el("p",text(format!("{status} / {} of {MAX_TRIAL_STEPS} ticks / {} movements, {} meals, {} uptake transfers / {} activity marks / height {} / size {}",trial.driver.steps(),trial.moved,trial.fed,trial.uptake_count,trial.marks.len(),trial.marker_height,trial.marker_size))),
        el("p",text("Recorded movement, feeding and soil uptake, with adjustable organism-level markers. The trial leaves saved generation unchanged.")),
        carving::view(state),
    )).attr("class","world-trial"))
}
