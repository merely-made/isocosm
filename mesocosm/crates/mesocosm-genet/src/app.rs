// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm presentation and control state, driven by the shared Assembly.

use std::time::Instant;

use isocosm::schema::Id;
use mesocosm_runtime::Runtime;

use crate::section::{Pan, SiteScene};

pub mod actions;
mod assembly;
pub mod bench;
mod config;
mod continuation;
mod devtime;
pub mod drive;
mod follow;
mod graft;
mod inspection;
mod pointer;
mod startup;

pub use config::HostConfig;

/// Steps the dev "step N" key runs at once (DT1).
pub const DEV_STEP_N: u64 = 10;

/// Matter the dev `G` key places, in mg: a dev tool's dose, not a rule.
pub const DEV_PLACE_MG: u64 = mesocosm_runtime::runtime::keymap::DEV_PLACE_MG;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SceneBranch {
    Physical,
    Coarse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SceneSourceKey {
    revision: u64,
    played: Option<Id>,
    site: Id,
    branch: SceneBranch,
}

pub struct Host {
    config: HostConfig,
    runtime: Runtime,
    /// The played site, lifted, with its placed bodies. Presentation only.
    scene: Option<SiteScene>,
    /// Current native source of the site's derived picture.
    scene_key: Option<(Id, String)>,
    /// Exact successful world cut from which the current scene was built.
    scene_source_key: Option<SceneSourceKey>,
    /// Where the section sits relative to the body it follows. Presentation.
    pan: Pan,
    last: Option<Instant>,
    /// Bounded host callback intervals, before pause/speed scaling. Not GPU timings.
    frame_intervals_us: Vec<u64>,
    frames: u32,
    steps: u64,
    /// Which offer of the trait board the cursor is on. Host state only.
    board_row: usize,
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
}

impl Host {
    pub fn new(mut config: HostConfig) -> Self {
        if config.spread.is_some()
            && (config.start.is_some() || config.mode.is_some() || config.unchosen_seed)
        {
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
            (Some(_), Some(_)) => {
                Err("authored spread founding cannot replace a resumed run".into())
            },
            (_, Some(path)) => std::fs::read(path)
                .map_err(|why| why.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<mesocosm_runtime::author_spread::Setup>(&bytes)
                        .map_err(|why| why.to_string())
                })
                .and_then(|setup| Runtime::from_authored_spread(&setup, config.ticks_per_second)),
            (Some(path), None) => {
                crate::played::resume_with_mode(path, config.ticks_per_second, config.mode)
            },
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
            let authored = std::fs::read(path)
                .map_err(|why| why.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<mesocosm_runtime::spatial::Setup>(&bytes)
                        .map_err(|why| why.to_string())
                })
                .and_then(|setup| runtime.setup_spatial(&setup));
            if let Err(why) = authored {
                eprintln!("spatial {}: {why}", path.display());
                std::process::exit(1);
            }
        }
        if let Some(path) = &config.graft {
            let admitted = std::fs::read(path)
                .map_err(|why| why.to_string())
                .and_then(|bytes| {
                    serde_json::from_slice::<mesocosm_runtime::runtime::graft::GraftInput>(&bytes)
                        .map_err(|why| why.to_string())
                })
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
            scene_source_key: None,
            pan: Pan::default(),
            last: None,
            frame_intervals_us: Vec::new(),
            frames: 0,
            steps: 0,
            board_row: 0,
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
        }
    }

    /// Runs the host to completion. The `i32` is the process exit code.
    pub fn run(config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
        Ok(assembly::run(config))
    }

    /// Re-lifts the played site when its successful source cut changes.
    /// Camera and pan are presentation inputs. Returns whether the ground was rebound.
    fn refresh_scene(&mut self) -> bool {
        let sim = self.runtime.sim();
        let played = self.runtime.critter();
        let Some(site) = crate::played::scene::site_of(&sim, played) else {
            self.scene_key = None;
            self.scene_source_key = None;
            return self.scene.take().is_some();
        };
        // A partial physical world can contain an unplaced controlled member.
        // Its ordinary site view supplies presentation, not a physical binding.
        let physical = self.config.spatial.is_some()
            || played.is_some_and(|id| self.runtime.spatial().members.contains_key(&id));
        let branch = if physical {
            SceneBranch::Physical
        } else {
            SceneBranch::Coarse
        };
        let source_key = self.runtime.revision().ok().map(|revision| SceneSourceKey {
            revision,
            played,
            site,
            branch,
        });
        if source_key.is_some() && self.scene_source_key == source_key && self.scene.is_some() {
            return false;
        }

        let visible = self.runtime.view();
        if physical {
            let result = self.runtime.physical_view().and_then(|visible| {
                crate::played::scene::site_scene_physical(
                    &sim,
                    played,
                    &visible,
                    self.runtime.spatial(),
                )
            });
            return match result {
                Ok(scene) => {
                    let window = (
                        &scene.window.min,
                        &scene.window.max,
                        scene.window.datum,
                        scene.window.ground().extent(),
                    );
                    let key = (
                        site,
                        format!("{}:{}", sim.terrain_revision(), isocosm::digest(&window)),
                    );
                    let changed = self.scene_key.as_ref() != Some(&key);
                    self.scene = Some(scene);
                    self.scene_key = Some(key);
                    self.scene_source_key = source_key;
                    changed
                },
                Err(why) => {
                    eprintln!("physical site: {why}");
                    self.scene_key = None;
                    self.scene_source_key = None;
                    self.scene.take().is_some()
                },
            };
        }
        let key = (site, mesocosm_runtime::runtime::placing::terrain_key(&sim));
        if self.scene_key.as_ref() == Some(&key)
            && let Some(scene) = &mut self.scene
        {
            scene.bodies =
                crate::played::scene::placed_visible(&sim, site, &scene.window, &visible);
            scene.volumes = crate::section::body_volumes(&scene.bodies);
            scene.played = played;
            self.scene_source_key = source_key;
            return false;
        }
        match crate::played::scene::site_scene_visible(&sim, played, &visible) {
            Ok(scene) => {
                let changed = self.scene_key.as_ref() != Some(&key);
                self.scene = Some(scene);
                self.scene_key = Some(key);
                self.scene_source_key = source_key;
                changed
            },
            Err(why) => {
                eprintln!("site: {why}");
                self.scene_key = None;
                self.scene_source_key = None;
                self.scene.take().is_some()
            },
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
        if elapsed_us > 0 && self.frame_intervals_us.len() < 10000 {
            self.frame_intervals_us.push(elapsed_us);
        }
        let elapsed_us = self.dev_paced_elapsed(elapsed_us);
        self.steps += self.runtime.advance(elapsed_us);
    }
}

#[cfg(test)]
#[path = "app/scene_cache_tests.rs"]
mod scene_cache_tests;

/// A play world stepped headless with the demo's answers, printing the
/// living count every `every` ticks: a quick look at how far a world booms.
pub fn curve(config: &HostConfig, ticks: u64, every: u64) -> Result<(), String> {
    let authored = mesocosm_runtime::Authored::load(&pack_root()).map_err(|why| format!("{why:?}"))?;
    let mut runtime = startup::generated(config, authored.affinity().cloned())?;
    for step in 0..ticks {
        crate::played::demo_step(&mut runtime, step);
        if step % every == 0 || step + 1 == ticks {
            let sim = runtime.sim();
            let living: u64 = sim.state().population.groups.values().filter(|g| g.entity.alive).map(|g| g.count).sum();
            println!("curve: soil {}% tick {} living {}", config.play_soil, sim.state().tick, living);
        }
    }
    Ok(())
}

/// The shipped pack's root, from this crate's own location.
fn pack_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|repo| repo.join("packs").join("mesocosm"))
        .unwrap_or_else(|| std::path::PathBuf::from("packs/mesocosm"))
}
