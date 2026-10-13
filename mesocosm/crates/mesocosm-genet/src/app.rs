// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window, the surface, and the loop over a native session.

use std::sync::Arc;
use std::time::Instant;

use isocosm::schema::Id;
use mesocosm_runtime::Runtime;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::chrome::Chrome;
use crate::section::{self, Pan, Section, SiteScene};

pub mod actions;
pub mod bench;
mod config;
mod continuation;
mod devtime;
pub mod drive;
mod follow;
mod frame;
mod graft;
mod inspection;
mod pointer;
mod receipts;
mod setup;
mod startup;

pub use config::HostConfig;

/// Steps the dev "step N" key runs at once (DT1).
pub const DEV_STEP_N: u64 = 10;

/// Matter the dev `G` key places, in mg: a dev tool's dose, not a rule.
pub const DEV_PLACE_MG: u64 = 5_000;

pub struct Host {
    config: HostConfig,
    runtime: Runtime,
    /// The played site, lifted, with its placed bodies. Presentation only.
    scene: Option<SiteScene>,
    /// Current native source of the site's derived picture.
    scene_key: Option<(Id, String)>,
    /// Where the section sits relative to the body it follows. Presentation.
    pan: Pan,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    adapter: Option<wgpu::AdapterInfo>,
    last: Option<Instant>,
    frames: u32,
    steps: u64,
    finished: bool,
    /// Parts of the played body past the lens capsule budget on the last frame.
    body_capsules_dropped: u32,
    /// Which offer of the trait board the cursor is on. Host state only.
    board_row: usize,
    /// Process exit code.
    code: i32,
    /// Host-only time control (DT1); see `app::devtime`.
    dev_paused: bool,
    dev_speed_idx: usize,
    dev_manual_steps: u64,
    /// The body the camera follows when it is not the played one (DT2).
    follow: Option<Id>,
    /// A followed critter that stopped being one.
    follow_lost: Option<mesocosm_views::Lost>,
    inspection: inspection::Inspection,
    pointer: pointer::Pointer,
    continuation: continuation::Pane,
    /// The scenario driving this run, when `--scenario` gave it one (DT4).
    scenario: Option<taproot::Scenario>,
    /// Semantic events since the driver last drained them.
    events: Vec<String>,
    /// Scripted steps still to take, off the clock (DT4).
    pump: Option<actions::Pump>,
    /// Trace entries already turned into events.
    noted: usize,
    /// Trace entries the vitals lane has already read.
    shown: usize,
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// The main view: the section over the played site's lifted Ground.
    section: Section,
    /// The chrome lanes and the device they share; `None` runs chromeless.
    chrome: Option<Lanes>,
    /// The most recently completed Netrender tenant envelope.
    last_tenant_receipt: Option<netrender::OpaqueTenantReceipt>,
}

/// The five chrome lanes over one shared-device Chrome. The checkpoint and the
/// board draw only while the run holds at a question, never both; the dev
/// lane only under `--dev`.
pub(crate) struct Lanes {
    device: Chrome,
    hud: crate::hud::Hud,
    vitals: crate::vitals::VitalsChrome,
    checkpoint: crate::succession::SuccessionChrome,
    board: crate::review::BoardChrome,
    dev: crate::dev::DevChrome,
    directing: crate::succession::SuccessionChrome,
    continuation: crate::succession::SuccessionChrome,
}

