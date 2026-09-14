// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! `PAREDROS_SESSION_SMOKE=1`: the scripted volley, inside the real window.
//!
//! The same sequence the timed-action smoke runs, driven through this host's
//! own action methods so the receipt is about this host and not a second copy
//! of the rules. Then a readback of the presented document through
//! `cambium_genet_winit_host::read_frame`, a non-trivial check over the
//! viewport leaf's own painted rectangle, a capture PNG, a save file, one
//! final line and an exit code.
//!
//! This is not the P3 scenario driver: no `genet-probe` scenario, no synthetic
//! input, no receipt JSON. P3 owns those.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Instant;

use cambium_genet_winit_host::{AppCtx, Frame, read_frame};
use genet_probe::Selector;
use mesocosm_core::PartId;
use paredros_world::{GameEvent, ItemKind, StrikeOutcome};

use super::view::{Child, Logic};
use super::{CHARGE_INTERVAL, SessionApp};

type Context<'a> = AppCtx<'a, SessionApp, Logic, Child>;

/// Frames the readback is given before the lane calls it lost.
const READBACK_GRACE: u64 = 16;
/// Minimum pixels inside the viewport that must differ from its modal colour.
const MIN_VIEWPORT_DETAIL: usize = 256;

enum Stage {
    /// Let the document present once so the producer has a frame.
    Settle,
    /// Run the volley, then wait for the frame that shows its consequences.
    Script,
    /// Readback armed; waiting for the presented frame.
    Capture,
    Done,
}

pub(super) struct Lane {
    directory: PathBuf,
    exit: Rc<Cell<i32>>,
    stage: Stage,
    frames: u64,
    armed: u64,
    presented: u64,
    readback: Rc<RefCell<Option<Result<Frame, String>>>>,
    viewport: Option<([f32; 4], f32)>,
    notes: Vec<String>,
    errors: Vec<String>,
}

