// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Which element is the session's viewport. The mask arithmetic and the
//! decoded-PNG comparison are `mesquite::pixels`; what is here is the
//! authored fixture those generic checks run over.
//!
//! `view.rs` gives the custom leaf `role="img"` and `aria-label="Scene"`, which
//! is the stable label this selector and the smoke lane both address it by. The
//! leaf carries no border, padding or transform of its own — the 3px border and
//! 8px padding belong to its `.scene-card` parent — so the only inset is the
//! antialiased rim.

use mesquite::Viewport;
use taproot::Selector;

use super::Context;

/// Two logical pixels of antialiased leaf edge, discarded.
const INSET: f32 = 2.0;

pub(super) fn viewport(ctx: &Context<'_>) -> Option<Viewport> {
    let dom = ctx.runner.dom();
    let node = taproot::matching(&dom.borrow(), &Selector::role("img").containing("Scene"))
        .into_iter()
        .next()?;
    let (x, y, width, height) = ctx.painted_rect(node)?;
    let pixel_scale = ctx.window.map_or(1.0, |window| window.scale_factor()) as f32 * ctx.ui_zoom;
    (width > 0.0 && height > 0.0).then(|| Viewport::new([x, y, width, height], pixel_scale, INSET))
}
