// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native bench acceptance. Controls use the host's actual pointer routing;
//! captures read the presented document, including its embedded depth scene.

use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    path::{Path, PathBuf},
    rc::Rc,
};

use cambium_genet_winit_host::{AppCtx, Frame, HostPointer, WindowCommand, read_frame};
use genet_probe::{
    Automatable, Driveable, Hit, Outcome, ProbeSnapshot, ProbeSurface, Progress, Scenario,
    Selector, SelectorTarget,
};
use serde::Serialize;

use super::{
    state::Bench,
    view::{Child, Logic, SHEET},
};

#[path = "probe_pixels.rs"]
mod pixels;
#[path = "probe_state.rs"]
mod state;
use pixels::{PixelCheck, Viewport, create_parent, write_png};
use state::snapshot;

type Context<'a> = AppCtx<'a, Bench, Logic, Child>;
type Readback = Rc<RefCell<Option<Result<Frame, String>>>>;

struct PendingCapture {
    name: String,
    path: PathBuf,
    armed: u64,
    readback: Readback,
    viewport: Option<Viewport>,
}

#[derive(Serialize)]
struct Capture {
    name: String,
    path: PathBuf,
    width: u32,
    height: u32,
    digest: String,
    fields: BTreeMap<String, String>,
    viewport: Option<Viewport>,
}

pub(super) struct Lane {
    scenario: Option<Scenario>,
    had_scenario: bool,
    outcome: Option<Outcome>,
    receipt: Option<PathBuf>,
    final_capture: Option<PathBuf>,
    final_armed: bool,
    pending: Option<PendingCapture>,
    captures: Vec<Capture>,
    pixel_checks: Vec<PixelCheck>,
    opacity: f32,
    checkpoints: BTreeMap<String, BTreeMap<String, String>>,
    errors: Vec<String>,
    misses: RefCell<Vec<String>>,
    exit_code: Rc<Cell<i32>>,
    frame_limit: Option<u32>,
    frames: u64,
    finished: bool,
}

impl Lane {
    pub fn new(
        scenario: Option<Scenario>,
        receipt: Option<PathBuf>,
        final_capture: Option<PathBuf>,
        exit_code: Rc<Cell<i32>>,
    ) -> Self {
        Self {
            had_scenario: scenario.is_some(),
            scenario,
            outcome: None,
            receipt,
            final_capture,
            final_armed: false,
            pending: None,
            captures: Vec::new(),
            pixel_checks: Vec::new(),
            opacity: 1.0,
            checkpoints: BTreeMap::new(),
            errors: Vec::new(),
            misses: RefCell::new(Vec::new()),
            exit_code,
            frame_limit: None,
            frames: 0,
            finished: false,
        }
    }

    /// Bounds scenario work. An already requested readback/final capture may
    /// use the following frame before the host closes.
    pub fn with_frame_limit(mut self, limit: Option<u32>) -> Self {
        self.frame_limit = limit;
        self
    }

    /// Defer native close until the last requested frame and receipt are saved.
    pub fn request_close(&mut self) {
        if self.finished || self.outcome.is_some() {
            return;
        }
        self.outcome = Some(if let Some(scenario) = self.scenario.take() {
            self.errors
                .push("window closed before the scenario completed".into());
            scenario.finish()
        } else {
            Outcome {
                ok: true,
                log: Vec::new(),
            }
        });
    }

