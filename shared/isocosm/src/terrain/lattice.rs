// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A site's surface is the exact interpolation of a lattice over its
//! footprint (rulings 400 and 401). The boundary is the four edge profiles,
//! the interior their Coons patch with windowed detail, and a correction
//! shaped by the same window makes the mean over the site's base-grain
//! columns its elevation exactly. Heights are held in units of `1 / q` base
//! units, so that mean has a closed form and no column is ever summed.

use super::{SPANS, View, signed};
use crate::{Result, schema::Id};

/// Lattice points along a side.
pub const POINTS: usize = SPANS as usize + 1;
const LAST: usize = SPANS as usize;

/// The window detail and the correction are shaped by: nothing at a site's
/// edges, 256 at its middle.
pub fn fading(k: u32) -> i64 {
    4 * i64::from(k) * i64::from(SPANS - k)
}

/// A site's surface lattice, indexed `[z][x]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lattice {
    /// Base units between lattice points: the side over `SPANS`.
    pub spacing: i64,
    /// Heights before the correction, in base units.
    pub base: [[i64; POINTS]; POINTS],
    /// The detail each point carries, within its site's relief.
    pub detail: [[i64; POINTS]; POINTS],
    /// Heights with the correction, in units of `1 / q` base units.
    pub heights: [[i128; POINTS]; POINTS],
    pub q: i128,
}

impl Lattice {
    /// The exact surface at `(x, z)` base units in the site's frame, both in
    /// `0..=side`, as a numerator over [`Self::denominator`].
    pub fn numerator(&self, x: u64, z: u64) -> i128 {
        let spacing = self.spacing as u64;
        let (i, fx) = cell(x, spacing);
        let (j, fz) = cell(z, spacing);
        let (s, fx, fz) = (i128::from(self.spacing), i128::from(fx), i128::from(fz));
        let h = &self.heights;
        h[j][i] * (s - fx) * (s - fz)
            + h[j][i + 1] * fx * (s - fz)
            + h[j + 1][i] * (s - fx) * fz
            + h[j + 1][i + 1] * fx * fz
    }

    pub fn denominator(&self) -> i128 {
        i128::from(self.spacing) * i128::from(self.spacing) * self.q
    }

    /// The surface `t` base units along `side` from the site's corner `side`,
    /// as a numerator over [`Self::denominator`].
    pub fn on_side(&self, side: u8, t: u64) -> i128 {
        let length = self.spacing as u64 * u64::from(SPANS);
        match side {
            0 => self.numerator(t, 0),
            1 => self.numerator(length, t),
            2 => self.numerator(length - t, length),
            _ => self.numerator(0, length - t),
        }
    }
}

/// The lattice cell a coordinate falls in, and how far into it.
fn cell(t: u64, spacing: u64) -> (usize, u64) {
    let i = (t / spacing).min(u64::from(SPANS) - 1);
    (i as usize, t - i * spacing)
}

