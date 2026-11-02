// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The stances of a window, each an index, and the steps between them:
//! what `Cells::steps` reads cell by cell, read here from the neighbouring
//! column's stances and runs.

use super::{Cell, HEADROOM, Profile};
use crate::places::cells::DIRECTIONS;

pub(super) struct Stances {
    /// Each stance's cell, column by column, lowest first.
    pub at: Vec<[i64; 3]>,
    /// Each column's first stance; one past the last at the end.
    first: Vec<usize>,
    pub room: Vec<i64>,
    pub open: Vec<bool>,
    /// Each stance's steps, any rise: `steps[step_first[s]..step_first[s + 1]]`.
    step_first: Vec<usize>,
    steps: Vec<(usize, i64)>,
}

impl Stances {
    pub fn new(p: &Profile<'_>) -> Self {
        let mut at = Vec::new();
        let mut first = Vec::with_capacity(p.columns.len() + 1);
        for (c, runs) in p.columns.iter().enumerate() {
            first.push(at.len());
            let x = p.v.min[0] + c as i64 % (p.v.max[0] - p.v.min[0]);
            let z = p.v.min[1] + c as i64 / (p.v.max[0] - p.v.min[0]);
            let mut below = Cell::Solid;
            for r in runs {
                if r.cell == Cell::Air && below == Cell::Solid {
                    at.push([x, r.y[0], z]);
                }
                below = r.cell;
            }
        }
        first.push(at.len());
        let room: Vec<i64> = at.iter().map(|&a| p.headroom(a)).collect();
        let open = at.iter().map(|&a| p.open(a)).collect();
        let mut s = Self { at, first, room, open, step_first: Vec::new(), steps: Vec::new() };
        let mut steps = Vec::new();
        let mut step_first = Vec::with_capacity(s.at.len() + 1);
        for (i, &a) in s.at.iter().enumerate() {
            step_first.push(steps.len());
            for [dx, dz] in DIRECTIONS {
                s.step_into(p, a, s.room[i], [a[0] + dx, a[2] + dz], &mut steps);
            }
        }
        step_first.push(steps.len());
        s.steps = steps;
        s.step_first = step_first;
        s
    }

    /// The stances of column `[x, z]` with `y` in `lo..hi`, ascending.
    pub fn within(&self, p: &Profile<'_>, [x, z]: [i64; 2], [lo, hi]: [i64; 2]) -> std::ops::Range<usize> {
        let Some(c) = p.index(x, z) else { return 0..0 };
        let ys = &self.at[self.first[c]..self.first[c + 1]];
        let start = self.first[c] + ys.partition_point(|a| a[1] < lo);
        let end = self.first[c] + ys.partition_point(|a| a[1] < hi);
        start..end.max(start)
    }

    /// The stance at `at`, if one stands there.
    pub fn id(&self, p: &Profile<'_>, at: [i64; 3]) -> Option<usize> {
        self.within(p, [at[0], at[2]], [at[1], at[1] + 1]).next()
    }

    /// Up or level onto any stance within `room`, no higher than the top;
    /// or down the neighbour's air run to its foot, if solid holds it.
    fn step_into(&self, p: &Profile<'_>, a: [i64; 3], room: i64, [nx, nz]: [i64; 2], out: &mut Vec<(usize, i64)>) {
        let y = a[1];
        for b in self.within(p, [nx, nz], [y, (y + room).min(p.top + 1)]) {
            out.push((b, self.at[b][1] - y));
        }
        let Some(c) = p.index(nx, nz).filter(|_| p.air([nx, y, nz])) else { return };
        let foot = p.run(c, y).1.y[0];
        if foot < y && foot >= p.floor && foot > y - HEADROOM
            && let Some(b) = self.id(p, [nx, foot, nz])
        {
            out.push((b, foot - y));
        }
    }

    pub fn steps(&self, s: usize) -> &[(usize, i64)] {
        &self.steps[self.step_first[s]..self.step_first[s + 1]]
    }
}
