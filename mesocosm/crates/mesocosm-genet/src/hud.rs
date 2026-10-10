// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The minimap: a sprigging leaf over the native site graph, rasterized by
//! netrender on the game's own device. The legacy enclosure backdrop is gone:
//! a site graph has no single enclosure to photograph from above.
//!
//! Route A of the staged host ruling (2026-08-02): the leaf paints `PaintCmd`s,
//! `paint_list_render` lowers them, and [`crate::chrome`] rasterizes and
//! blends. **Painted, not textless** — the guard was lane discipline, not a
//! text ban (views founding plan §6, amended 2026-08-29). This lane draws
//! marks in the scene; words go through cambium, which
//! [`crate::vitals`] is the first consumer of. What the guard still forbids is
//! teaching this lane lettering as a shortcut.

use isocosm::schema::Id;
use isocosm::simulation::Simulation;
use mesocosm_views::MinimapLeaf;
use sprigging::{Leaf, PaintCx, Size};

use crate::chrome::{Chrome, Raster};

/// The minimap's square side, in pixels. Also the rasterized texture's size,
/// so the leaf paints at the resolution it is shown.
pub const SIDE: u32 = 160;

/// Distance from the frame's corner.
pub const MARGIN: f32 = 12.0;

pub struct Hud {
    leaf: MinimapLeaf,
    raster: Raster,
}

/// Where the minimap sits in a frame of this size.
pub fn placement(frame: (u32, u32)) -> (f32, f32, f32, f32) {
    let x = frame.0 as f32 - SIDE as f32 - MARGIN;
    (x.max(0.0), MARGIN, SIDE as f32, SIDE as f32)
}

impl Hud {
    pub fn new(chrome: &Chrome, sim: &Simulation, played: Option<Id>) -> Self {
        Self {
            leaf: mesocosm_views::minimap_leaf(sim, played),
            raster: Raster::new(chrome.device(), "minimap", SIDE, SIDE),
        }
    }

    /// Reprojects the world and rasterizes the minimap if anything changed.
    ///
    /// The leaf dedups identical projections, so an idle world costs a scene
    /// build and no raster.
    pub fn refresh(&mut self, chrome: &Chrome, sim: &Simulation, played: Option<Id>) {
        self.leaf
            .refresh_from(mesocosm_views::minimap_leaf(sim, played));

        if !self.leaf.paint_dirty() {
            return;
        }
        let mut cmds = Vec::new();
        let mut cx = PaintCx::new(
            &mut cmds,
            Size {
                width: SIDE as f32,
                height: SIDE as f32,
            },
        );
        self.leaf.paint(&mut cx);

        let translated = paint_list_render::translate_paint_cmd_stream(
            paint_list_api::DeviceIntSize::new(SIDE as i32, SIDE as i32),
            &cmds,
            &[],
            &[],
        );
        chrome.raster(&self.raster, &translated.scene);
    }

    /// Blends the minimap into the frame's top-right corner.
    pub fn composite(
        &self,
        chrome: &Chrome,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        let dest = placement(frame);
        chrome.draw(encoder, target, self.raster.sample_view(), dest, frame);
    }
}