    pub fn after_frame(&mut self, ctx: &mut Context<'_>) {
        if self.finished {
            return;
        }
        self.frames += 1;
        self.collect_capture(ctx);
        if self.pending.is_none()
            && let Some(mut scenario) = self.scenario.take()
        {
            let progress = scenario.tick(&mut Probe { ctx, lane: self });
            if progress == Progress::Done {
                self.outcome = Some(scenario.finish());
            } else {
                self.scenario = Some(scenario);
            }
        }
        if self.outcome.is_none()
            && self
                .frame_limit
                .is_some_and(|limit| self.frames >= u64::from(limit))
        {
            if self.had_scenario {
                self.errors
                    .push("frame limit ran out before the scenario completed".into());
            }
            self.outcome = Some(self.scenario.take().map_or(
                Outcome {
                    ok: true,
                    log: Vec::new(),
                },
                |scenario| scenario.finish(),
            ));
        }
        if self.outcome.is_some() && self.pending.is_none() {
            if !self.final_armed {
                self.final_armed = true;
                if let Some(path) = self.final_capture.clone()
                    && let Err(why) = self.arm_capture(ctx, "final".into(), path)
                {
                    self.errors.push(why);
                }
            }
            if self.pending.is_none() {
                self.finish(ctx);
            }
        }
        if !self.finished
            && (self.scenario.is_some()
                || self.pending.is_some()
                || self.frame_limit.is_some()
                || self.outcome.is_some())
            && let Some(window) = ctx.window
        {
            window.request_redraw();
        }
    }

    fn arm_capture(
        &mut self,
        ctx: &mut Context<'_>,
        name: String,
        path: PathBuf,
    ) -> Result<(), String> {
        if self.pending.is_some() || ctx.capture.is_some() {
            return Err("another native capture is still pending".into());
        }
        if self.captures.iter().any(|capture| capture.path == path) {
            return Err(format!("capture path already used: {}", path.display()));
        }
        let readback = Rc::new(RefCell::new(None));
        let sink = readback.clone();
        *ctx.capture = Some(Box::new(move |surface, view, width, height| {
            *sink.borrow_mut() = Some(
                read_frame(surface, view, width, height)
                    .ok_or_else(|| "native frame readback failed".to_owned()),
            );
        }));
        self.pending = Some(PendingCapture {
            name,
            path,
            armed: self.frames,
            readback,
            viewport: pixels::viewport(ctx),
        });
        Ok(())
    }

    fn collect_capture(&mut self, ctx: &Context<'_>) {
        let Some(pending) = self.pending.take() else {
            return;
        };
        let result = pending.readback.borrow_mut().take();
        let Some(result) = result else {
            if self.frames.saturating_sub(pending.armed) > 8 {
                self.errors.push(format!(
                    "capture {} never reached a presented frame",
                    pending.name
                ));
            } else {
                self.pending = Some(pending);
            }
            return;
        };
        let frame = match result {
            Ok(frame) => frame,
            Err(why) => {
                self.errors.push(format!("capture {}: {why}", pending.name));
                return;
            },
        };
        if frame.width == 0
            || frame.height == 0
            || frame.rgba.len() != frame.width as usize * frame.height as usize * 4
        {
            self.errors.push(format!(
                "capture {} has invalid pixel storage",
                pending.name
            ));
            return;
        }
        if let Err(why) = write_png(&pending.path, &frame) {
            self.errors.push(format!("capture {}: {why}", pending.name));
            return;
        }
        if frame.is_blank()
            || !frame
                .rgba
                .chunks_exact(4)
                .any(|pixel| pixel != &frame.rgba[..4])
        {
            self.errors.push(format!(
                "capture {} contains no visible bench detail",
                pending.name
            ));
        }
        self.captures.push(Capture {
            name: pending.name,
            path: pending.path,
            width: frame.width,
            height: frame.height,
            digest: format!("{:016x}", frame.digest()),
            fields: snapshot(ctx, self.captures.len() + 1, self.opacity).fields,
            viewport: pending.viewport,
        });
    }

    fn named_capture(&self, name: &str) -> PathBuf {
        let path = Path::new(name);
        if path.is_absolute() || name.contains('/') || name.contains('\\') {
            return path.to_path_buf();
        }
        let default = crate::played::default_out_dir().join("specimen-bench.png");
        let base = self
            .final_capture
            .as_ref()
            .or(self.receipt.as_ref())
            .unwrap_or(&default);
        let stem = base
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("specimen-bench");
        let name: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        base.with_file_name(format!("{stem}-{name}.png"))
    }

