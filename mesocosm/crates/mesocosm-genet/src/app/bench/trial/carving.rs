// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Controls issue a runtime-owned carve. Marks indicate recorded locations;
//! their height is presentation, not a contact normal or debris simulation.
use super::{Bench, Child};
use crate::section::{GlyphOrientation, SpatialGlyph, Stroke};
use cambium::{clickable, el, focusable, text};
use mesocosm_core::World;
use mesocosm_runtime::TrialCarve;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct ActionReceipt {
    target: [i32; 3],
    radius: i32,
    reach: i32,
    tick_processed: bool,
    before_ground_revision: u64,
    after_ground_revision: u64,
    before_solid_voxels: usize,
    after_solid_voxels: usize,
    outcomes: String,
}

pub(super) struct Carving {
    offset: [i32; 3],
    radius: i32,
    show: bool,
    recent: Vec<TrialCarve>,
    last_observed: Option<(u64, u64)>,
    events: u64,
    removed: u64,
    action: Option<ActionReceipt>,
    feedback: String,
}

impl Carving {
    pub(super) fn new() -> Self {
        Self {
            offset: [0, -1, 0],
            radius: 1,
            show: true,
            recent: Vec::new(),
            last_observed: None,
            events: 0,
            removed: 0,
            action: None,
            feedback: "Target is relative to the controlled critter.".into(),
        }
    }

    pub(super) fn reset(&mut self) {
        self.recent.clear();
        self.last_observed = None;
        self.events = 0;
        self.removed = 0;
        self.action = None;
        self.feedback = "Carving history reset; target settings retained.".into();
    }

    pub(super) fn observe(&mut self, events: &[TrialCarve], tick: u64) {
        for event in events {
            let key = (event.tick, event.sequence);
            if self.last_observed.is_some_and(|last| key <= last) {
                continue;
            }
            self.last_observed = Some(key);
            if event.removed == 0 {
                continue;
            }
            self.events += 1;
            self.removed += u64::from(event.removed);
            self.recent.push(event.clone());
        }
        self.recent.retain(|e| tick >= e.tick && tick - e.tick < 8);
        if self.recent.len() > 128 {
            self.recent.drain(..self.recent.len() - 128);
        }
    }

    pub(super) fn marks(
        &self,
        tick: u64,
        height: f32,
        size: f32,
    ) -> Vec<((u64, u8, u64), SpatialGlyph)> {
        if !self.show {
            return Vec::new();
        }
        self.recent
            .iter()
            .filter(|e| tick >= e.tick && tick - e.tick < 8)
            .map(|e| {
                let age = (tick - e.tick) as f32 / 8.;
                let mut centre = e.at.map(|v| v as f32);
                centre[1] += height;
                (
                    (e.tick, 2, e.sequence),
                    SpatialGlyph {
                        centre,
                        size: size * (1. - age * 0.5),
                        angle: 0.,
                        glyph: Stroke::Slashes,
                        orientation: GlyphOrientation::CameraFacing,
                        color: [1., 0.45, 0.2, 1.],
                    },
                )
            })
            .collect()
    }

    pub(super) fn probe_fields(&self) -> Vec<(&'static str, String)> {
        let mut fields = vec![
            ("trial-carving-offset", format!("{:?}", self.offset)),
            ("trial-carving-radius", self.radius.to_string()),
            ("trial-carving-shown", self.show.to_string()),
            ("trial-carving-events", self.events.to_string()),
            ("trial-carving-removed", self.removed.to_string()),
            (
                "trial-carving-recent",
                serde_json::to_string(&self.recent).expect("carves serialize"),
            ),
            (
                "trial-carving-action",
                serde_json::to_string(&self.action).expect("action serializes"),
            ),
            ("trial-carving-feedback", self.feedback.clone()),
        ];
        if let Some(action) = &self.action {
            fields.extend([
                ("trial-carving-target", format!("{:?}", action.target)),
                ("trial-carving-reach", action.reach.to_string()),
                (
                    "trial-carving-tick-processed",
                    action.tick_processed.to_string(),
                ),
                (
                    "trial-carving-before-revision",
                    action.before_ground_revision.to_string(),
                ),
                (
                    "trial-carving-after-revision",
                    action.after_ground_revision.to_string(),
                ),
                (
                    "trial-carving-before-solid",
                    action.before_solid_voxels.to_string(),
                ),
                (
                    "trial-carving-after-solid",
                    action.after_solid_voxels.to_string(),
                ),
                ("trial-carving-outcomes", action.outcomes.clone()),
            ]);
        }
        fields
    }
}

fn solid_count(world: &World, at: [i32; 3], radius: i32) -> usize {
    let mut count = 0;
    for x in -radius..=radius {
        for y in -radius..=radius {
            for z in -radius..=radius {
                if let (Some(x), Some(y), Some(z)) = (
                    at[0].checked_add(x),
                    at[1].checked_add(y),
                    at[2].checked_add(z),
                ) {
                    count += usize::from(world.ground().solid([x, y, z]));
                }
            }
        }
    }
    count
}

