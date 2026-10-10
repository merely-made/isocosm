// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A border read cell by cell (the place-graph plan's A.2, item 3): whether
//! a walker crosses at each base cell along it, and so whether each of the
//! profile's spans passes, climbs or stops. Derived from the lift on both
//! sides, never drawn, so a class never disagrees with the volume: a span
//! passes where some cell crosses dry within the world's climb, climbs
//! where every dry cell is steeper, and stops where water lies at every
//! cell. A span stops at a cliff where the lift drew one (ruling 744): a
//! step of [`CLIFF`] or more, which no walker in the volume steps.

use crate::edit::SiteRule;
use crate::lift::CLIFF;
use crate::places::Climb;
use crate::{Atlas, Border, Result, SPANS, SiteId};
use serde::{Deserialize, Serialize};

/// How a walker meets a border at one cell, or along one span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Span {
    /// Dry, within the world's climb.
    Passes,
    /// Dry, with a step past the climb: the least such step.
    Climbs { step: u64 },
    /// Water on one side or both, or a cliff: a step of [`CLIFF`] or more.
    Stops,
}

/// The base column just inside `side`, `t` cells along it from its corner.
pub fn edge_cell(side: u8, t: i64, length: i64) -> [i64; 2] {
    let (c, d, n) = crate::edit::frame(side, 2 * length);
    let p = [0, 1].map(|k| c[k] + (2 * t + 1) * d[k] + n[k]);
    p.map(|v| (v - 1) / 2)
}

/// A cell of one site's frame, inside it or past `border`, in the
/// neighbour's frame.
pub fn across_cell(cell: [i64; 3], border: Border, length: i64) -> [i64; 3] {
    let doubled = cell.map(|v| 2 * v + 1);
    crate::edit::across(doubled, border, 2 * length).map(|v| (v - 1).div_euclid(2))
}

/// The cell just past `side` beside edge cell `t`, in the neighbour's frame.
pub(crate) fn far_cell(side: u8, t: i64, length: i64, border: Border, y: i64) -> [i64; 3] {
    let [x, z] = edge_cell(side, t, length);
    let (_, _, n) = crate::edit::frame(side, length);
    across_cell([x - n[0], y, z - n[1]], border, length)
}

/// Each base cell along `side` of `site`, from its corner, read against
/// the world's climb.
pub fn cells<A: Atlas + ?Sized>(a: &A, site: SiteId, side: u8, climb: Climb) -> Result<Vec<Span>> {
    cells_with(a, site, side, climb, true)
}

/// The same, with water seen or not, so a control can blind it.
pub fn cells_with<A: Atlas + ?Sized>(
    a: &A,
    site: SiteId,
    side: u8,
    climb: Climb,
    water: bool,
) -> Result<Vec<Span>> {
    let (to, border) = a.border(site, side).ok_or("a side without a border")?;
    let length = a.footprint().side as i64;
    let (near, far) = (SiteRule::of(a, site)?, SiteRule::of(a, to)?);
    let mut out = Vec::with_capacity(length as usize);
    for t in 0..length {
        let [x, z] = edge_cell(side, t, length);
        let [fx, _, fz] = far_cell(side, t, length, border, 0);
        let (top, other) = (near.top(x, z), far.top(fx, fz));
        let wet = top < near.water || other < far.water;
        let step = top.abs_diff(other);
        out.push(if (water && wet) || step >= CLIFF as u64 {
            Span::Stops
        } else if step * u64::from(climb.run) <= u64::from(climb.rise) {
            Span::Passes
        } else {
            Span::Climbs { step }
        });
    }
    Ok(out)
}

/// The profile's spans along `side` of `site`, each the easiest of its
/// cells.
pub fn spans<A: Atlas + ?Sized>(a: &A, site: SiteId, side: u8, climb: Climb) -> Result<Vec<Span>> {
    let cells = cells(a, site, side, climb)?;
    let per = cells.len() / SPANS as usize;
    Ok(cells.chunks(per).map(easiest).collect())
}

/// The easiest crossing among cells: any pass, else the least climb.
pub fn easiest(cells: &[Span]) -> Span {
    let mut best = Span::Stops;
    for &c in cells {
        best = match (best, c) {
            (Span::Passes, _) | (_, Span::Passes) => Span::Passes,
            (Span::Climbs { step: a }, Span::Climbs { step: b }) => Span::Climbs { step: a.min(b) },
            (Span::Stops, c) | (c, Span::Stops) => c,
        };
    }
    best
}
