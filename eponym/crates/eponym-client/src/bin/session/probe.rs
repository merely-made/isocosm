// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! P3b: the session host's scenario acceptance, as a thin adapter over
//! `mesquite`.
//!
//! The lane lifecycle — scenario ticking per presented frame, the frame limit,
//! the deferred close, capture arming, PNG writing, the receipt JSON, pixel
//! checks and cost accounting — is the shared crate's, extracted from
//! Mesocosm's bench in P3a. What lives here is what is about *this* host: the
//! snapshot fields a Eponym scenario asserts, its event stream, which leaf is
//! the viewport, the keyboard-only `act` vocabulary, and the default artifact
//! directory. P5 promoted this host's `mark`/`differs`/`dropped` into the
//! shared grammar as `remember` plus `differs`/`dropped`, so nothing here adds
//! a verb any more.
//!
//! The smoke lane in `smoke.rs` is untouched and still answers
//! `PAREDROS_SESSION_SMOKE=1`; this is the driven lane.

use std::{cell::Cell, path::PathBuf, rc::Rc};

use cambium_genet_winit_host::AppCtx;
use mesquite::{CostObservation, Totals, Viewport};
use taproot::{ProbeSnapshot, Scenario};

use super::SessionApp;
use super::view::{Child, Logic, SHEET};

#[path = "probe_act.rs"]
mod act;
#[path = "probe_pixels.rs"]
mod pixels;
#[path = "probe_state.rs"]
mod state;

pub(super) type Context<'a> = AppCtx<'a, SessionApp, Logic, Child>;

/// The session host's answers to the shared lane's questions.
pub(super) struct SessionProduct {
    /// How many accepted `GameEvent`s have already been reported. A load
    /// replaces the session, so the cursor is clamped rather than trusted.
    drained: usize,
}

impl mesquite::Product for SessionProduct {
    type State = SessionApp;
    type Logic = Logic;
    type View = Child;

    const KIND: &'static str = "paredros-session";
    const SURFACE: &'static str = "paredros-session";
    const LOG_PREFIX: &'static str = "session";

    fn sheet(&self) -> &'static str {
        SHEET.as_str()
    }

    fn snapshot(&self, ctx: &Context<'_>, captures: usize, opacity: f32) -> ProbeSnapshot {
        state::snapshot(ctx, captures, opacity)
    }

    fn drain_events(&mut self, ctx: &mut Context<'_>) -> Vec<String> {
        let (events, seen) = state::events_since(ctx, self.drained);
        self.drained = seen;
        events
    }

    fn default_capture_path(&self) -> PathBuf {
        default_out_dir().join("scratch_session.png")
    }

    fn act(&mut self, ctx: &mut Context<'_>, label: &str) -> bool {
        let mut done = false;
        ctx.runner.update(|state| done = act::act(state, label));
        done
    }

    /// Quiet once the producer has presented a frame and no readback is owed.
    /// World changes here are synchronous, so nothing else is outstanding.
    fn busy(&self, ctx: &Context<'_>, capture_pending: bool) -> Option<bool> {
        let rendered = ctx.runner.state().scene.borrow().renders() > 0;
        Some(capture_pending || !rendered)
    }

    fn viewport(&self, ctx: &Context<'_>) -> Option<Viewport> {
        pixels::viewport(ctx)
    }

    fn cost_observation(&self, ctx: &Context<'_>) -> CostObservation {
        let scene = ctx.runner.state().scene.borrow();
        let stats = scene.body_stats();
        CostObservation {
            totals: Totals {
                redraws: scene.renders(),
                mesh_bytes: stats.mesh_upload_bytes,
                instance_bytes: stats.instance_upload_bytes,
            },
            valid: scene.last_error().is_none(),
            // Rigid part placements submitted: the shared stats' nearest
            // reading of the retired `instances` counter.
            populated: stats.draw_parts > 0,
        }
    }
}

/// `eponym/testing/session/`, the headed-verify home for this host.
pub(super) fn default_out_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|eponym| eponym.join("testing").join("session"))
        .unwrap_or_else(|| PathBuf::from("testing/session"))
}

