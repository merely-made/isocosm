// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Replaying edits: the edits that reach a site, read into its frame (ruling
//! 414); the cells they leave changed (ruling 413); a chunk with those cells
//! as exceptions (ruling 415); and what one edit moves, counted run by run
//! against the column rule rather than cell by cell (ruling 412).

use super::{Edit, Op, runs};
use crate::lift::{Chunk, Exception, Lattice, Materials, soil_depth};
use crate::{Atlas, Result, SiteId};
use std::collections::BTreeMap;

/// One site's column rule at the base grain.
pub(crate) struct SiteRule {
    lattice: Lattice,
    pub water: i64,
    pub soil: i64,
    pub m: Materials,
}

impl SiteRule {
    pub fn of<A: Atlas + ?Sized>(a: &A, site: SiteId) -> Result<Self> {
        Ok(Self {
            lattice: a.lattice(site)?,
            water: a.water(site)?,
            soil: soil_depth(a.relief(site)?, 1),
            m: a.materials()?,
        })
    }

    /// The highest solid base cell of column `(x, z)`.
    pub fn top(&self, x: i64, z: i64) -> i64 {
        let exact = self.lattice.numerator(x as u64, z as u64);
        exact.div_euclid(self.lattice.denominator()) as i64
    }

    pub fn rule(&self, top: i64, y: i64) -> u8 {
        if y > top {
            if y <= self.water { self.m.water } else { self.m.air }
        } else if y > top - self.soil {
            self.m.soil
        } else {
            self.m.rock
        }
    }

    /// How many cells of each material the rule puts in `[y0, y1)`.
    fn counts(&self, top: i64, [y0, y1]: [i64; 2]) -> [(u8, i64); 4] {
        let span = |lo: i64, hi: i64| (hi.min(y1) - lo.max(y0)).max(0);
        let wet = self.water.max(top);
        [
            (self.m.rock, span(i64::MIN, top - self.soil + 1)),
            (self.m.soil, span(top - self.soil + 1, top + 1)),
            (self.m.water, span(top + 1, wet + 1)),
            (self.m.air, span(wet + 1, i64::MAX)),
        ]
    }
}

impl Op {
    pub(crate) fn material(self, m: Materials) -> u8 {
        match self {
            Op::Carve => m.air,
            Op::Fill(id) => id,
        }
    }
}

/// Every edit reaching `site`, in order and in its frame: its own, and each
/// a neighbour made read across their shared border (ruling 414). Derived,
/// never stored.
pub fn reaching<A: Atlas + ?Sized>(a: &A, site: SiteId, edits: &[(SiteId, Edit)]) -> Vec<Edit> {
    let footprint = a.footprint();
    let mut out = Vec::new();
    for (made, edit) in edits {
        if *made == site {
            out.push(edit.clone());
        }
        for side in 0..footprint.sides {
            if let Some((to, b)) = a.border(*made, side)
                && to == site
            {
                out.push(edit.across(b, footprint.side));
            }
        }
    }
    out
}

/// The cells `edits` leave changed within base columns `min..max`, keyed
/// by column and height, the last edit to touch a cell deciding it. Only
/// cells whose coordinates are multiples of `step` are kept: a coarse lift
/// point-samples each coarse cell's origin.
pub(crate) fn touched(
    edits: &[Edit],
    m: Materials,
    min: [i64; 2],
    max: [i64; 2],
    step: i64,
) -> BTreeMap<([i64; 2], i64), u8> {
    let up = |v: i64| v + (step - v.rem_euclid(step)) % step;
    let mut out = BTreeMap::new();
    for edit in edits {
        let [lo, hi] = edit.shape.extent();
        let material = edit.op.material(m);
        let (x0, x1) = (up(lo[0].max(min[0])), hi[0].min(max[0]));
        let (z0, z1) = (up(lo[2].max(min[1])), hi[2].min(max[1]));
        for x in (x0..x1).step_by(step as usize) {
            for z in (z0..z1).step_by(step as usize) {
                for [y0, y1] in runs(&edit.shape, x, z) {
                    for y in (up(y0)..y1).step_by(step as usize) {
                        out.insert(([x, z], y), material);
                    }
                }
            }
        }
    }
    out
}

pub(crate) fn lift_edited<A: Atlas + ?Sized>(
    a: &A,
    site: SiteId,
    level: u8,
    at: [u32; 2],
    edits: &[(SiteId, Edit)],
) -> Result<Chunk> {
    let mut chunk = a.lift(site, level, at)?;
    let reach = reaching(a, site, edits);
    if reach.is_empty() {
        return Ok(chunk);
    }
    let step = 1i64 << level;
    let origin = at.map(|v| i64::from(v) * i64::from(crate::CHUNK));
    let side = a.footprint().side as i64;
    let min = origin.map(|o| o * step);
    let max = [0, 1].map(|k| ((origin[k] + i64::from(chunk.columns[k])) * step).min(side));
    for (([x, z], y), material) in touched(&reach, chunk.materials, min, max, step) {
        let column = [(x / step - origin[0]) as u32, (z / step - origin[1]) as u32];
        let y = y.div_euclid(step);
        if material != chunk.rule(column[0], column[1], y) {
            chunk.exceptions.push(Exception {
                column,
                y,
                material,
            });
        }
    }
    chunk.exceptions.sort_unstable();
    Ok(chunk)
}

/// What one edit moves, per material id: cells the volume lost less cells
/// it gained, over every site it reaches. Air weighs nothing and is left out
/// (ruling 412); a product turns cells into mass by its own densities.
pub type Moved = BTreeMap<u8, i64>;

/// The cells `edit`, made in `site` after `prior`, moves.
pub fn tally<A: Atlas + ?Sized>(
    a: &A,
    site: SiteId,
    prior: &[(SiteId, Edit)],
    edit: &Edit,
) -> Result<Moved> {
    let footprint = a.footprint();
    let side = footprint.side as i64;
    let mut copies = vec![(site, edit.clone())];
    for k in 0..footprint.sides {
        if let Some((to, b)) = a.border(site, k) {
            copies.push((to, edit.across(b, footprint.side)));
        }
    }
    let mut moved = Moved::new();
    for (at, copy) in copies {
        let rule = SiteRule::of(a, at)?;
        let [lo, hi] = copy.shape.extent();
        let (min, max) = ([lo[0].max(0), lo[2].max(0)], [hi[0].min(side), hi[2].min(side)]);
        if min[0] >= max[0] || min[1] >= max[1] {
            continue;
        }
        let before = touched(&reaching(a, at, prior), rule.m, min, max, 1);
        let after = copy.op.material(rule.m);
        for x in min[0]..max[0] {
            for z in min[1]..max[1] {
                let top = rule.top(x, z);
                for run in runs(&copy.shape, x, z) {
                    let mut count = |m: u8, n: i64| *moved.entry(m).or_default() += n;
                    for (m, n) in rule.counts(top, run) {
                        count(m, n);
                    }
                    for ((_, y), m) in before.range(([x, z], run[0])..([x, z], run[1])) {
                        count(rule.rule(top, *y), -1);
                        count(*m, 1);
                    }
                    count(after, run[0] - run[1]);
                }
            }
        }
    }
    let air = a.materials()?.air;
    moved.retain(|m, n| *m != air && *n != 0);
    Ok(moved)
}
