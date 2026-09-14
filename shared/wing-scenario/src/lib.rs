// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The generic scenario lane for a Cambium document host.
//!
//! Mesocosm's `--bench` grew the whole shape first: a [`genet_probe::Scenario`]
//! ticked once per *presented* frame inside `after_frame`, a deferred window
//! close that lets the last requested readback land, capture arming through the
//! host's own `read_frame`, PNG writing, viewport-masked pixel comparison,
//! bounded CPU cost accounting, and one JSON receipt carrying captures,
//! digests, snapshot fields, checkpoints, errors and misses. None of that is
//! about specimens. This crate is that lane with the product cut out of it.
//!
//! What stays the product's is everything that reads its own state: the typed
//! [`ProbeSnapshot`], its event stream, its `act` labels, which DOM element is
//! the viewport, where its default output directory is, and any verb its own
//! grammar adds. That is the [`Product`] trait — a small set of hooks a
//! consumer implements once, after which [`Lane`] runs the lifecycle.
//!
//! ```ignore
//! let mut lane = Lane::new(BenchProduct, scenario, receipt, capture, exit_code)
//!     .with_frame_limit(Some(1800));
//! // in HostHooks::after_frame
//! lane.after_frame(ctx);
//! // in HostHooks::close_request
//! lane.request_close();
//! ```

use std::path::PathBuf;

use cambium_rootstock::{AppCtx, NodeId, meristem_bounds::RootView};
use genet_probe::ProbeSnapshot;

mod checkpoints;
mod cost;
mod lane;
mod pixels;
mod probe;

pub use checkpoints::{Checkpoint, Checkpoints};
pub use cost::{CostObservation, Costs, Totals};
pub use lane::{Capture, Lane, capture_path};
pub use pixels::{PixelCheck, Viewport, ViewportTransform, create_parent, write_png};

/// The host context a [`Product`]'s hooks are handed, spelled once.
pub type Ctx<'a, P> =
    AppCtx<'a, <P as Product>::State, <P as Product>::Logic, <P as Product>::View>;

/// Everything the generic lane cannot know: the host's own types, and the
/// handful of readings that are about the product rather than about driving a
/// document host.
///
/// Every method except [`snapshot`](Product::snapshot),
/// [`drain_events`](Product::drain_events) and
/// [`default_capture_path`](Product::default_capture_path) has a defensible
/// default, so a new consumer starts with three implementations.
pub trait Product: Sized {
    /// The host application state the Cambium runner holds.
    type State: 'static;
    /// The view logic the runner diffs into a DOM.
    type Logic: FnMut(&Self::State) -> Self::View;
    /// The root view type that logic produces.
    type View: RootView<Self::State>;

    /// The receipt's `kind` field — what this lane is a receipt *of*.
    const KIND: &'static str;
    /// A stable surface name for hit attribution in probe's `Hit`.
    const SURFACE: &'static str;
    /// The prefix on every line the lane prints to stdout and stderr.
    const LOG_PREFIX: &'static str;

    /// The stylesheet the retained surface lays out under.
    fn sheet(&self) -> &'static str;

    /// A typed read of product state for assertions the DOM cannot express.
    /// `captures` is how many captures have completed; `opacity` is the value
    /// the `opacity` verb last set.
    fn snapshot(&self, ctx: &Ctx<'_, Self>, captures: usize, opacity: f32) -> ProbeSnapshot;

    /// Drain the semantic events emitted since the last call.
    fn drain_events(&mut self, ctx: &mut Ctx<'_, Self>) -> Vec<String>;

    /// Where captures land when a scenario names one without a path. Its file
    /// stem is also the fallback stem for generated capture names.
    fn default_capture_path(&self) -> PathBuf;

    /// Run one product-named command (the `act <label>` verb). `false` when no
    /// such command exists, so the driver fails loudly. The default refuses
    /// every label: a host whose controls are exercised *as controls*, through
    /// pointer delivery, wants exactly that.
    fn act(&mut self, ctx: &mut Ctx<'_, Self>, label: &str) -> bool {
        let _ = (ctx, label);
        false
    }

    /// Whether the product still has work the next scenario step must not
    /// race. `capture_pending` is the lane's own outstanding readback, which is
    /// always part of the answer. `None` means this product does not report
    /// quiescence; see [`genet_probe::Automatable::busy`].
    fn busy(&self, ctx: &Ctx<'_, Self>, capture_pending: bool) -> Option<bool> {
        let _ = (ctx, capture_pending);
        None
    }

    /// The product's rendered viewport, as a pixel mask for capture
    /// comparison. `None` disables pixel checks for that capture.
    fn viewport(&self, ctx: &Ctx<'_, Self>) -> Option<Viewport> {
        let _ = ctx;
        None
    }

    /// Where a click on a matched element should land, given its painted rect
    /// `[x, y, w, h]`. The default is the rect's centre; a product whose
    /// viewport is CSS-transformed maps the centre through that transform.
    fn target_point(&self, ctx: &Ctx<'_, Self>, node: NodeId, rect: [f32; 4]) -> (f32, f32) {
        let _ = (ctx, node);
        (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
    }

    /// The stylesheet to install for an `opacity <value>` verb. `None` records
    /// the value in the snapshot without restyling anything.
    fn opacity_sheet(&self, opacity: f32) -> Option<String> {
        let _ = opacity;
        None
    }

    /// This frame's cumulative redraw and upload totals, plus whether the frame
    /// is eligible for cost summaries and whether its scene is populated.
    fn cost_observation(&self, ctx: &Ctx<'_, Self>) -> CostObservation {
        let _ = ctx;
        CostObservation::default()
    }

    /// Handle a scenario verb the generic grammar did not recognize.
    /// `checkpoints` is the read-only view of what `remember` has taken, so a
    /// product verb compares against the same checkpoints the shared verbs do.
    fn app_step(
        &mut self,
        ctx: &mut Ctx<'_, Self>,
        checkpoints: Checkpoints<'_>,
        line: &str,
    ) -> Result<(), String> {
        let _ = (ctx, checkpoints);
        Err(format!("unknown scenario step: {line}"))
    }
}

#[cfg(test)]
mod tests;
