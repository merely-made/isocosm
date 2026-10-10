// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host, driven by a text scenario. (DT4)
//!
//! The host implements [`Automatable`] and [`Driveable`], and
//! [`taproot::Scenario`] pumps a text file against it one step per frame.
//!
//! | Verb | Here |
//! | --- | --- |
//! | `act <name>` | a documented key name or a host action; see [`super::actions`] |
//! | `settle N` | pump N frames |
//! | `wait [cap]` | hold until [`Automatable::busy`] reads quiet |
//! | `assert text <s>` | `s` is on a chrome lane that is on screen |
//! | `assert snap <f> <op> <v>` | a field of the snapshot below |
//! | `assert event <s>` | `s` is in what the runtime answered |
//! | `capture <name>` | a PNG through [`super::Host::capture_to`] |
//!
//! Busy means scripted work still in flight: a demo pumping or an envelope
//! queued. A checkpoint with nothing queued is quiet, because nothing about
//! it resolves on its own and the script's next step is its answer.
//!
//! # Stack gaps found doing this, reported rather than built
//!
//! - **Pointer delivery.** `Automatable` requires `press`/`moved`/`release`,
//!   and `mesocosm-genet` has nowhere to route a window point: DT2 already
//!   found that click-to-select in the section needs picking machinery this
//!   host does not have, and the chrome lanes are rasters composited over the
//!   frame with no hit-test path back into cambium. So the three are
//!   **attributed no-ops**: they record `pointer-unrouted x y` into the event
//!   stream, which an `assert event` can catch, rather than silently swallowing
//!   a click and letting the scenario believe it landed. `click` therefore
//!   resolves a selector correctly and then goes nowhere, which is the honest
//!   state of this host and not a defect in taproot.
//! - **No verb for founding a world.** The seed and the population are flags.
//! - **No loop or repeat verb.** The demo is a host action taking a count
//!   (`act demo 3100`).

use std::path::PathBuf;

use taproot::{Automatable, Driveable, ProbeSnapshot, ProbeSurface, Progress};
use winit::event_loop::ActiveEventLoop;

use super::Host;
use crate::played;

impl Host {
    /// Pumps the scenario one step, after the frame it is asserting about was
    /// drawn. Ends the run when the steps are exhausted, or when a frame limit
    /// ran out with steps left — which is a failure, because a scenario that
    /// did not finish asserted nothing about the rest of itself.
    pub(super) fn drive_scenario(&mut self, event_loop: &ActiveEventLoop, hit_limit: bool) {
        let Some(mut scenario) = self.scenario.take() else {
            return;
        };
        let progress = scenario.tick(self);
        if progress == Progress::Running && !hit_limit {
            self.scenario = Some(scenario);
            return;
        }
        let outcome = scenario.finish();
        for line in &outcome.log {
            println!("scenario: {line}");
        }
        let finished = progress == Progress::Done;
        if !finished {
            println!("scenario: the frame limit ran out with steps left");
        }
        let ok = outcome.ok && finished;
        println!("scenario: {}", if ok { "ok" } else { "FAILED" });
        if !ok {
            self.code = 1;
        }
        self.finish(event_loop);
    }

    /// What the runtime answered since the last call, as grep-friendly
    /// strings an `assert event` matches: each applied envelope's outcome,
    /// and what the last round brought. Nothing accumulates without a
    /// scenario to drain it.
    pub(super) fn note_outcomes(&mut self) {
        if self.config.scenario.is_none() {
            return;
        }
        let trace = self.runtime.trace();
        if trace.len() == self.noted {
            return;
        }
        for (envelope, outcome) in &trace[self.noted.min(trace.len())..] {
            self.events
                .push(format!("outcome {:?} {outcome:?}", envelope.intent));
        }
        for happening in self.runtime.happenings() {
            self.events.push(format!("happening {happening:?}"));
        }
        self.noted = trace.len();
    }

    /// Where a `capture <name>` writes.
    ///
    /// A name carrying a separator is a path and is taken as one, so a receipt
    /// can name an explicit non-default file; a bare name lands beside the
    /// fixtures in the workspace's headed-verify home, which is where the
    /// run's own capture goes.
    fn capture_path(&self, name: &str) -> PathBuf {
        let path = PathBuf::from(name);
        if path.is_absolute() || name.contains('/') || name.contains('\\') {
            path
        } else {
            played::default_out_dir().join(format!("{name}.png"))
        }
    }
}