impl Host {
    pub fn new(mut config: HostConfig) -> Self {
        if config.spread.is_some() && (config.start.is_some() || config.mode.is_some() || config.unchosen_seed) {
            eprintln!("authored spread supplies its own seed, start and viewing mode");
            std::process::exit(1);
        }
        if config.watch.is_some() && (config.start.is_some() || config.unchosen_seed) {
            eprintln!("a resumed run already has its native founding and start");
            std::process::exit(1);
        }
        let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap_or_else(|why| {
            eprintln!("pack admission: {why:?}");
            std::process::exit(1);
        });
        let mut runtime = match (&config.watch, &config.spread) {
            (Some(_), Some(_)) => Err("authored spread founding cannot replace a resumed run".into()),
            (_, Some(path)) => std::fs::read(path).map_err(|why| why.to_string())
                .and_then(|bytes| serde_json::from_slice::<mesocosm_runtime::author_spread::Setup>(&bytes)
                    .map_err(|why| why.to_string()))
                .and_then(|setup| Runtime::from_authored_spread(&setup, config.ticks_per_second)),
            (Some(path), None) => crate::played::resume_with_mode(path, config.ticks_per_second,config.mode),
            (None, None) => startup::generated(&config, authored.affinity().cloned()),
        }
        .unwrap_or_else(|why| {
            eprintln!("world: {why}");
            std::process::exit(1);
        });
        config.seed = runtime.founded().genesis.seed;
        if config.spread.is_some() {
            if let Some(founding) = &runtime.founded().genesis.founding {
                config.population = founding.population;
            }
        }
        if let Some(path) = &config.spatial {
            let authored = std::fs::read(path).map_err(|why| why.to_string())
                .and_then(|bytes| serde_json::from_slice::<mesocosm_runtime::spatial::Setup>(&bytes)
                    .map_err(|why| why.to_string()))
                .and_then(|setup| runtime.setup_spatial(&setup));
            if let Err(why) = authored {
                eprintln!("spatial {}: {why}", path.display());
                std::process::exit(1);
            }
        }
        if let Some(path) = &config.graft {
            let admitted = std::fs::read(path).map_err(|why| why.to_string())
                .and_then(|bytes| serde_json::from_slice::<mesocosm_runtime::runtime::graft::GraftInput>(&bytes)
                    .map_err(|why| why.to_string()))
                .and_then(|input| runtime.apply_graft(&input));
            if let Err(why) = admitted {
                eprintln!("graft {}: {why}", path.display());
                std::process::exit(1);
            }
        }
        let runtime = runtime.with_authored(authored).unwrap_or_else(|why| {
            eprintln!("pack/native founding conflict: {why}");
            std::process::exit(1);
        });
        // Parsed once, so a typo stops the run before a window opens (DT4).
        let scenario = match config.scenario.as_deref().map(taproot::Scenario::parse) {
            Some(Ok(scenario)) => Some(scenario),
            Some(Err(why)) => {
                eprintln!("scenario: {why}");
                std::process::exit(1);
            },
            None => None,
        };
        Self {
            follow: config.follow,
            config,
            runtime,
            scene: None,
            scene_key: None,
            pan: Pan::default(),
            window: None,
            gpu: None,
            adapter: None,
            last: None,
            frames: 0,
            steps: 0,
            finished: false,
            body_capsules_dropped: 0,
            board_row: 0,
            code: 0,
            dev_paused: false,
            dev_speed_idx: devtime::DEV_SPEED_DEFAULT_IDX,
            dev_manual_steps: 0,
            follow_lost: None,
            inspection: inspection::Inspection::default(),
            pointer: pointer::Pointer::default(),
            continuation: continuation::Pane::default(),
            scenario,
            events: Vec::new(),
            pump: None,
            noted: 0,
            shown: 0,
        }
    }

    /// Runs the host to completion. The `i32` is the process exit code.
    pub fn run(config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut host = Self::new(config);
        event_loop.run_app(&mut host)?;
        Ok(host.code)
    }

