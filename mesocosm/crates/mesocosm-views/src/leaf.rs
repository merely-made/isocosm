// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The minimap paint leaf: a solved hulls scene, realized.
//!
//! A sprigging [`Leaf`], so any cambium host can splice it into a paint list
//! and any test can read its commands back without a GPU. It paints what it is
//! handed and computes nothing: cells come solved from `scenomise`, tints come
//! resolved from the adapter, and the leaf's only arithmetic is mapping scene
//! units onto its own pixels.

use sceno::{Footprint, Scene, Vec2};
use sprigging::{ColorF, Leaf, PaintCx, Path, Size, SizeHint, solid_stroke};

/// How much of a cell's tint survives as fill. The boundary keeps full
/// strength; the interior stays translucent so a backdrop reads through,
/// which is what lets the world's own image sit underneath.
const FILL_ALPHA: f32 = 0.35;

/// The played critter's marker radius, in leaf pixels.
const MARKER_RADIUS: f32 = 4.0;

/// What a held surface leaves at a site: a ring on the ground, and a diamond
/// where bodies carry it, each as a strength from nought to one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoldMark {
    /// The site, in scene units.
    pub at: (f32, f32),
    pub land: f32,
    pub hosts: f32,
}

/// The ring's colour (amber) and the diamond's (cyan), apart from every
/// lineage tint's muted range and from the white player mark.
const LAND_COLOUR: (f32, f32, f32) = (0.96, 0.62, 0.16);
const HOST_COLOUR: (f32, f32, f32) = (0.25, 0.85, 0.9);

/// A realized minimap: regions tinted by holder, sites dotted, held surfaces
/// ringed, the player marked.
pub struct MinimapLeaf {
    scene: Scene,
    /// One tint per scene region, adapter-resolved, in region order. `None`
    /// is an unheld region, painted as boundary only.
    tints: Vec<Option<ColorF>>,
    /// The played critter's world x/z, if anyone is embodied.
    player: Option<(f32, f32)>,
    /// Where held surfaces lie; empty when nothing is held.
    holds: Vec<HoldMark>,
    size: Size,
    dirty: bool,
}

impl MinimapLeaf {
    pub fn new(scene: Scene, tints: Vec<Option<ColorF>>, player: Option<(f32, f32)>) -> Self {
        Self {
            scene,
            tints,
            player,
            holds: Vec::new(),
            size: Size {
                width: 160.0,
                height: 160.0,
            },
            dirty: true,
        }
    }

    /// The same minimap with the marks of held surfaces.
    pub fn with_holds(mut self, holds: Vec<HoldMark>) -> Self {
        self.holds = holds;
        self
    }

    pub fn holds(&self) -> &[HoldMark] { &self.holds }

    /// Replaces the projection wholesale. The next paint repaints everything;
    /// diffing a nine-region minimap would cost more than it saves.
    pub fn update(&mut self, scene: Scene, tints: Vec<Option<ColorF>>, player: Option<(f32, f32)>,
        holds: Vec<HoldMark>) {
        if self.scene == scene && self.tints == tints && self.player == player && self.holds == holds {
            return;
        }
        self.scene = scene;
        self.tints = tints;
        self.player = player;
        self.holds = holds;
        self.dirty = true;
    }

    /// Adopts a freshly assembled projection, keeping this leaf's paint
    /// retention: identical projections stay settled.
    pub fn refresh_from(&mut self, fresh: MinimapLeaf) {
        self.update(fresh.scene, fresh.tints, fresh.player, fresh.holds);
    }

    /// Scene units to leaf pixels: fit the scene bounds into the leaf,
    /// preserving aspect.
    fn to_px(&self, p: Vec2) -> (f32, f32) {
        let bounds = self.scene.bounds;
        let scale = (self.size.width / bounds.size.w.max(1.0))
            .min(self.size.height / bounds.size.h.max(1.0));
        (
            (p.x - bounds.origin.x) * scale,
            (p.y - bounds.origin.y) * scale,
        )
    }

    fn polygon_path(&self, points: &[Vec2]) -> Option<sprigging::PathData> {
        let (first, rest) = points.split_first()?;
        let (x, y) = self.to_px(*first);
        let mut path = Path::new().move_to(x, y);
        for p in rest {
            let (x, y) = self.to_px(*p);
            path = path.line_to(x, y);
        }
        Some(path.close().build())
    }
}

