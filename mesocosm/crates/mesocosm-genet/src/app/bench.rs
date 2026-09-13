// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native specimen bench over the existing creator and shared-depth Section.
//! Genet owns document geometry and input; this module owns specimen readings.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{Host, HostConfig};
use cambium_genet_winit_host::{HostHooks, HostOptions, Init};

mod comparison;
mod comparison_view;
mod effects;
mod generation_controls;
mod probe;
mod producer;
mod state;
mod structure_controls;
mod view;

use state::{Bench, Specimen};
use view::{Child, Logic};

const LEAF_KEY: u64 = 0x5350_4543;

/// Runs the bench with the creator flags already admitted by the ordinary CLI.
pub fn run(config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
    run_comparison(config, None)
}

pub fn run_comparison(
    config: HostConfig,
    path: Option<std::path::PathBuf>,
) -> Result<i32, winit::error::EventLoopError> {
    run_inputs(config, path, None)
}

pub fn run_inputs(
    mut config: HostConfig,
    path: Option<std::path::PathBuf>,
    effect_path: Option<std::path::PathBuf>,
) -> Result<i32, winit::error::EventLoopError> {
    let effect = match effect_path {
        Some(path) => match effects::Effects::load(&path) {
            Ok(effect) => effect,
            Err(why) => {
                eprintln!("Effect experiment refused: {why}");
                return Ok(1);
            },
        },
        None => effects::Effects::default(),
    };
    let restore = match path {
        Some(path) => match comparison::SavedComparison::load(&path) {
            Ok(saved) => {
                config.creator_request = Some(saved.selection.source.request.clone());
                Some(saved)
            },
            Err(why) => {
                eprintln!("{why}");
                return Ok(1);
            },
        },
        None => None,
    };
    if config.creator_request.is_none() {
        config.creator_request = Some(mesocosm_core::world::generation::Request {
            seed: config.seed,
            ..Default::default()
        });
    }
    let options = HostOptions {
        title: "Mesocosm · Specimen bench".into(),
        initial_logical_size: (config.width as f64, config.height as f64),
        ..Default::default()
    };
    let mut host = Host::new(config);
    if let Some(saved) = &restore {
        host.volumes = saved
            .content
            .resolve()
            .expect("saved content was validated");
        host.content = Some(saved.content.clone());
        host.creator = Some(super::creator::Creator::new(
            saved.selection.source.request.clone(),
            saved.content.palette,
            host.runtime.world(),
            host.config.camera,
            host.config.creator_draft.clone(),
        ));
    }
    let export_directory = host
        .config
        .capture
        .as_ref()
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf();
    let model = Rc::new(RefCell::new(Specimen {
        creator: host.creator.take().expect("bench has a creator"),
        volumes: host.volumes,
        epoch: 1,
        revision: 1,
        selected: None,
        yaw: 0.0,
        isolated: true,
        camera: host.config.camera,
        content: host.content,
        comparison: None,
    }));
    let mut generation = generation_controls::Controls::new(&model.borrow());
    if let Some(saved) = &restore {
        generation.size = saved.size;
        generation.base = saved
            .base_content
            .clone()
            .or_else(|| Some(saved.content.clone()));
    }
    let scene = Rc::new(RefCell::new(producer::BenchScene::new(model.clone())));
    let cards = (0..5)
        .map(|card| {
            Rc::new(RefCell::new(producer::BenchScene::for_card(
                model.clone(),
                card,
            )))
        })
        .collect();
    let exit_code = Rc::new(Cell::new(0));
    let lane = Rc::new(RefCell::new(
        probe::Lane::new(
            host.scenario.take(),
            host.config.receipt.clone(),
            host.config.capture.clone(),
            exit_code.clone(),
        )
        .with_frame_limit(host.config.frames),
    ));
    let close_lane = lane.clone();
    let hooks: HostHooks<Bench, Logic, Child> = HostHooks {
        frame: Box::new(|ctx| {
            if ctx.runner.state().effects.open && ctx.runner.state().effects.playing {
                ctx.runner.update(|s| s.effects.advance());
            }
            ctx.runner.state().effects.sync(ctx.leaves);
            let pending = ctx.runner.state().model.borrow().creator.pending;
            if pending {
                ctx.runner.update(|s| {
                    s.poll();
                });
            }
            let state = ctx.runner.state();
            if state.visible && !state.effects.open && !ctx.producers.contains(LEAF_KEY) {
                ctx.producers
                    .register(LEAF_KEY, state.scene.clone(), &["color"])
                    .expect("bench owns one producer key");
            }
            if state.visible && !state.effects.open {
                if let Some(comparison) = &state.model.borrow().comparison {
                    for (index, card) in comparison.cards.iter().enumerate() {
                        let key = LEAF_KEY + 1 + index as u64;
                        if card.world.is_some() && !ctx.producers.contains(key) {
                            ctx.producers
                                .register(key, state.cards[index].clone(), &["color"])
                                .expect("comparison owns its producer keys");
                        }
                    }
                }
            }
            state.model.borrow().creator.pending || (state.effects.open && state.effects.playing)
        }),
        after_dispatch: Box::new(|_| {}),
        after_frame: Box::new(move |ctx| {
            let error = ctx
                .runner
                .state()
                .scene
                .borrow()
                .error
                .clone()
                .or_else(|| {
                    ctx.producers
                        .error(LEAF_KEY)
                        .map(|why| format!("Viewport unavailable: {why:?}"))
                })
                .or_else(|| {
                    let state = ctx.runner.state();
                    let model = state.model.borrow();
                    let comparison = model.comparison.as_ref()?;
                    comparison
                        .cards
                        .iter()
                        .enumerate()
                        .filter(|(_, c)| c.world.is_some())
                        .find_map(|(index, _)| {
                            state.cards[index].borrow().error.clone().or_else(|| {
                                ctx.producers
                                    .error(LEAF_KEY + 1 + index as u64)
                                    .map(|why| format!("Alternative viewport: {why:?}"))
                            })
                        })
                });
            if ctx.runner.state().published_error != error {
                ctx.runner.update(|state| state.published_error = error);
                if let Some(window) = ctx.window {
                    window.request_redraw();
                }
            }
            lane.borrow_mut().after_frame(ctx);
        }),
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
    cambium_genet_winit_host::run(
        options,
        move |_, _, _| Init {
            state: Bench {
                model,
                scene,
                events: Vec::new(),
                notice: String::new(),
                published_error: None,
                tint: 0,
                decorated: false,
                visible: true,
                transformed: false,
                overlay_clicks: 0,
                cards,
                export_directory,
                restore,
                generation,
                effects: effect,
            },
            logic: view::root as Logic,
            sheet: view::SHEET.into(),
        },
        hooks,
    )?;
    Ok(exit_code.get())
}