    /// Re-lifts the played site when it or its edits changed, and re-places
    /// its bodies every frame. Returns whether the ground was rebound.
    fn refresh_scene(&mut self) -> bool {
        let sim = self.runtime.sim();
        let played = self.runtime.critter();
        let Some(site) = crate::played::scene::site_of(sim, played) else {
            self.scene_key = None;
            return self.scene.take().is_some();
        };
        let visible = self.runtime.view();
        if self.config.spatial.is_some() || !self.runtime.spatial().members.is_empty() {
            let result = self.runtime.physical_view().and_then(|visible|
                crate::played::scene::site_scene_physical(sim, played, &visible, self.runtime.spatial()));
            return match result {
                Ok(scene) => {
                    let window = (&scene.window.min, &scene.window.max, scene.window.datum, scene.window.extent);
                    let key = (site, format!("{}:{}", sim.terrain_revision(), isocosm::digest(&window)));
                    let changed = self.scene_key.as_ref() != Some(&key);
                    self.scene = Some(scene); self.scene_key = Some(key); changed
                },
                Err(why) => {
                    eprintln!("physical site: {why}"); self.scene_key = None; self.scene.take().is_some()
                },
            };
        }
        let key = (site, mesocosm_runtime::runtime::placing::terrain_key(sim));
        if self.scene_key.as_ref() == Some(&key)
            && let Some(scene) = &mut self.scene
        {
            scene.bodies = crate::played::scene::placed_visible(sim, site, &scene.window, &visible);
            scene.volumes = isometer::DeclaredExtentVolumes::from_documents(
                scene.bodies.iter().map(|b| &b.document),
                1,
            );
            scene.played = played;
            return false;
        }
        match crate::played::scene::site_scene_visible(sim, played, &visible) {
            Ok(scene) => {
                self.scene = Some(scene);
                self.scene_key = Some(key);
                true
            },
            Err(why) => {
                eprintln!("site: {why}");
                self.scene_key = None;
                self.scene.take().is_some()
            },
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        if let Some(gpu) = &mut self.gpu {
            gpu.config.width = self.config.width;
            gpu.config.height = self.config.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
            gpu.section.resize(self.config.width, self.config.height);
        }
    }

    /// Steps the world: a scripted pump off the clock, else elapsed wall time
    /// into whole fixed steps.
    fn advance(&mut self) {
        if self.pump.is_some() {
            self.pump_frame();
            return;
        }
        let now = Instant::now();
        let elapsed_us = self
            .last
            .map(|then| now.duration_since(then).as_micros() as u64)
            .unwrap_or(0);
        self.last = Some(now);
        let elapsed_us = self.dev_paced_elapsed(elapsed_us);
        self.steps += self.runtime.advance(elapsed_us);
    }
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.boot(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.finish(event_loop),
            WindowEvent::Resized(size) => self.resize(size.width, size.height),
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match &event.logical_key {
                    key if !event.repeat && self.try_continuation_key(key) => {},
                    Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Tab) if self.continuation_open() => {},
                    key if !event.repeat && self.try_directing_key(key) => {},
                    Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown) if self.directing_open() => {},
                    Key::Named(NamedKey::Escape) => self.finish(event_loop),
                    // Panning is presentation.
                    Key::Named(NamedKey::ArrowLeft) => self.pan.x -= section::PAN_STEP,
                    Key::Named(NamedKey::ArrowRight) => self.pan.x += section::PAN_STEP,
                    Key::Named(NamedKey::ArrowUp) => self.pan.y += section::PAN_STEP,
                    Key::Named(NamedKey::ArrowDown) => self.pan.y -= section::PAN_STEP,
                    // Auto-repeat is dropped, so a held key sends one envelope.
                    key if !event.repeat => self.press_key(key),
                    _ => {},
                }
            },
            WindowEvent::CursorMoved { position, .. } => self.move_pointer([position.x as f32,position.y as f32]),
            WindowEvent::MouseInput { state:ElementState::Pressed,button,.. } => self.press_pointer(button),
            WindowEvent::Focused(false) | WindowEvent::CursorLeft { .. } => self.clear_pointer(),
            WindowEvent::RedrawRequested => {
                self.frame(event_loop);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            _ => {},
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/// The shipped pack's root, from this crate's own location.
fn pack_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|repo| repo.join("packs").join("mesocosm"))
        .unwrap_or_else(|| std::path::PathBuf::from("packs/mesocosm"))
}