impl Automatable for Host {
    /// The chrome lanes that are **actually on screen**, topmost first.
    ///
    /// The board and the checkpoint are listed only while they stand and the
    /// dev tile only under `--dev`, because that is what makes `assert text` a
    /// claim about what a person would see rather than about a retained tree
    /// nobody is looking at. The minimap is not here at all: it is the painted
    /// lane, and by lane discipline it holds no words.
    fn with_surfaces<R>(&self, f: impl FnOnce(&[ProbeSurface<'_>]) -> R) -> R {
        let Some(gpu) = &self.gpu else {
            return f(&[]);
        };
        let Some(lanes) = &gpu.chrome else {
            return f(&[]);
        };
        let frame = (gpu.config.width, gpu.config.height);
        let (board_dom, board_rect, board_sheet) = lanes.board.probe(frame);
        let (held_dom, held_rect, held_sheet) = lanes.checkpoint.probe(frame);
        let (dev_dom, dev_rect, dev_sheet) = lanes.dev.probe(frame);
        let (vitals_dom, vitals_rect, vitals_sheet) = lanes.vitals.probe(frame);
        let board = board_dom.borrow();
        let held = held_dom.borrow();
        let dev = dev_dom.borrow();
        let vitals = vitals_dom.borrow();

        let mut surfaces = Vec::with_capacity(4);
        if lanes.board.standing() {
            surfaces.push(ProbeSurface {
                name: "board",
                dom: &board,
                rect: board_rect,
                sheet: board_sheet,
            });
        }
        if lanes.checkpoint.standing() {
            surfaces.push(ProbeSurface {
                name: "checkpoint",
                dom: &held,
                rect: held_rect,
                sheet: held_sheet,
            });
        }
        if self.config.dev {
            surfaces.push(ProbeSurface {
                name: "dev",
                dom: &dev,
                rect: dev_rect,
                sheet: dev_sheet,
            });
        }
        surfaces.push(ProbeSurface {
            name: "vitals",
            dom: &vitals,
            rect: vitals_rect,
            sheet: vitals_sheet,
        });
        f(&surfaces)
    }

    /// Everything a scenario asserts that the lanes cannot say in words. The
    /// hash is hex to sixteen places, matching the receipt line.
    fn snapshot(&self) -> ProbeSnapshot {
        let dev_intents = self.runtime.dev_intents();
        let some = |id: Option<u64>| id.map_or("none".to_string(), |id| id.to_string());
        let living = self
            .runtime
            .sim()
            .state()
            .population
            .groups
            .values()
            .filter(|group| group.entity.alive)
            .map(|group| group.count)
            .sum::<u64>();
        ProbeSnapshot {
            focused: self.followed().map(|id| format!("critter {id}")),
            fields: [
                ("hash", format!("{:016x}", self.runtime.state_hash())),
                ("mode", self.mode().to_string()),
                ("camera", self.config.camera.name().to_string()),
                ("tick", self.runtime.tick().to_string()),
                ("steps", self.steps.to_string()),
                ("frames", self.frames.to_string()),
                ("dev", self.config.dev.to_string()),
                ("dev-intents", dev_intents.to_string()),
                (
                    "assisted",
                    match played::assisted_label(dev_intents) {
                        label if label.is_empty() => "unassisted".to_string(),
                        label => label,
                    },
                ),
                ("queued", self.runtime.queued_len().to_string()),
                ("controlled", some(self.runtime.critter())),
                ("follow", some(self.followed())),
                ("site", some(self.scene.as_ref().map(|s| s.site))),
                ("living", living.to_string()),
                ("checkpoint", yes_no(self.runtime.checkpoint().is_some())),
                ("boundary", yes_no(self.runtime.review().is_some())),
                ("fault", self.runtime.fault().unwrap_or("").to_string()),
                ("paused", yes_no(self.dev_paused)),
                ("inspecting", yes_no(self.inspection.open)),
                (
                    "selected-part",
                    self.inspection
                        .selected
                        .map_or("none".into(), |s| s.part.0.to_string()),
                ),
            ]
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
        }
    }

    fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.events)
    }

    fn act(&mut self, label: &str) -> bool {
        self.run_action(label)
    }

    // Attributed, not delivered. See the stack gap in the module docs: this
    // host has no route from a window point to either the section or a chrome
    // raster, and a silent swallow would let a scenario believe a click landed.
    fn press(&mut self, x: f32, y: f32) {
        self.events.push(format!("pointer-unrouted {x} {y}"));
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.events.push(format!("pointer-unrouted {x} {y}"));
    }

    fn release(&mut self, x: f32, y: f32) {
        self.events.push(format!("pointer-unrouted {x} {y}"));
    }

    /// See the module docs: scripted work in flight, and a checkpoint with
    /// nothing queued is quiet.
    fn busy(&mut self) -> Option<bool> {
        Some(self.pump.is_some() || self.runtime.queued_len() > 0)
    }
}

impl Driveable for Host {
    fn capture(&mut self, name: &str) -> bool {
        let path = self.capture_path(name);
        match self.capture_to(&path) {
            Ok(()) => {
                self.events.push(format!("captured {}", path.display()));
                true
            },
            Err(why) => {
                eprintln!("capture: {why}");
                false
            },
        }
    }
}

fn yes_no(flag: bool) -> String {
    if flag { "yes" } else { "no" }.to_string()
}