impl View<'_> {
    /// A site's lattice, fading and corrected as rulings 400 and 401 ask.
    pub fn lattice(&self, site: Id) -> Result<Lattice> {
        self.lattice_with(site, fading, true)
    }

    /// A site's lattice with its detail shaped by `window` and the correction
    /// on or off, so the checks' controls can break either.
    pub fn lattice_with(
        &self,
        site: Id,
        window: impl Fn(u32) -> i64,
        corrected: bool,
    ) -> Result<Lattice> {
        let side = self.footprint.side;
        if self.footprint.sides != 4 || !side.is_multiple_of(u64::from(SPANS)) || side > 1 << 16 {
            return Err("a footprint outside the lift's exact range".into());
        }
        let s = (side / u64::from(SPANS)) as i64;
        let sides = [
            self.side_heights(site, 0)?,
            self.side_heights(site, 1)?,
            self.side_heights(site, 2)?,
            self.side_heights(site, 3)?,
        ];
        let relief = self.relief(site)?;
        let seed = self.sites.get(&site).ok_or("an absent site")?.terrain_seed;
        let mut lattice = Lattice {
            spacing: s,
            base: [[0; POINTS]; POINTS],
            detail: [[0; POINTS]; POINTS],
            heights: [[0; POINTS]; POINTS],
            q: 4 * i128::from(s) * i128::from(s),
        };
        let mut sum = 0i128;
        for j in 0..POINTS {
            for i in 0..POINTS {
                let draw = crate::draw(seed, "terrain:detail", &[i as u64, j as u64]);
                let detail = signed(draw, relief) * window(i as u32) * window(j as u32) / 65_536;
                let base = boundary(&sides, i, j).unwrap_or_else(|| coons(&sides, i, j)) + detail;
                lattice.detail[j][i] = detail;
                lattice.base[j][i] = base;
                lattice.heights[j][i] = i128::from(base) * lattice.q;
                sum += i128::from(base) * weight(s, i) * weight(s, j);
            }
        }
        if corrected {
            let side = i128::from(side);
            let elevation = i128::from(self.elevation(site)?);
            spread(&mut lattice.heights, 4 * elevation * side * side - sum);
        }
        Ok(lattice)
    }

    /// A side's control heights in this site's own direction, from its
    /// corner `side` to the next. A bordered side reads its shared profile;
    /// an open edge draws its own from its corners, having nobody to agree
    /// with.
    fn side_heights(&self, site: Id, side: u8) -> Result<[i64; POINTS]> {
        let mut heights = [0; POINTS];
        if let Some((_, b)) = self.border(site, side) {
            let profile = self.edge_profile(site, side)?;
            let forward = b.flipped || self.canonical(site, side)? == (site, side);
            for (k, h) in heights.iter_mut().enumerate() {
                *h = profile.heights[if forward { k } else { LAST - k }];
            }
            return Ok(heights);
        }
        let next = (side + 1) % self.footprint.sides;
        let (h0, h1) = (
            self.corner_height(site, side)?,
            self.corner_height(site, next)?,
        );
        let amplitude = self.relief(site)? / 2;
        let spans = i64::from(SPANS);
        for (k, h) in heights.iter_mut().enumerate() {
            let at = k as i64;
            let draw = crate::draw(
                self.seed,
                "terrain:open-edge",
                &[site, side.into(), at as u64],
            );
            let fade = 4 * at * (spans - at);
            *h = h0 + (h1 - h0) * at / spans + signed(draw, amplitude) * fade / (spans * spans);
        }
        Ok(heights)
    }
}

/// A boundary point's height, from the side it lies on.
fn boundary(sides: &[[i64; POINTS]; 4], i: usize, j: usize) -> Option<i64> {
    match (i, j) {
        (_, 0) => Some(sides[0][i]),
        (LAST, _) => Some(sides[1][j]),
        (_, LAST) => Some(sides[2][LAST - i]),
        (0, _) => Some(sides[3][LAST - j]),
        _ => None,
    }
}

/// The Coons patch of the four sides at an interior point.
fn coons(sides: &[[i64; POINTS]; 4], i: usize, j: usize) -> i64 {
    let north = |i: usize| sides[0][i];
    let east = |j: usize| sides[1][j];
    let south = |i: usize| sides[2][LAST - i];
    let west = |j: usize| sides[3][LAST - j];
    let n = i64::from(SPANS);
    let (u, v) = (i as i64, j as i64);
    let (c00, c10, c11, c01) = (north(0), north(LAST), east(LAST), south(0));
    let edges = (n - v) * n * north(i) + v * n * south(i) + (n - u) * n * west(j) + u * n * east(j);
    let corners = (n - u) * (n - v) * c00 + u * (n - v) * c10 + (n - u) * v * c01 + u * v * c11;
    (edges - corners).div_euclid(n * n)
}

/// A lattice point's share of the site's base-grain columns, doubled: an
/// edge point counts half a spacing, give or take a column.
fn weight(s: i64, k: usize) -> i128 {
    i128::from(match k {
        0 => s + 1,
        LAST => s - 1,
        _ => 2 * s,
    })
}

/// Spreads `total` units of `1 / q` over the interior points in proportion
/// to the fading window, the remainder one unit each to the points with the
/// largest window and then by position, so the sum is exact.
fn spread(heights: &mut [[i128; POINTS]; POINTS], total: i128) {
    let window = |i: usize, j: usize| i128::from(fading(i as u32) * fading(j as u32));
    let interior = || (1..LAST).flat_map(|j| (1..LAST).map(move |i| (i, j)));
    let whole: i128 = interior().map(|(i, j)| window(i, j)).sum();
    let mut given = 0;
    for (i, j) in interior() {
        let share = (total * window(i, j)).div_euclid(whole);
        heights[j][i] += share;
        given += share;
    }
    let mut order: Vec<(i128, usize, usize)> =
        interior().map(|(i, j)| (-window(i, j), j, i)).collect();
    order.sort_unstable();
    for &(_, j, i) in order.iter().take((total - given) as usize) {
        heights[j][i] += 1;
    }
}
