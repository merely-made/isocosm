// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What places read of a volume, cell by cell: air, water or solid; where a
//! walker can stand; how much headroom it has; and which neighbouring
//! stances it can step to.

use super::{Cover, Rules};
use crate::volume::Volume;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Headroom is counted this high and no further, and no step is sought
/// this tall either way: a drawn cliff's least drop (ruling 744).
pub(super) const HEADROOM: i64 = crate::lift::CLIFF;

/// The four horizontal neighbours.
pub(super) const DIRECTIONS: [[i64; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cell {
    Air,
    Water,
    Solid,
    Out,
}

pub(super) struct Cells<'a> {
    pub v: &'a Volume,
    pub rules: Rules,
    /// The lowest cell held; below is rock.
    pub floor: i64,
    /// The open sky: every cell at or above it is air.
    pub top: i64,
    roofs: RefCell<BTreeMap<[i64; 2], Option<i64>>>,
}

impl<'a> Cells<'a> {
    pub fn new(v: &'a Volume, rules: Rules) -> Self {
        Self {
            v,
            rules,
            floor: v.datum,
            top: v.ceiling(),
            roofs: RefCell::default(),
        }
    }

    pub fn cell(&self, [x, y, z]: [i64; 3]) -> Cell {
        if !self.v.holds(x, z) {
            return Cell::Out;
        }
        if y >= self.top {
            return Cell::Air;
        }
        let m = self.v.material([x, y, z]).unwrap_or(self.v.materials.rock);
        if m == self.v.materials.air {
            Cell::Air
        } else if m == self.v.materials.water {
            Cell::Water
        } else {
            Cell::Solid
        }
    }

    pub fn air(&self, at: [i64; 3]) -> bool {
        self.cell(at) == Cell::Air
    }

    /// Air over something solid: where a walker stands.
    pub fn stance(&self, [x, y, z]: [i64; 3]) -> bool {
        self.air([x, y, z]) && self.cell([x, y - 1, z]) == Cell::Solid
    }

    /// Air cells from `at` up, `at` included, counted to [`HEADROOM`].
    pub fn headroom(&self, [x, y, z]: [i64; 3]) -> i64 {
        (0..HEADROOM).take_while(|d| self.air([x, y + d, z])).count() as i64
    }

    /// The highest solid cell in a column.
    fn roof(&self, x: i64, z: i64) -> Option<i64> {
        if let Some(r) = self.roofs.borrow().get(&[x, z]) {
            return *r;
        }
        let r = (self.floor..self.top).rev().find(|&y| self.cell([x, y, z]) == Cell::Solid);
        self.roofs.borrow_mut().insert([x, z], r);
        r
    }

    /// Whether a walkable flood may take `at` as outdoors. Under `Sealed`
    /// a flood never leaves the air its seed was judged in, so it may.
    pub fn open(&self, [x, y, z]: [i64; 3]) -> bool {
        match self.rules.cover {
            Cover::Sealed => true,
            Cover::Roofed => self.roof(x, z).is_none_or(|r| r < y),
        }
    }

    /// Whether a step of `rise` across one unit is within the world's climb.
    pub fn climbable(&self, rise: i64) -> bool {
        rise.unsigned_abs() * u64::from(self.rules.climb.run) <= u64::from(self.rules.climb.rise)
    }

    /// The stances a walker at stance `a` can step to across one column,
    /// any rise, with the rise: up while `a`'s column is clear above it,
    /// level, or down while the neighbour's column is clear up to `a`.
    pub fn steps(&self, a: [i64; 3]) -> Vec<([i64; 3], i64)> {
        let room = self.headroom(a);
        DIRECTIONS
            .iter()
            .flat_map(|[dx, dz]| self.step_into(a, room, self, [a[0] + dx, a[2] + dz]))
            .collect()
    }

    /// The steps from stance `a`, with `room` headroom, into column
    /// `[nx, nz]` of `far`: this volume's, or a neighbour's across a border
    /// with the same heights.
    pub fn step_into(&self, a: [i64; 3], room: i64, far: &Cells<'_>, [nx, nz]: [i64; 2]) -> Vec<([i64; 3], i64)> {
        let y = a[1];
        let mut out = Vec::new();
        let mut below = far.cell([nx, y - 1, nz]);
        for ny in y..(y + room).min(far.top + 1) {
            let here = far.cell([nx, ny, nz]);
            if here == Cell::Air && below == Cell::Solid {
                out.push(([nx, ny, nz], ny - y));
            }
            below = here;
        }
        if far.air([nx, y, nz]) {
            let mut ny = y - 1;
            while ny >= far.floor && ny > y - HEADROOM && far.air([nx, ny, nz]) {
                if far.stance([nx, ny, nz]) {
                    out.push(([nx, ny, nz], ny - y));
                    break;
                }
                ny -= 1;
            }
        }
        out
    }

    /// Every cell of the window, floor to sky.
    pub fn everywhere(&self) -> Vec<[i64; 3]> {
        let mut out = Vec::new();
        for x in self.v.min[0]..self.v.max[0] {
            for z in self.v.min[1]..self.v.max[1] {
                out.extend((self.floor..=self.top).map(|y| [x, y, z]));
            }
        }
        out
    }

    /// Every cell of the given columns, floor to sky.
    pub fn columns(&self, columns: impl IntoIterator<Item = [i64; 2]>) -> Vec<[i64; 3]> {
        let mut out = Vec::new();
        for [x, z] in columns {
            if self.v.holds(x, z) {
                out.extend((self.floor..=self.top).map(|y| [x, y, z]));
            }
        }
        out
    }
}
