// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm presentation and control state, driven by the shared Assembly.

use std::time::Instant;

use cosmograph::schema::Id;
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
mod view;
mod graft;
mod holding;
mod inspection;
mod pointer;
mod reading;
mod scene_source;
mod startup;

pub use config::{HostConfig, WorldRules};

/// Steps the dev "step N" key runs at once (DT1).
pub const DEV_STEP_N: u64 = 10;

/// Matter the dev `G` key places, in mg: a dev tool's dose, not a rule.
pub const DEV_PLACE_MG: u64 = mesocosm_runtime::runtime::keymap::DEV_PLACE_MG;

use scene_source::{SceneBranch, SceneSourceKey};

pub struct Host {
    config: HostConfig,
    runtime: Runtime,
    /// The startup pack's presentation styles (1182); never saved.
    styles: mesocosm_runtime::StyleSheet,
    /// The played site, lifted, with its placed bodies. Presentation only.
    scene: Option<std::rc::Rc<SiteScene>>,
    /// Current native source of the site's derived picture.
    scene_key: Option<(Id, String)>,
    /// Exact successful world cut from which the current scene was built.
    scene_source_key: Option<SceneSourceKey>,
    scene_observed_revision: Option<u64>,
    scene_components: Option<crate::played::components::ComponentScene>,
    scene_extractions: u64,
    /// Where the section sits relative to the body it follows. Presentation.
    pan: Pan,
    /// Which camera the played body is seen through. Presentation.
    view: isometer::View,
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
    /// The newest refused round key and the tick it stops showing at.
    round_notice: Option<(String, u64)>,
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
        let authored = mesocosm_runtime::Authored::load(&pack_root()).unwrap_or_else(|why| {
            eprintln!("pack admission: {why:?}");
            std::process::exit(1);
        });
        let mut runtime = startup::configured(&config, &authored).unwrap_or_else(|why| {
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
        let styles = authored.styles().clone();
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
        let dev_paused = config.dev && config.start_paused;
        Self {
            follow: config.follow,
            config,
            runtime,
            styles,
            scene: None,
            scene_key: None,
            scene_source_key: None,
            scene_observed_revision: None,
            scene_components: None,
            scene_extractions: 0,
            pan: Pan::default(),
            view: isometer::View::default(),
            last: None,
            frame_intervals_us: Vec::new(),
            frames: 0,
            steps: 0,
            board_row: 0,
            dev_paused,
            dev_speed_idx: devtime::DEV_SPEED_DEFAULT_IDX,
            dev_manual_steps: 0,
            follow_lost: None,
            inspection: inspection::Inspection::default(),
            pointer: pointer::Pointer::default(),
            continuation: continuation::Pane::default(),
            round_notice: None,
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
        if let Err(why) = self.runtime.synchronize_view() {
            eprintln!("scene source: {why}");
            self.scene_key = None;
            self.scene_source_key = None;
            self.scene_components = None;
            return self.scene.take().is_some();
        }
        let sim = self.runtime.sim();
        let styles = &self.styles;
        let played = self.runtime.critter();
        let Some(site) = crate::played::scene::site_of(&sim, played) else {
            self.scene_key = None;
            self.scene_source_key = None;
            self.scene_components = None;
            return self.scene.take().is_some();
        };
        // A marked physical world never fills a missing native pose with a
        // presentation grid spot, including after birth or a refused arrival.
        let physical = self.config.spatial.is_some()
            || self.runtime.spatial().driver.is_some()
            || played.is_some_and(|id| self.runtime.spatial().members.contains_key(&id));
        let branch = if physical {
            SceneBranch::Physical
        } else {
            SceneBranch::Coarse
        };
        let revision = self.runtime.revision().ok();
        let native = self.runtime.world().native_revision().ok();
        if revision.is_some()
            && self.scene_observed_revision == revision
            && native.as_ref().is_some_and(|native| {
                self.scene_source_key.as_ref().is_some_and(|key| {
                    key.context_matches(
                        &self.runtime,
                        native,
                        played,
                        site,
                        branch,
                        self.config.component_scene,
                    )
                })
            })
            && self.scene.is_some()
        {
            return false;
        }
        let source_key = native.map(|native| {
            SceneSourceKey::new(
                &self.runtime,
                native,
                played,
                site,
                branch,
                self.config.component_scene,
            )
        });
        self.scene_observed_revision = revision;
        if source_key.is_some() && self.scene_source_key == source_key && self.scene.is_some() {
            return false;
        }

        self.scene_extractions += 1;
        let visible = self.runtime.view();
        let holder = self.runtime.interim().participant;
        if physical {
            let result = self
                .runtime
                .physical_view()
                .and_then(|visible| {
                    crate::played::scene::site_scene_physical(
                        &sim,
                        played,
                        &visible,
                        self.runtime.spatial(),
                    )
                })
                .and_then(|scene| {
                    scene_source::store(
                        scene,
                        self.config.component_scene,
                        &mut self.scene_components,
                        |scene| styled(scene, &sim, styles, holder),
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
                        format!("{}:{}", sim.terrain_revision(), cosmograph::digest(&window)),
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
                    self.scene_components = None;
                    self.scene.take().is_some()
                },
            };
        }
        let key = (site, mesocosm_runtime::runtime::placing::terrain_key(&sim));
        if self.scene_key.as_ref() == Some(&key)
            && let Some(scene) = &self.scene
        {
            let mut bodies =
                crate::played::scene::placed_visible(&sim, site, &scene.window, &visible);
            let volumes = match crate::section::body_volumes(&mut bodies) {
                Ok(volumes) => volumes,
                Err(why) => {
                    eprintln!("site appearance: {why}");
                    self.scene_key = None;
                    self.scene_source_key = None;
                    self.scene_components = None;
                    return self.scene.take().is_some();
                },
            };
            let scene = SiteScene {
                site,
                ground: scene.ground.clone(),
                window: scene.window.clone(),
                bodies,
                played,
                volumes,
                walking: scene.walking.clone(),
            };
            self.scene = match scene_source::store(
                scene,
                self.config.component_scene,
                &mut self.scene_components,
                |scene| styled(scene, &sim, styles, holder),
            ) {
                Ok(scene) => Some(scene),
                Err(why) => {
                    eprintln!("scene components: {why}");
                    self.scene_key = None;
                    self.scene_source_key = None;
                    self.scene_components = None;
                    return self.scene.take().is_some();
                },
            };
            self.scene_source_key = source_key;
            return false;
        }
        let result =
            crate::played::scene::site_scene_visible(&sim, played, &visible).and_then(|scene| {
                scene_source::store(
                    scene,
                    self.config.component_scene,
                    &mut self.scene_components,
                    |scene| styled(scene, &sim, styles, holder),
                )
            });
        match result {
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
                self.scene_components = None;
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
        // An answered question leaves no stale cursor for the next one.
        if self.runtime.review().is_none() && self.runtime.checkpoint().is_none() { self.board_row = 0; }
    }
}

#[cfg(test)]
#[path = "app/scene_cache_tests.rs"]
mod scene_cache_tests;

#[cfg(test)]
#[path = "app/founding_tests.rs"]
mod founding_tests;

#[path = "app/curve.rs"]
mod curve_run;
pub use curve_run::run as curve;

/// The shipped pack's root, from this crate's own location.
fn pack_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|repo| repo.join("packs").join("mesocosm"))
        .unwrap_or_else(|| std::path::PathBuf::from("packs/mesocosm"))
}

/// The pack's styles on a freshly projected scene (1182).
fn styled(
    scene: &mut SiteScene,
    sim: &cosmograph::simulation::Simulation,
    styles: &mesocosm_runtime::StyleSheet,
    holder: cosmograph::schema::Id,
) {
    let played = scene.played;
    crate::played::scene::style(&mut scene.bodies, sim, played, styles);
    // Bodies carrying what the holder holds are tinted (the minimap's diamond).
    let surfaces = mesocosm_views::held_surfaces(sim, holder);
    crate::played::scene::badge_carriers(&mut scene.bodies, sim, &surfaces);
}