impl Leaf for MinimapLeaf {
    fn measure(&mut self, known: SizeHint, available: SizeHint) -> Size {
        // Square, sized to whatever the host offers, defaulting modest. A
        // minimap earns corner space, not a panel.
        let side = known
            .width
            .or(known.height)
            .or_else(|| {
                available
                    .width
                    .map(|w| w.min(available.height.unwrap_or(w)))
            })
            .unwrap_or(160.0);
        self.size = Size {
            width: side,
            height: side,
        };
        self.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.size = cx.size();

        for (index, region) in self.scene.regions.iter().enumerate() {
            let Some(Footprint::Polygon { points }) = &region.contour else {
                continue;
            };
            let Some(path) = self.polygon_path(points) else {
                continue;
            };
            let tint = self.tints.get(index).copied().flatten();
            let fill = tint.map(|t| ColorF::new(t.r, t.g, t.b, t.a * FILL_ALPHA));
            let edge = tint.unwrap_or(ColorF::new(0.5, 0.5, 0.5, 1.0));
            cx.path(path, fill, Some(solid_stroke(edge, 1.0)));
        }

        // Sites, over the cells they own.
        for item in &self.scene.items {
            if !item.visible {
                continue;
            }
            let (x, y) = self.to_px(Vec2::new(
                item.transform.translate.x,
                item.transform.translate.y,
            ));
            cx.fill_path(Path::circle(x, y, 1.5), ColorF::new(0.85, 0.85, 0.85, 0.9));
        }

        // Held surfaces, over the sites: a ring for matter on the ground and
        // a diamond beside it for bodies that carry it.
        for mark in &self.holds {
            let (x, y) = self.to_px(Vec2::new(mark.at.0, mark.at.1));
            if mark.land > 0.0 {
                let (r, g, b) = LAND_COLOUR;
                cx.path(Path::circle(x, y, 3.0 + 5.0 * mark.land), None,
                    Some(solid_stroke(ColorF::new(r, g, b, mark.land), 2.0)));
            }
            if mark.hosts > 0.0 {
                let (r, g, b) = HOST_COLOUR;
                let size = 2.5 + 2.5 * mark.hosts;
                let (cx0, cy0) = (x + 6.0, y - 6.0);
                let diamond = Path::new().move_to(cx0, cy0 - size).line_to(cx0 + size, cy0)
                    .line_to(cx0, cy0 + size).line_to(cx0 - size, cy0).close().build();
                cx.fill_path(diamond, ColorF::new(r, g, b, mark.hosts));
            }
        }

        // The player, over everything: the one mark that answers "where am I".
        if let Some((wx, wz)) = self.player {
            let (x, y) = self.to_px(Vec2::new(wx, wz));
            cx.fill_path(
                Path::circle(x, y, MARKER_RADIUS),
                ColorF::new(1.0, 1.0, 1.0, 1.0),
            );
        }

        self.dirty = false;
    }

    fn paint_dirty(&self) -> bool {
        self.dirty
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(holds: Vec<HoldMark>) -> MinimapLeaf {
        let score = crate::minimap::minimap_score_of(&[(1, Vec2::new(0.0, 0.0))]);
        MinimapLeaf::new(scenomise::solve(&score), vec![None], None).with_holds(holds)
    }

    fn painted(leaf: &mut MinimapLeaf) -> usize {
        let mut cmds = Vec::new();
        let mut cx = PaintCx::new(&mut cmds, Size { width: 160.0, height: 160.0 });
        leaf.paint(&mut cx);
        cmds.len()
    }

    #[test]
    fn held_marks_add_paint_and_a_changed_mark_repaints() {
        let bare = painted(&mut leaf(vec![]));
        let ring = HoldMark { at: (0.0, 0.0), land: 0.5, hosts: 0.0 };
        let both = HoldMark { hosts: 1.0, ..ring };
        let with_ring = painted(&mut leaf(vec![ring]));
        let with_both = painted(&mut leaf(vec![both]));
        assert!(with_ring > bare && with_both > with_ring, "{bare} {with_ring} {with_both}");
        let mut settled = leaf(vec![ring]);
        painted(&mut settled);
        assert!(!settled.paint_dirty());
        settled.refresh_from(leaf(vec![ring]));
        assert!(!settled.paint_dirty(), "an identical projection stays settled");
        settled.refresh_from(leaf(vec![both]));
        assert!(settled.paint_dirty(), "a new mark repaints");
    }
}