    fn finish(&mut self, ctx: &mut Context<'_>) {
        self.finished = true;
        self.errors.extend(self.misses.borrow_mut().drain(..));
        let final_state = snapshot(ctx, self.captures.len(), self.opacity);
        if let Some(error) = final_state.field("error").filter(|value| *value != "none") {
            self.errors.push(format!("scene producer: {error}"));
        }
        let outcome = self.outcome.as_ref().expect("completion requested");
        let mut ok = outcome.ok && self.errors.is_empty() && self.exit_code.get() == 0;
        let receipt = serde_json::json!({
            "ok": ok, "kind": "native-specimen-bench", "frames": self.frames,
            "scenario": self.had_scenario, "scenario_log": outcome.log,
            "errors": self.errors, "final": final_state.fields,
            "checkpoints": self.checkpoints, "captures": self.captures,
            "pixel_checks": self.pixel_checks,
        });
        if let Some(path) = &self.receipt {
            let result = create_parent(path).and_then(|()| {
                std::fs::write(
                    path,
                    serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())
            });
            if let Err(why) = result {
                ok = false;
                self.errors
                    .push(format!("receipt {}: {why}", path.display()));
            }
        }
        for line in &outcome.log {
            println!("bench: {line}");
        }
        for error in &self.errors {
            eprintln!("bench: FAIL: {error}");
        }
        println!(
            "bench: RESULT {} ({} frames, {} captures)",
            if ok { "ok" } else { "fail" },
            self.frames,
            self.captures.len()
        );
        if !ok {
            self.exit_code.set(1);
        }
        *ctx.close = true;
    }
}

struct Probe<'a, 'c> {
    ctx: &'a mut Context<'c>,
    lane: &'a mut Lane,
}

impl Automatable for Probe<'_, '_> {
    fn with_surfaces<R>(&self, f: impl FnOnce(&[ProbeSurface<'_>]) -> R) -> R {
        let dom = self.ctx.runner.dom();
        let dom = dom.borrow();
        let (width, height) = self.ctx.logical_size;
        f(&[ProbeSurface {
            name: "specimen-bench",
            dom: &dom,
            rect: [0.0, 0.0, width, height],
            sheet: SHEET,
        }])
    }

    fn selector_target(&self, selector: &Selector) -> SelectorTarget {
        let nodes = genet_probe::matching(&self.ctx.runner.dom().borrow(), selector);
        for node in nodes {
            if let Some((x, y, width, height)) = self.ctx.painted_rect(node)
                && width > 0.0
                && height > 0.0
            {
                return SelectorTarget::Hit(Hit {
                    surface: "specimen-bench",
                    point: pixels::target_point(self.ctx, node, [x, y, width, height]),
                });
            }
        }
        self.lane
            .misses
            .borrow_mut()
            .push(format!("no current DOM target for {selector:?}"));
        SelectorTarget::Miss
    }

    fn snapshot(&self) -> ProbeSnapshot {
        snapshot(self.ctx, self.lane.captures.len(), self.lane.opacity)
    }

    fn drain_events(&mut self) -> Vec<String> {
        let mut events = Vec::new();
        self.ctx
            .runner
            .update(|state| events = std::mem::take(&mut state.events));
        events
    }

    // Bench controls are exercised as controls, through pointer delivery.
    fn act(&mut self, _label: &str) -> bool {
        false
    }
    fn press(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Press(x, y));
    }
    fn moved(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Moved(x, y));
    }
    fn release(&mut self, x: f32, y: f32) {
        self.ctx.pointer.push(HostPointer::Release(x, y));
    }

    fn busy(&mut self) -> Option<bool> {
        let state = self.ctx.runner.state();
        let model = state.model.borrow();
        let scene = state.scene.borrow();
        Some(
            model.creator.pending
                || self.lane.pending.is_some()
                || (state.visible
                    && !state.effects.open
                    && scene.section.is_none()
                    && scene.error.is_none()),
        )
    }
}

