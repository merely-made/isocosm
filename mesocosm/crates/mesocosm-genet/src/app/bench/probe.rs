// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native bench acceptance, as a thin adapter over `mesquite`.
//!
//! The lane lifecycle — scenario ticking per presented frame, the frame limit,
//! the deferred close, capture arming through `read_frame`, PNG writing, the
//! receipt, pixel checks and cost accounting — moved to the shared crate on
//! 2026-09-14 (the document-host plan's P3a, ruled the day before). What stays
//! here is what is about *this* bench: its snapshot fields, its viewport
//! element, its stylesheet, its default output directory and its cost scenes.
//! Controls still use the host's actual pointer routing; captures still read
//! the presented document, including its embedded depth scene.

use std::{cell::Cell, path::PathBuf, rc::Rc};

use cambium_genet_winit_host::AppCtx;
use mesquite::{CostObservation, Viewport};
use taproot::{ProbeSnapshot, Scenario};

use super::{
    state::Bench,
    view::{Child, Logic, SHEET},
};

#[path = "probe_cost.rs"]
mod cost;
#[path = "probe_pixels.rs"]
mod pixels;
#[path = "probe_state.rs"]
mod state;

pub(super) type Context<'a> = AppCtx<'a, Bench, Logic, Child>;

/// The bench's answers to the shared lane's questions.
struct BenchProduct;

impl mesquite::Product for BenchProduct {
    type State = Bench;
    type Logic = Logic;
    type View = Child;

    const KIND: &'static str = "native-specimen-bench";
    const SURFACE: &'static str = "specimen-bench";
    const LOG_PREFIX: &'static str = "bench";

    fn sheet(&self) -> &'static str {
        SHEET.as_str()
    }

    fn snapshot(&self, ctx: &Context<'_>, captures: usize, opacity: f32) -> ProbeSnapshot {
        state::snapshot(ctx, captures, opacity)
    }

    fn drain_events(&mut self, ctx: &mut Context<'_>) -> Vec<String> {
        let mut events = Vec::new();
        ctx.runner
            .update(|state| events = std::mem::take(&mut state.events));
        events
    }

    fn default_capture_path(&self) -> PathBuf {
        crate::played::default_out_dir().join("specimen-bench.png")
    }

    fn busy(&self, ctx: &Context<'_>, capture_pending: bool) -> Option<bool> {
        let state = ctx.runner.state();
        let model = state.model.borrow();
        let scene = state.scene.borrow();
        Some(
            model.creator.pending
                || capture_pending
                || (state.visible
                    && !state.effects.open
                    && scene.section.is_none()
                    && scene.population_stats.is_none()
                    && scene.error.is_none()),
        )
    }

    fn viewport(&self, ctx: &Context<'_>) -> Option<Viewport> {
        pixels::viewport(ctx)
    }

    fn target_point(
        &self,
        ctx: &Context<'_>,
        node: genet_scripted_dom::NodeId,
        rect: [f32; 4],
    ) -> (f32, f32) {
        pixels::target_point(ctx, node, rect)
    }

    fn opacity_sheet(&self, opacity: f32) -> Option<String> {
        Some(format!(
            "{}\n#specimen-viewport {{ opacity:{opacity}; }}",
            SHEET.as_str()
        ))
    }

    fn cost_observation(&self, ctx: &Context<'_>) -> CostObservation {
        cost::observation(ctx)
    }

    fn app_step(
        &mut self,
        _ctx: &mut Context<'_>,
        _checkpoints: mesquite::Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        Err(format!("unknown bench step: {line}"))
    }
}

/// The bench's scenario lane. The API `bench.rs` drives is unchanged; the body
/// is `mesquite::Lane`.
pub(super) struct Lane(mesquite::Lane<BenchProduct>);

impl Lane {
    pub fn new(
        scenario: Option<Scenario>,
        receipt: Option<PathBuf>,
        final_capture: Option<PathBuf>,
        exit_code: Rc<Cell<i32>>,
    ) -> Self {
        Self(mesquite::Lane::new(
            BenchProduct,
            scenario,
            receipt,
            final_capture,
            exit_code,
        ))
    }

    /// Bounds scenario work. An already requested readback/final capture may
    /// use the following frame before the host closes.
    pub fn with_frame_limit(self, limit: Option<u32>) -> Self {
        Self(self.0.with_frame_limit(limit))
    }

    /// Defer native close until the last requested frame and receipt are saved.
    pub fn request_close(&mut self) {
        self.0.request_close();
    }

    pub fn after_frame(&mut self, ctx: &mut Context<'_>) {
        self.0.after_frame(ctx);
    }
}