impl Bench {
    fn carve_trial(&mut self) {
        let mut model = self.model.borrow_mut();
        let Some(trial) = &mut model.trial else {
            return;
        };
        trial.playing = false;
        let Some(position) = trial.driver.world().position() else {
            trial.carving.feedback = "No controlled critter can reach this target.".into();
            return;
        };
        let mut target = [0; 3];
        for axis in 0..3 {
            let Some(value) = position[axis].checked_add(trial.carving.offset[axis]) else {
                trial.carving.feedback = "Carve target exceeds coordinate range.".into();
                return;
            };
            target[axis] = value;
        }
        let radius = trial.carving.radius;
        let before_ground_revision = trial.driver.world().ground().revision();
        let before_solid_voxels = solid_count(trial.driver.world(), target, radius);
        let reach = trial.driver.world().reach();
        let processed = trial.driver.carve(target, radius);
        let outcomes = if processed {
            format!("{:?}", trial.driver.last_outcomes())
        } else {
            "No tick processed: trial limit or checkpoint.".into()
        };
        trial.carving.action = Some(ActionReceipt {
            target,
            radius,
            reach,
            tick_processed: processed,
            before_ground_revision,
            after_ground_revision: trial.driver.world().ground().revision(),
            before_solid_voxels,
            after_solid_voxels: solid_count(trial.driver.world(), target, radius),
            outcomes: outcomes.clone(),
        });
        trial.carving.feedback = outcomes;
        if processed {
            trial.consume_step();
        }
        model.selected = None;
        model.changed();
    }

    fn move_carve_target(&mut self, axis: usize, delta: i32) {
        let mut model = self.model.borrow_mut();
        if let Some(trial) = &mut model.trial {
            trial.carving.offset[axis] = (trial.carving.offset[axis] + delta).clamp(-64, 64);
            model.changed();
        }
    }
    fn carve_radius(&mut self) {
        let mut model = self.model.borrow_mut();
        if let Some(trial) = &mut model.trial {
            trial.carving.radius = if trial.carving.radius == 1 { 2 } else { 1 };
            model.changed();
        }
    }
    fn reset_carve_target(&mut self) {
        let mut model = self.model.borrow_mut();
        if let Some(trial) = &mut model.trial {
            trial.carving.offset = [0, -1, 0];
            trial.carving.radius = 1;
            model.changed();
        }
    }
    fn show_carving(&mut self) {
        let mut model = self.model.borrow_mut();
        if let Some(trial) = &mut model.trial {
            trial.carving.show = !trial.carving.show;
            trial.refresh_marks();
            model.changed();
        }
    }
}

fn button(label: &'static str, action: impl Fn(&mut Bench) + 'static) -> Child {
    Box::new(focusable(clickable(
        el("button", text(label)).attr("aria-label", label),
        move |state: &mut Bench, _| action(state),
    )))
}

pub(super) fn view(state: &Bench) -> Child {
    let model = state.model.borrow();
    let Some(trial) = &model.trial else {
        return Box::new(el("div", ()));
    };
    let carving = &trial.carving;
    Box::new(el(
        "section",
        (
            el(
                "div",
                vec![
                    button("Carve -X", |s| s.move_carve_target(0, -1)),
                    button("Carve +X", |s| s.move_carve_target(0, 1)),
                    button("Carve -Y", |s| s.move_carve_target(1, -1)),
                    button("Carve +Y", |s| s.move_carve_target(1, 1)),
                    button("Carve -Z", |s| s.move_carve_target(2, -1)),
                    button("Carve +Z", |s| s.move_carve_target(2, 1)),
                    button("Carve radius", Bench::carve_radius),
                    button("Reset carve target", Bench::reset_carve_target),
                    button("Carve ground", Bench::carve_trial),
                    button(
                        if carving.show {
                            "Hide carving"
                        } else {
                            "Show carving"
                        },
                        Bench::show_carving,
                    ),
                ],
            )
            .attr("class", "toolbar"),
            el(
                "p",
                text(format!(
                    "Offset {:?} / radius {} / reach {} / {} carves, {} voxels. {} Each attempt advances one tick. Orange marks indicate recorded locations, raised for display.",
                    carving.offset,
                    carving.radius,
                    trial.driver.world().reach(),
                    carving.events,
                    carving.removed,
                    carving.feedback
                )),
            ),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event(sequence: u64) -> TrialCarve {
        TrialCarve {
            tick: 1,
            sequence,
            organism: mesocosm_core::OrganismId(1),
            at: [2, 3, 4],
            removed: 5,
        }
    }
    #[test]
    fn accepted_records_deduplicate_expire_and_keep_raw_coordinates() {
        let mut carving = Carving::new();
        carving.observe(&[event(0)], 1);
        carving.observe(&[event(0)], 1);
        assert_eq!((carving.events, carving.removed), (1, 5));
        let marks = carving.marks(1, 4., 1.4);
        assert_eq!(marks[0].0, (1, 2, 0));
        assert_eq!(marks[0].1.centre, [2., 7., 4.]);
        assert_eq!(carving.recent[0].at, [2, 3, 4]);
        assert_eq!(carving.marks(8, 4., 1.4).len(), 1);
        assert!(carving.marks(9, 4., 1.4).is_empty());
        carving.observe(&[], 9);
        assert!(carving.recent.is_empty());
    }
    #[test]
    fn bounded_history_and_reset_preserve_configuration() {
        let mut carving = Carving::new();
        carving.offset = [4, 5, 6];
        carving.radius = 2;
        carving.show = false;
        carving.observe(&(0..140).map(event).collect::<Vec<_>>(), 1);
        assert_eq!((carving.recent.len(), carving.events), (128, 140));
        assert!(carving.marks(1, 4., 1.4).is_empty());
        carving.reset();
        assert_eq!(
            (carving.offset, carving.radius, carving.show),
            ([4, 5, 6], 2, false)
        );
        assert_eq!((carving.events, carving.removed), (0, 0));
        carving.observe(&[event(0)], 1);
        assert_eq!(carving.events, 1);
    }
}
