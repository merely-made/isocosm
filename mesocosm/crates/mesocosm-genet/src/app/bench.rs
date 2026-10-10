// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench: a cambium document over a native Isocosm session. Found a
//! world, step it, inspect sites and individuals, save, load, branch and
//! merge. The legacy specimen bench went with the legacy world.

use std::sync::LazyLock;
use std::{cell::Cell, rc::Rc};

use cambium::{AnyView, GenetCtx, GenetElement};
use cambium_genet_winit_host::{HostHooks, HostOptions, Init};
use isomere::{Picked, Seeds, Sizes};

use super::HostConfig;

mod probe;
mod sim;

pub(super) type Child = Box<dyn AnyView<Bench, (), GenetCtx, GenetElement>>;
pub(super) type Logic = fn(&Bench) -> Child;

/// The bench's state: the sim panel and what a scenario drains.
pub(super) struct Bench {
    sim: sim::Panel,
    events: Vec<String>,
    export_directory: std::path::PathBuf,
}

/// Runs the bench until its window closes; the `i32` is the exit code.
pub fn run(config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
    let scenario = match config.scenario.as_deref().map(taproot::Scenario::parse) {
        Some(Ok(scenario)) => Some(scenario),
        Some(Err(why)) => {
            eprintln!("scenario: {why}");
            return Ok(1);
        },
        None => None,
    };
    let options = HostOptions {
        title: "Mesocosm · Bench".into(),
        initial_logical_size: (config.width as f64, config.height as f64),
        ..Default::default()
    };
    let export_directory = config
        .capture
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let exit_code = Rc::new(Cell::new(0));
    let lane = Rc::new(std::cell::RefCell::new(
        probe::Lane::new(
            scenario,
            config.receipt.clone(),
            config.capture.clone(),
            exit_code.clone(),
        )
        .with_frame_limit(config.frames),
    ));
    let close_lane = lane.clone();
    let hooks: HostHooks<Bench, Logic, Child> = HostHooks {
        frame: Box::new(|ctx| {
            if ctx.runner.state().sim.playing {
                ctx.runner.update(|s| s.sim_step());
            }
            ctx.runner.state().sim.playing
        }),
        after_dispatch: Box::new(|_| {}),
        after_frame: Box::new(move |ctx| lane.borrow_mut().after_frame(ctx)),
        after_wake: Box::new(|_| {}),
        close_request: Box::new(move |ctx, _| {
            close_lane.borrow_mut().request_close();
            if let Some(window) = ctx.window {
                window.request_redraw();
            }
            cambium_genet_winit_host::CloseDisposition::KeepVisible
        }),
        focused_text: Box::new(|_| None),
        key_intercept: Box::new(|_, _| false),
    };
    let seed = config.seed;
    cambium_genet_winit_host::run(
        options,
        move |_, _, _| {
            let mut state = Bench {
                sim: sim::Panel::new(seed),
                events: Vec::new(),
                export_directory,
            };
            state.sim_found();
            Init {
                state,
                logic: sim::view as Logic,
                sheet: SHEET.clone(),
                fonts: Vec::new(),
                images: Vec::new(),
            }
        },
        hooks,
    )?;
    Ok(exit_code.get())
}

/// The bench palette, handed to isomere as seeds (M0 of the isomere plan).
fn seeds() -> Seeds {
    let hex = |s: &str| isomere::color_from_hex(s).expect("bench seed hex");
    Seeds {
        background: hex("#eeeae1"),
        ink: hex("#27332e"),
        muted: hex("#65736a"),
        accent: hex("#315c3e"),
        warning: None,
        answer: None,
        picked: Picked {
            panel: Some(hex("#faf8f2")),
            card: Some(hex("#dce3d7")),
            border: Some(hex("#c3cabc")),
            card_border: Some(hex("#b7c1b3")),
            button_border: Some(hex("#a6b3a5")),
            viewport_border: Some(hex("#6f8470")),
            button_bg: Some(hex("#faf8f2")),
            button_ink: Some(hex("#263d2e")),
            hover: Some(hex("#dfebda")),
            focus: Some(hex("#367f56")),
            accent_ink: Some(hex("#ffffff")),
            selected_ink: Some(hex("#ffffff")),
            selected_border: Some(hex("#a6b3a5")),
            error: Some(hex("#27332e")),
            ..Picked::NONE
        },
    }
}

const SIZES: Sizes = Sizes {
    help_font: "15px",
    ..Sizes::DEFAULT
};

/// The bench's own rules, after isomere's shared ones.
const RULES: &str = r#"
.bench { padding:24px; min-height:100vh; background:var(--isomere-background); color:var(--isomere-ink); font:var(--isomere-font-size) sans-serif; }
.generation-input { width:145px; padding:6px; border:1px solid var(--isomere-button-border); background:white; }
.generation-input input { display:block; width:100%; min-height:20px; color:var(--isomere-ink); font:14px monospace; }
.sim h1 { font-size:24px; margin:0 0 6px; }
.sim > p { margin:8px 0; font-size:13px; }
.sim .toolbar { gap:6px; margin:6px 0; align-items:flex-end; }
.sim button { padding:6px 8px; font-size:12px; }
.sim label { font-size:12px; }
.sim .generation-input { width:110px; padding:4px; overflow:hidden; }
.sim .generation-input input { white-space:pre; height:20px; overflow:hidden; }
.sim .sim-saved-world-path { width:320px; }
.sim-columns { display:flex; gap:12px; align-items:flex-start; margin:10px 0; }
.sim-sites { display:flex; flex-wrap:wrap; gap:8px; flex:3; }
.sim-site { box-sizing:border-box; width:calc(50% - 4px); min-width:0; padding:8px; border:1px solid var(--isomere-button-border); background:var(--isomere-panel); }
.sim-site h3 { font-size:15px; margin:0 0 4px; }
.sim-site p { font-size:12px; margin:4px 0; }
.sim-detail { flex:2; min-width:240px; padding:12px; background:var(--isomere-panel); }
.sim-detail h2 { font-size:18px; margin:0 0 8px; }
.sim-detail pre { white-space:pre-wrap; overflow-wrap:anywhere; font:12px monospace; }
.sim #sim-notice { white-space:pre-wrap; overflow-wrap:anywhere; font:12px monospace; }
"#;

pub(super) static SHEET: LazyLock<String> =
    LazyLock::new(|| isomere::sheet_with(&seeds(), &SIZES, RULES));
