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

mod probe;
mod producer;
mod state;
mod view;

use state::{Bench, Specimen};
use view::{Child, Logic};

const LEAF_KEY: u64 = 0x5350_4543;

/// Runs the bench with the creator flags already admitted by the ordinary CLI.
pub fn run(mut config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
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
    let model = Rc::new(RefCell::new(Specimen {
        creator: host.creator.take().expect("bench has a creator"),
        volumes: host.volumes,
        epoch: 1,
        revision: 1,
        selected: None,
        yaw: 0.0,
        isolated: true,
        camera: host.config.camera,
    }));
    let scene = Rc::new(RefCell::new(producer::BenchScene::new(model.clone())));
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
            let pending = ctx.runner.state().model.borrow().creator.pending;
            if pending {
                ctx.runner.update(|s| {
                    s.poll();
                });
            }
            let state = ctx.runner.state();
            if state.visible && !ctx.producers.contains(LEAF_KEY) {
                ctx.producers
                    .register(LEAF_KEY, state.scene.clone(), &["color"])
                    .expect("bench owns one producer key");
            }
            state.model.borrow().creator.pending
        }),
        after_dispatch: Box::new(|_| {}),
        after_frame: Box::new(move |ctx| {
            let error = ctx.runner.state().scene.borrow().error.clone().or_else(|| {
                ctx.producers
                    .error(LEAF_KEY)
                    .map(|why| format!("Viewport unavailable: {why:?}"))
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
            },
            logic: view::root as Logic,
            sheet: view::SHEET.into(),
        },
        hooks,
    )?;
    Ok(exit_code.get())
}
