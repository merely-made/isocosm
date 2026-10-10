// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Edits as shape operations (the place-graph plan's SP4, rulings 412 to
//! 416). An edit carves to air or fills with a named material over a sphere,
//! a box or a route, in base units in the frame of the site it was made in.
//! The product keeps each one as an asserted fact, in order (ruling 696);
//! this module replays them onto chunks and counts what each one moves.
//!
//! A cell is inside a shape when its centre is, so every test is exact in
//! doubled integer coordinates and a shape read across a border through the
//! frame relation (ruling 414) holds exactly the cells it held at home.

mod replay;
mod shape;
#[cfg(test)]
mod tests;

pub use replay::{Moved, reaching, tally};
pub(crate) use replay::{SiteRule, lift_edited, touched};
pub(crate) use shape::runs;

use crate::{Border, Footprint, Result};
use serde::{Deserialize, Serialize};

/// What an edit does to the cells inside its shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    /// To air.
    Carve,
    /// With the product's material id.
    Fill(u8),
}

/// A region of cells, in base units in its site's frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shape {
    /// The cells whose centres lie within `radius` of `centre`.
    Sphere { centre: [i64; 3], radius: u64 },
    /// The cells from `min` up to but excluding `max`.
    Box { min: [i64; 3], max: [i64; 3] },
    /// The cells within `radius` of the path through `points`, in order.
    Route { points: Vec<[i64; 3]>, radius: u64 },
}

/// One edit: an operation over a shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    pub op: Op,
    pub shape: Shape,
}

/// The most points a route may carry.
pub const ROUTE_POINTS: usize = 256;

impl Shape {
    /// The cells it can touch, from `lo` up to but excluding `hi`.
    pub fn extent(&self) -> [[i64; 3]; 2] {
        let around = |p: [i64; 3], r: u64| {
            let r = r as i64;
            [p.map(|c| c - r), p.map(|c| c + r)]
        };
        match self {
            Shape::Sphere { centre, radius } => around(*centre, *radius),
            Shape::Box { min, max } => [*min, *max],
            Shape::Route { points, radius } => {
                let mut out = [[i64::MAX; 3], [i64::MIN; 3]];
                for &p in points {
                    let [lo, hi] = around(p, *radius);
                    for k in 0..3 {
                        out[0][k] = out[0][k].min(lo[k]);
                        out[1][k] = out[1][k].max(hi[k]);
                    }
                }
                out
            },
        }
    }

    /// The same cells read in the neighbour's frame across `border`, the
    /// border's length `side` (squares only, ruling 395).
    pub fn across(&self, border: Border, side: u64) -> Shape {
        let map = |p: [i64; 3]| across(p, border, side as i64);
        match self {
            Shape::Sphere { centre, radius } => Shape::Sphere {
                centre: map(*centre),
                radius: *radius,
            },
            Shape::Box { min, max } => {
                let (a, b) = (map(*min), map(*max));
                Shape::Box {
                    min: [0, 1, 2].map(|k| a[k].min(b[k])),
                    max: [0, 1, 2].map(|k| a[k].max(b[k])),
                }
            },
            Shape::Route { points, radius } => Shape::Route {
                points: points.iter().map(|&p| map(p)).collect(),
                radius: *radius,
            },
        }
    }
}

impl Edit {
    /// Refuses an edit the lift cannot read: an empty route or one with too
    /// many points, a box turned inside out, or a shape spanning a whole
    /// site side, which would reach past its immediate neighbours.
    pub fn check(&self, footprint: Footprint) -> Result<()> {
        if footprint.sides != 4 {
            return Err("edits need square footprints".into());
        }
        match &self.shape {
            Shape::Route { points, .. } if points.is_empty() || points.len() > ROUTE_POINTS => {
                return Err("a route of no points or too many".into());
            },
            Shape::Box { min, max } if (0..3).any(|k| min[k] > max[k]) => {
                return Err("a box turned inside out".into());
            },
            _ => {},
        }
        let [lo, hi] = self.shape.extent();
        if (0..3).any(|k| hi[k] - lo[k] >= footprint.side as i64) {
            return Err("an edit spanning a whole site side".into());
        }
        Ok(())
    }

    pub fn across(&self, border: Border, side: u64) -> Edit {
        Edit {
            op: self.op,
            shape: self.shape.across(border, side),
        }
    }
}

/// Where each side of a square starts, which way it runs, and which way is
/// inward, in `(x, z)`.
fn frame(k: u8, side: i64) -> ([i64; 2], [i64; 2], [i64; 2]) {
    match k % 4 {
        0 => ([0, 0], [1, 0], [0, 1]),
        1 => ([side, 0], [0, 1], [-1, 0]),
        2 => ([side, side], [-1, 0], [0, -1]),
        _ => ([0, side], [0, -1], [1, 0]),
    }
}

/// A point of one site's frame in its neighbour's, across `border`: as far
/// along the shared side, run the other way unless flipped, and as far past
/// it as it was inside.
fn across(p: [i64; 3], border: Border, side: i64) -> [i64; 3] {
    let (c, d, n) = frame(border.side, side);
    let rel = [p[0] - c[0], p[2] - c[1]];
    let t = rel[0] * d[0] + rel[1] * d[1];
    let u = rel[0] * n[0] + rel[1] * n[1];
    let t = if border.flipped { t } else { side - t };
    let (c, d, n) = frame(border.enters, side);
    [
        c[0] + t * d[0] - u * n[0],
        p[1],
        c[1] + t * d[1] - u * n[1],
    ]
}
