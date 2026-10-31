// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Square sites on a plane, a ring or a torus (ruling 395).

use super::{Laid, default_materials, default_skeleton};
use crate::{
    Result,
    schema::{Border, Footprint, Id, Key, Route, Site},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A grid of square sites. Site `row * width + column` has side 0 to the
/// north, 1 east, 2 south and 3 west; a ring wraps east to west, a torus
/// both ways, and a plane not at all.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grid {
    /// `shape:plane`, `shape:ring` or `shape:torus`.
    pub shape: Key,
    pub width: u32,
    pub height: u32,
    /// Each site's side, in base units.
    pub side: u64,
    /// The band site elevations are drawn in, in base units.
    pub elevation: [i64; 2],
    /// The band site relief is drawn in, in base units.
    pub relief: [i64; 2],
    /// The share of sites, per mille, lying below the water level.
    pub sea_per_mille: u32,
}

pub const SHAPES: [&str; 3] = ["shape:plane", "shape:ring", "shape:torus"];
/// Terrain values stay within this, so no sum of them can overflow.
const BAND: i64 = 1 << 40;

impl Grid {
    /// A grid from SP1's declared space (ruling 399): planes, rings and tori
    /// of 2 to 16 sites a side, sites of 256 to 2,048 base units, elevation
    /// within one side and relief within an eighth of one.
    pub fn drawn(seed: u64) -> Self {
        let pick = |domain: &str| crate::draw(seed, domain, &[]);
        let side = 16 * (16 + pick("map:side") % 113);
        Self {
            shape: SHAPES[(pick("map:shape") % 3) as usize].into(),
            width: 2 + (pick("map:width") % 15) as u32,
            height: 2 + (pick("map:height") % 15) as u32,
            side,
            elevation: [0, side as i64],
            relief: [0, side as i64 / 8],
            sea_per_mille: (pick("map:sea") % 1001) as u32,
        }
    }

    pub(super) fn lay(&self, seed: u64, sites: u32) -> Result<Laid> {
        if !SHAPES.contains(&self.shape.as_str())
            || !(2..=16).contains(&self.width)
            || !(2..=16).contains(&self.height)
            || !(16..=1 << 20).contains(&self.side)
            || self.elevation[0] > self.elevation[1]
            || self.relief[0] < 0
            || self.relief[0] > self.relief[1]
            || self.elevation[0] < -BAND
            || self.elevation[1] > BAND
            || self.relief[1] > BAND
            || self.sea_per_mille > 1000
        {
            return Err("map parameters outside declared generator domain".into());
        }
        if u64::from(sites) != u64::from(self.width) * u64::from(self.height) {
            return Err("a map's sites disagree with its founding's".into());
        }
        let skeleton = default_skeleton();
        let elevation = self.smoothed(seed, "map:elevation", self.elevation);
        let relief = self.smoothed(seed, "map:relief", self.relief);
        let water = sea_level(&elevation, self.sea_per_mille);
        let draw = |domain: &str, values: &[u64]| crate::draw(seed, domain, values);
        let mut laid = BTreeMap::new();
        for (id, (column, row)) in self.cells().enumerate() {
            let id = id as Id;
            let routes = (0..4u8)
                .filter_map(|side| {
                    let (c, r) = self.across(column, row, side)?;
                    let to = self.id(c, r);
                    (to != id).then(|| Route {
                        to,
                        travel: 1 + draw("map:travel", &[id, side.into()]) % 4,
                        transmission: 500_000
                            + (draw("map:transmission", &[id, side.into()]) % 500_001) as u32,
                        border: Some(Border {
                            side,
                            enters: (side + 2) % 4,
                            flipped: false,
                        }),
                        authored: None,
                    })
                })
                .collect();
            let conditions = BTreeMap::from([
                ("world:habitable".into(), 1),
                ("world:weather".into(), 0),
                (skeleton.elevation.clone(), elevation[id as usize]),
                (skeleton.relief.clone(), relief[id as usize]),
                (skeleton.water.clone(), water),
            ]);
            let site = Site {
                terrain_seed: draw("terrain", &[id]),
                accounts: BTreeMap::new(),
                conditions,
                routes,
                authored: None,
            };
            laid.insert(id, site);
        }
        Ok(Laid {
            sites: laid,
            conditions: BTreeSet::from([
                skeleton.elevation.clone(),
                skeleton.relief.clone(),
                skeleton.water.clone(),
            ]),
            skeleton,
            footprint: Footprint {
                sides: 4,
                side: self.side,
            },
            shape: self.shape.clone(),
            materials: default_materials(),
        })
    }

    /// Every (column, row), in site-id order.
    fn cells(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        (0..self.height).flat_map(move |row| (0..self.width).map(move |column| (column, row)))
    }

    /// The column and row of `site`, if the grid holds it.
    pub fn at(&self, site: Id) -> Option<[u32; 2]> {
        let w = u64::from(self.width);
        (site < w * u64::from(self.height)).then(|| [(site % w) as u32, (site / w) as u32])
    }

    fn id(&self, column: u32, row: u32) -> Id {
        u64::from(row) * u64::from(self.width) + u64::from(column)
    }

    /// The site across `side` of the site at (column, row), if the shape
    /// has one there.
    fn across(&self, column: u32, row: u32, side: u8) -> Option<(u32, u32)> {
        let (wrap_x, wrap_z) = match self.shape.as_str() {
            "shape:ring" => (true, false),
            "shape:torus" => (true, true),
            _ => (false, false),
        };
        let (w, h) = (i64::from(self.width), i64::from(self.height));
        let (mut c, mut r) = (i64::from(column), i64::from(row));
        match side {
            0 => r -= 1,
            1 => c += 1,
            2 => r += 1,
            _ => c -= 1,
        }
        if wrap_x {
            c = c.rem_euclid(w);
        }
        if wrap_z {
            r = r.rem_euclid(h);
        }
        ((0..w).contains(&c) && (0..h).contains(&r)).then_some((c as u32, r as u32))
    }

    /// A value per site drawn in `band`, then averaged once with the sites
    /// around it, so neighbours are related: the skeleton laid top-down.
    fn smoothed(&self, seed: u64, domain: &str, band: [i64; 2]) -> Vec<i64> {
        let span = (band[1] - band[0]) as u64 + 1;
        let raw: Vec<i64> = self
            .cells()
            .map(|(c, r)| band[0] + (crate::draw(seed, domain, &[self.id(c, r)]) % span) as i64)
            .collect();
        self.cells()
            .map(|(c, r)| {
                let (mut sum, mut n) = (raw[self.id(c, r) as usize], 1);
                for side in 0..4 {
                    if let Some((c2, r2)) = self.across(c, r, side) {
                        sum += raw[self.id(c2, r2) as usize];
                        n += 1;
                    }
                }
                sum / n
            })
            .collect()
    }
}

/// The elevation below which `per_mille` of the sites lie.
fn sea_level(elevations: &[i64], per_mille: u32) -> i64 {
    let mut sorted = elevations.to_vec();
    sorted.sort_unstable();
    let at = (sorted.len() * per_mille as usize / 1000).min(sorted.len() - 1);
    sorted[at]
}