impl Lane {
    pub(super) fn new(directory: PathBuf, exit: Rc<Cell<i32>>) -> Self {
        Self {
            directory,
            exit,
            stage: Stage::Settle,
            frames: 0,
            armed: 0,
            presented: 0,
            readback: Rc::new(RefCell::new(None)),
            viewport: None,
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// A close arriving before the script finished is a failure, not an exit.
    pub(super) fn refuse(&mut self) {
        if !matches!(self.stage, Stage::Done) {
            self.errors
                .push("window closed before the smoke finished".into());
            self.finish();
        }
    }

    pub(super) fn after_frame(&mut self, ctx: &mut Context<'_>) {
        self.frames += 1;
        self.presented += 1;
        match self.stage {
            Stage::Settle => {
                if self.frames >= 2 {
                    ctx.runner.update(|state| script(state, &mut self.errors));
                    self.stage = Stage::Script;
                }
            },
            Stage::Script => {
                self.viewport = viewport(ctx);
                if self.viewport.is_none() {
                    self.errors
                        .push("the scene viewport has no painted box".into());
                    self.finish();
                    self.stage = Stage::Done;
                } else {
                    self.arm(ctx);
                    self.stage = Stage::Capture;
                }
            },
            Stage::Capture => self.collect(ctx),
            Stage::Done => {},
        }
        if !matches!(self.stage, Stage::Done)
            && let Some(window) = ctx.window
        {
            window.request_redraw();
        }
        if matches!(self.stage, Stage::Done) {
            *ctx.close = true;
        }
    }

    fn arm(&mut self, ctx: &mut Context<'_>) {
        let sink = self.readback.clone();
        *ctx.capture = Some(Box::new(move |surface, view, width, height| {
            *sink.borrow_mut() = Some(
                read_frame(surface, view, width, height)
                    .ok_or_else(|| "native frame readback failed".to_owned()),
            );
        }));
        self.armed = self.frames;
    }

    fn collect(&mut self, ctx: &mut Context<'_>) {
        let result = self.readback.borrow_mut().take();
        let Some(result) = result else {
            if self.frames.saturating_sub(self.armed) > READBACK_GRACE {
                self.errors
                    .push("the capture never reached a presented frame".into());
                self.stage = Stage::Done;
                self.finish();
            }
            return;
        };
        self.stage = Stage::Done;
        match result {
            Ok(frame) => self.inspect(ctx, frame),
            Err(why) => self.errors.push(format!("capture: {why}")),
        }
        self.finish();
    }

    fn inspect(&mut self, ctx: &Context<'_>, frame: Frame) {
        if frame.width == 0
            || frame.height == 0
            || frame.rgba.len() != frame.width as usize * frame.height as usize * 4
        {
            self.errors
                .push("the capture has invalid pixel storage".into());
            return;
        }
        let path = self.directory.join("session-smoke.png");
        if let Err(why) = write_png(&path, &frame) {
            self.errors.push(format!("capture: {why}"));
            return;
        }
        self.notes.push(format!("capture {}", path.display()));
        if frame
            .rgba
            .chunks_exact(4)
            .all(|pixel| pixel == &frame.rgba[..4])
        {
            self.errors
                .push("the presented frame is one flat colour".into());
        }
        let Some((rect, scale)) = self.viewport else {
            return;
        };
        let mut colours = BTreeMap::<[u8; 3], usize>::new();
        for y in 0..frame.height {
            for x in 0..frame.width {
                let point = ((x as f32 + 0.5) / scale, (y as f32 + 0.5) / scale);
                // Two logical pixels of the leaf edge are discarded as anti-aliasing.
                if point.0 < rect[0] + 2.0
                    || point.0 >= rect[0] + rect[2] - 2.0
                    || point.1 < rect[1] + 2.0
                    || point.1 >= rect[1] + rect[3] - 2.0
                {
                    continue;
                }
                let index = (y as usize * frame.width as usize + x as usize) * 4;
                *colours
                    .entry([
                        frame.rgba[index],
                        frame.rgba[index + 1],
                        frame.rgba[index + 2],
                    ])
                    .or_default() += 1;
            }
        }
        let sampled: usize = colours.values().sum();
        let modal = colours.values().copied().max().unwrap_or(0);
        let detail = sampled - modal;
        self.notes.push(format!(
            "viewport {sampled} px, {} colours, {detail} off-modal",
            colours.len()
        ));
        if sampled == 0 {
            self.errors
                .push("the viewport mask covered no captured pixels".into());
        } else if detail < MIN_VIEWPORT_DETAIL || colours.len() < 3 {
            self.errors.push(format!(
                "the viewport region is trivial: {detail} off-modal pixels in {} colours",
                colours.len()
            ));
        }
        if let Some(error) = ctx.runner.state().published_error.clone() {
            self.errors
                .push(format!("viewport error published: {error}"));
        }
    }

    fn finish(&mut self) {
        self.stage = Stage::Done;
        let failed = !self.errors.is_empty();
        if failed {
            self.exit.set(1);
            for error in &self.errors {
                eprintln!("session smoke: {error}");
            }
        }
        println!(
            "session smoke: {} after {} presented frame(s); {}",
            if failed { "FAILED" } else { "passed" },
            self.presented,
            self.notes.join("; ")
        );
    }
}

/// The leaf's painted rectangle, in logical pixels, plus the physical scale.
fn viewport(ctx: &Context<'_>) -> Option<([f32; 4], f32)> {
    let dom = ctx.runner.dom();
    let dom = dom.borrow();
    let node = genet_probe::matching(&dom, &Selector::role("img").containing("Scene"))
        .into_iter()
        .next()?;
    drop(dom);
    let (x, y, width, height) = ctx.painted_rect(node)?;
    (width > 0.0 && height > 0.0).then(|| {
        (
            [x, y, width, height],
            ctx.window.map_or(1.0, |window| window.scale_factor()) as f32 * ctx.ui_zoom,
        )
    })
}

/// The volley, the injury, the pickup, the rest, and a save/load round trip —
/// each through the host's own action path.
fn script(state: &mut SessionApp, errors: &mut Vec<String>) {
    macro_rules! check {
        ($condition:expr, $why:expr) => {
            if !$condition {
                errors.push(($why).to_owned());
            }
        };
    }
    state.move_played([1, 0, 0]);
    state.prepare(paredros_world::timed_action::Direction::Right);
    state.join_limb(PartId(2));
    state.charging = true;
    for _ in 0..4 {
        state.last_charge = Instant::now() - CHARGE_INTERVAL;
        state.tick_charge();
    }
    let events = {
        let target = state.target;
        let rules = state.combat_rules;
        let tick = state
            .model
            .borrow()
            .action()
            .and_then(|action| action.action())
            .map(|open| open.last_tick);
        match tick {
            Some(tick) => state
                .model
                .borrow_mut()
                .action_mut()
                .expect("the session holds a timed action")
                .release_against(target, tick, rules)
                .map_err(|error| format!("{error:?}")),
            None => Err("no action was open to release".to_owned()),
        }
    };
    state.charging = false;
    match &events {
        Ok(events) => {
            let hit = events.iter().any(|event| match event {
                GameEvent::VolleyResolved { strikes, .. } => strikes
                    .iter()
                    .any(|strike| matches!(strike.outcome, StrikeOutcome::Hit { .. })),
                _ => false,
            });
            check!(hit, "the scripted volley landed no strike");
            state.status = vec![format!("Smoke volley: {} event(s)", events.len())];
        },
        Err(why) => errors.push(format!("release: {why}")),
    }
    {
        let model = state.model.borrow();
        let severed = model
            .game()
            .current_anatomy(state.target)
            .map(|record| record.document.parts.iter().any(|part| part.severed))
            .unwrap_or(false);
        check!(severed, "the scripted volley severed no target part");
    }
    state.take_dressing();
    state.injure();
    state.rest();
    {
        let model = state.model.borrow();
        let game = model.game();
        let played = model.played();
        check!(
            game.bodies()
                .get(played)
                .is_some_and(|body| body.wound == 0),
            "rest did not clear the played subject's wound"
        );
        check!(
            game.current_anatomy(played)
                .map(|record| {
                    record
                        .document
                        .part(PartId(1))
                        .is_some_and(|part| part.severed)
                })
                .unwrap_or(false),
            "the injury debug did not sever part 1"
        );
        check!(
            game.items()
                .carried_by(played)
                .filter(|item| item.kind == ItemKind::Dressing)
                .count()
                == 0,
            "the dressing pickup left no consumed dressing"
        );
    }
    let before = state.save_bytes();
    state.save();
    check!(
        state
            .status
            .first()
            .is_some_and(|line| line.starts_with("Saved ")),
        "save did not report a written file"
    );
    state.move_played([0, 0, -1]);
    state.load();
    check!(
        state
            .status
            .first()
            .is_some_and(|line| line.starts_with("Loaded ")),
        "load did not report a restored file"
    );
    check!(
        before.is_ok() && state.save_bytes() == before,
        "the save/load round trip did not restore the same bytes"
    );
    // Frame the aftermath rather than the mid-volley pose.
    state.selected = state
        .scene
        .borrow()
        .pick_body_ignoring_terrain([0.0, 0.0])
        .or(state.selected);
}

fn write_png(path: &Path, frame: &Frame) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let file = BufWriter::new(File::create(path).map_err(|error| error.to_string())?);
    let mut encoder = png::Encoder::new(file, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
    writer
        .write_image_data(&frame.rgba)
        .map_err(|error| error.to_string())?;
    writer.finish().map_err(|error| error.to_string())
}