impl Driveable for Probe<'_, '_> {
    fn capture(&mut self, name: &str) -> bool {
        let path = self.lane.named_capture(name);
        match self.lane.arm_capture(self.ctx, name.to_owned(), path) {
            Ok(()) => true,
            Err(why) => {
                self.lane.errors.push(why);
                false
            },
        }
    }

    fn app_step(&mut self, line: &str) -> Result<(), String> {
        let words: Vec<_> = line.split_whitespace().collect();
        match words.as_slice() {
            ["input-text", value] => {
                let mut select = cambium::KeyEvent::new(cambium::Key::Character("a".into()));
                select.mods.ctrl = true;
                self.ctx.runner.dispatch_key(select);
                self.ctx
                    .runner
                    .dispatch_key(cambium::KeyEvent::new(cambium::Key::Character(
                        (*value).into(),
                    )));
            },
            ["resize", width, height] => {
                let dimension = |value: &str| {
                    value
                        .parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite() && *v >= 1.0 && *v <= 16_384.0)
                        .ok_or_else(|| format!("invalid native size {value}"))
                };
                self.ctx
                    .window_commands
                    .push(WindowCommand::Resize(dimension(width)?, dimension(height)?));
            },
            ["remember", name] => {
                let fields = self.snapshot().fields;
                self.lane.checkpoints.insert((*name).into(), fields);
            },
            ["zoom", value] => {
                let zoom = value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && (0.5..=2.0).contains(v))
                    .ok_or_else(|| format!("invalid receipt zoom {value}"))?;
                *self.ctx.set_ui_zoom = Some(zoom);
            },
            ["opacity", value] => {
                let opacity = value
                    .parse::<f32>()
                    .ok()
                    .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
                    .ok_or_else(|| format!("invalid receipt opacity {value}"))?;
                self.lane.opacity = opacity;
                *self.ctx.set_sheet = Some(format!(
                    "{SHEET}\n#specimen-viewport {{ opacity:{opacity}; }}"
                ));
            },
            [
                verb @ ("body-pixels-change" | "viewport-pixels-change" | "viewport-pixels-same"),
                first,
                second,
            ] => {
                let check = pixels::compare(&self.lane.captures, verb, first, second)?;
                self.lane.pixel_checks.push(check);
            },
            [verb @ ("same" | "more"), name, fields @ ..] if !fields.is_empty() => {
                let before = self
                    .lane
                    .checkpoints
                    .get(*name)
                    .ok_or_else(|| format!("unknown checkpoint {name}"))?;
                let now = self.snapshot();
                for field in fields {
                    let first = before
                        .get(*field)
                        .ok_or_else(|| format!("checkpoint {name} has no {field}"))?;
                    let current = now
                        .field(field)
                        .ok_or_else(|| format!("no snapshot field {field}"))?;
                    let matches = if *verb == "same" {
                        first == current
                    } else {
                        current
                            .parse::<u64>()
                            .map_err(|_| format!("{field} is not a counter"))?
                            > first
                                .parse::<u64>()
                                .map_err(|_| format!("{field} was not a counter"))?
                    };
                    if !matches {
                        return Err(format!("{verb} {name} {field}: {first} -> {current}"));
                    }
                }
            },
            [
                verb @ ("captures-differ" | "capture-size-changed"),
                first,
                second,
            ] => {
                let capture = |name: &str| {
                    self.lane
                        .captures
                        .iter()
                        .find(|c| c.name == name)
                        .ok_or_else(|| format!("capture {name} has not completed"))
                };
                let (a, b) = (capture(first)?, capture(second)?);
                let changed = if *verb == "captures-differ" {
                    a.digest != b.digest
                } else {
                    (a.width, a.height) != (b.width, b.height)
                };
                if !changed {
                    return Err(format!("{verb}: {first} and {second} agree"));
                }
            },
            _ => return Err(format!("unknown bench step: {line}")),
        }
        Ok(())
    }
}
