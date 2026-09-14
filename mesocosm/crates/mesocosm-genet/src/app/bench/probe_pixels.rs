// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which element is the bench's viewport, and how a click reaches it. The mask
//! arithmetic and the decoded-PNG comparison are `mesquite::pixels`; what
//! is here is the authored fixture those generic checks are run over.
//!
//! The transform below is restated from `view.rs` on purpose: a check that asks
//! Genet's own transform implementation where a pixel went proves nothing.

use mesquite::{Viewport, ViewportTransform};
use taproot::Selector;

use super::Context;

/// Symmetric 3px border + 8px padding, plus 2px of antialiased edge discarded.
const INSET: f32 = 13.0;

/// `view.rs` authors `rotate(7deg) scale(0.9)`, origin `25% 75%`.
const TRANSFORM: ViewportTransform = ViewportTransform {
    rotate_deg: 7.0,
    scale: 0.9,
    origin: [0.25, 0.75],
};

pub(super) fn viewport(ctx: &Context<'_>) -> Option<Viewport> {
    let dom = ctx.runner.dom();
    let dom = dom.borrow();
    let rect = |selector: Selector| {
        taproot::matching(&dom, &selector)
            .into_iter()
            .find_map(|node| ctx.painted_rect(node))
            .map(|(x, y, w, h)| [x, y, w, h])
    };
    let pixel_scale = ctx.window.map_or(1.0, |w| w.scale_factor()) as f32 * ctx.ui_zoom;
    if let Some(border) = rect(Selector::role("img").containing("Glyph effect experiment")) {
        return Some(Viewport::new(border, pixel_scale, INSET));
    }
    let border = rect(Selector::role("img").containing("Specimen"))?;
    (border[2] > 0.0 && border[3] > 0.0).then(|| {
        Viewport::new(border, pixel_scale, INSET)
            .with_overlay(rect(Selector::role("button").containing("Clear selection")))
            .with_transform(ctx.runner.state().transformed.then_some(TRANSFORM))
    })
}

pub(super) fn target_point(
    ctx: &Context<'_>,
    node: genet_scripted_dom::NodeId,
    border: [f32; 4],
) -> (f32, f32) {
    let point = (border[0] + border[2] * 0.5, border[1] + border[3] * 0.5);
    let specimen = taproot::matching(
        &ctx.runner.dom().borrow(),
        &Selector::role("img").containing("Specimen"),
    )
    .contains(&node);
    if specimen && let Some(viewport) = viewport(ctx) {
        return viewport.map(point, false);
    }
    point
}