/// The session's scenario lane.
///
/// M4 removed the newtype that used to stand here. It existed only to hide
/// `SessionProduct` behind two forwarding methods, and
/// `isomere::host::ScenarioLane` is implemented for every `mesquite::Lane`
/// whose product matches the host's — deferring the native close until the
/// last frame and the receipt are saved included. So the lane is handed to the
/// assembly as itself.
pub(super) fn lane(
    scenario: Option<Scenario>,
    receipt: Option<PathBuf>,
    final_capture: Option<PathBuf>,
    exit_code: Rc<Cell<i32>>,
    frames: Option<u32>,
) -> mesquite::Lane<SessionProduct> {
    mesquite::Lane::new(
        SessionProduct { drained: 0 },
        scenario,
        receipt,
        final_capture,
        exit_code,
    )
    .with_frame_limit(frames)
}

/// The scenario lane's command line, mirroring Mesocosm's bench flags.
pub(super) struct Options {
    pub scenario: Option<Scenario>,
    pub receipt: Option<PathBuf>,
    pub capture: Option<PathBuf>,
    pub frames: Option<u32>,
    pub size: (f64, f64),
    /// Whether any of the flags above asked for a driven run at all.
    pub driven: bool,
}

const HELP: &str = "\
eponym-client session — one played Eponym session as a Cambium document.

  --scenario PATH  drive the run from a text scenario and exit 1 if it fails
  --receipt PATH   write the run's receipt JSON
  --capture PATH   write the final frame as a PNG, and name generated captures
  --frames N       bound scenario work to N presented frames
  --size WxH       initial window size in logical pixels (default 1240x820)
  --help           this text

Without any of these the window is interactive; PAREDROS_SESSION_SMOKE=1 still
runs the scripted in-window smoke instead. Artifacts default to scratch names
under eponym/testing/session/; acceptance receipts are kept in
eponym/testing/session/receipts/<date>/.";

/// Parses the flags. A bad value exits rather than running a misleading lane.
pub(super) fn options() -> Option<Options> {
    let (mut scenario, mut receipt, mut capture, mut frames) = (None, None, None, None);
    let mut size = (1240.0, 820.0);
    let mut args = std::env::args().skip(1);
    let fail = |why: &str| -> ! {
        eprintln!("session: {why}");
        std::process::exit(2);
    };
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scenario" => {
                let path = args
                    .next()
                    .unwrap_or_else(|| fail("--scenario wants a path"));
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| fail(&format!("scenario {path}: {error}")));
                scenario = Some(
                    Scenario::parse(&text)
                        .unwrap_or_else(|error| fail(&format!("scenario {path}: {error}"))),
                );
            },
            "--receipt" => {
                receipt = Some(PathBuf::from(
                    args.next()
                        .unwrap_or_else(|| fail("--receipt wants a path")),
                ))
            },
            "--capture" => {
                capture = Some(PathBuf::from(
                    args.next()
                        .unwrap_or_else(|| fail("--capture wants a path")),
                ))
            },
            "--frames" => {
                frames = Some(
                    args.next()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or_else(|| fail("--frames wants a positive count")),
                );
            },
            "--size" => {
                let value = args.next().unwrap_or_default();
                let parsed = value.split_once('x').and_then(|(width, height)| {
                    Some((width.parse::<f64>().ok()?, height.parse::<f64>().ok()?))
                });
                size = parsed
                    .filter(|(width, height)| *width >= 1.0 && *height >= 1.0)
                    .unwrap_or_else(|| fail("--size wants positive WIDTHxHEIGHT"));
            },
            "--help" | "-h" => {
                println!("{HELP}");
                return None;
            },
            other => eprintln!("session: ignoring unknown argument: {other}"),
        }
    }
    let driven = scenario.is_some() || receipt.is_some() || capture.is_some() || frames.is_some();
    Some(Options {
        scenario,
        // Scratch names, so an unqualified driven run never overwrites a kept
        // acceptance artifact.
        receipt: receipt.or_else(|| driven.then(|| default_out_dir().join("scratch_session.json"))),
        capture: capture.or_else(|| driven.then(|| default_out_dir().join("scratch_session.png"))),
        frames,
        size,
        driven,
    })
}
