// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The minimap adapter: who holds where, disclosed as a score.
//!
//! # The cells are simulation truth
//!
//! `Places::at` assigns a position to its nearest site; `Arrangement::Hulls`
//! partitions by the same rule. So the cells a solved minimap scene draws are
//! **exactly** the regions the world itself reasons with, not a cartographer's
//! approximation of them. A congruence test holds the two rules together.
//!
//! # Dominance is derived, never stored
//!
//! Which lineage holds a region is read off the living roster at projection
//! time, the same discipline as capability, temperament, and the possibility
//! space: a stored verdict drifts from the facts it was drawn from, and a
//! derived one cannot.

use std::collections::BTreeMap;

use isocosm::schema::{Id, Key};
use isocosm::simulation::Simulation;
use sceno::{
    Arrangement, Footprint, Hulls, Placement, Rect, Representation, Scene, Score, ScoreItem, Size2,
    SourceRef, Vec2,
};
use sprigging::ColorF;

pub const MINIMAP_ADAPTER: &str = "mesocosm";

/// The map's half extent, in its own units.
const EXTENT: f32 = 64.0;

/// Where each site sits on the map: its column and row where the founding
/// laid a map (783), scaled into the map's extent; on a ring in site order
/// otherwise.
pub fn site_points(sim: &Simulation) -> BTreeMap<Id, Vec2> {
    let sites = &sim.state().sites;
    let laid: BTreeMap<Id, [u32; 2]> = sites
        .keys()
        .filter_map(|id| isocosm::map::position(sim.genesis(), *id).map(|at| (*id, at)))
        .collect();
    if laid.len() == sites.len() && !laid.is_empty() {
        let across = |k: usize| laid.values().map(|at| at[k]).max().unwrap_or(0) as f32 + 1.0;
        let (w, h) = (across(0), across(1));
        let step = (EXTENT * 2.0) / w.max(h);
        return laid
            .into_iter()
            .map(|(id, [c, r])| {
                let x = (c as f32 + 0.5) * step - step * w / 2.0;
                let y = (r as f32 + 0.5) * step - step * h / 2.0;
                (id, Vec2::new(x, y))
            })
            .collect();
    }
    let n = sites.len().max(1) as f32;
    sites
        .keys()
        .enumerate()
        .map(|(i, id)| {
            let a = std::f32::consts::TAU * i as f32 / n;
            (*id, Vec2::new(a.cos() * EXTENT * 0.7, a.sin() * EXTENT * 0.7))
        })
        .collect()
}

pub fn minimap_score(sim: &Simulation) -> Score {
    let mut score = Score::new(Arrangement::Hulls(Hulls {
        origin: Vec2::ZERO,
        units_per_coordinate: 1.0,
        invert_y: false,
        bounds: Rect::new(Vec2::new(-EXTENT, -EXTENT), Size2::new(EXTENT * 2.0, EXTENT * 2.0)),
    }));
    for (id, at) in site_points(sim) {
        score.items.push(ScoreItem {
            source: SourceRef::new(MINIMAP_ADAPTER, format!("site:{id}")),
            ordinal: id as u32,
            footprint: Footprint::Point,
            representation: Representation::Glyph,
            placement: Placement::Coordinate(at),
            layer: 0,
            visible: true,
            axis: None,
            embedding: None,
            weight: None,
        });
    }
    score
}

pub fn minimap_scene(sim: &Simulation) -> Scene {
    scenomise::solve(&minimap_score(sim))
}

/// Each site's most numerous living lineage, ties to the lower key.
pub fn dominant_lineages(sim: &Simulation) -> BTreeMap<Id, Option<Key>> {
    let s = sim.state();
    let mut count: BTreeMap<(Id, Key), u64> = BTreeMap::new();
    for group in s.population.groups.values().filter(|g| g.entity.alive) {
        let e = &group.entity;
        *count.entry((e.place, e.lineage.clone())).or_default() += group.count;
    }
    let mut holders: BTreeMap<Id, Option<Key>> = s.sites.keys().map(|id| (*id, None)).collect();
    let mut best: BTreeMap<Id, u64> = BTreeMap::new();
    for ((site, lineage), n) in count {
        if n > best.get(&site).copied().unwrap_or(0) {
            best.insert(site, n);
            holders.insert(site, Some(lineage));
        }
    }
    holders
}

pub fn lineage_tint(lineage: &str) -> ColorF {
    let n = lineage.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)));
    let hue = (n % 360) as f32 * 137.507_77 % 360.0;
    let (r, g, b) = hsl(hue, 0.45, 0.55);
    ColorF::new(r, g, b, 1.0)
}

fn hsl(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0..60 => (c, x, 0.0),
        60..120 => (x, c, 0.0),
        120..180 => (0.0, c, x),
        180..240 => (0.0, x, c),
        240..300 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

/// The whole minimap, assembled: solved scene, resolved tints, player mark.
///
/// The one call a host makes per refresh. Tint resolution walks scene regions
/// back through their member's source ref, so the leaf never learns what a
/// place is.
pub fn minimap_leaf(sim: &Simulation, played: Option<Id>) -> crate::leaf::MinimapLeaf {
    let scene = minimap_scene(sim);
    let holders = dominant_lineages(sim);
    let points = site_points(sim);
    let tints = scene
        .regions
        .iter()
        .map(|region| {
            let item = &scene.items[region.members[0].0 as usize];
            let source = &scene.sources[item.source.0 as usize];
            let id: Id = source.id.trim_start_matches("site:").parse().expect("minimap sources are sites");
            holders.get(&id).cloned().flatten().map(|l| lineage_tint(&l))
        })
        .collect();
    let pop = &sim.state().population;
    let player = played
        .and_then(|c| pop.get(c))
        .and_then(|e| points.get(&e.place))
        .map(|p| (p.x, p.y));
    crate::leaf::MinimapLeaf::new(scene, tints, player)
}
