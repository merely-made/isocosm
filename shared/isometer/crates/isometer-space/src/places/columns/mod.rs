// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Places from the column rule and its exceptions (the place-graph plan's
//! PG7). Each column is read once, as runs of air, water and solid from
//! the floor up: the rule's layers with the edits laid over them, never a
//! voxel at a time. Rooms and water are runs joined where they overlap
//! across a side, so rule air under the sky is never flooded and a room
//! needs an exception to exist; the walking surface is a graph over each
//! column's stances, the run starts with solid below. The places and
//! passages are the cell-by-cell derivation's (`flood`, `passages`), which
//! stays as the reference and serves local re-derivation.

mod crossings;
mod derive;
mod necks;
mod stances;
#[cfg(test)]
mod tests;

use super::cells::{Cell, HEADROOM};
use super::{Cover, Rules};
use crate::volume::Volume;

pub(super) use derive::derive;

/// One run of a column: cells `y[0]..y[1]` alike.
#[derive(Clone, Copy, Debug)]
pub(super) struct Run {
    pub y: [i64; 2],
    pub cell: Cell,
}

/// A window's columns as runs.
pub(super) struct Profile<'a> {
    pub v: &'a Volume,
    pub rules: Rules,
    /// The lowest cell held; below is rock.
    pub floor: i64,
    /// The open sky: every cell at or above it is air.
    pub top: i64,
    wide: i64,
    /// Each column's runs from the floor, its last air reaching `i64::MAX`.
    pub columns: Vec<Vec<Run>>,
    /// Each column's highest solid cell below the top.
    roofs: Vec<Option<i64>>,
}

impl<'a> Profile<'a> {
    pub fn new(v: &'a Volume, rules: Rules) -> Self {
        let (floor, top) = (v.datum, v.ceiling());
        let m = v.materials;
        let class = |id: u8| match id {
            id if id == m.air => Cell::Air,
            id if id == m.water => Cell::Water,
            _ => Cell::Solid,
        };
        let wide = v.max[0] - v.min[0];
        let mut columns = Vec::with_capacity((wide * (v.max[1] - v.min[1])) as usize);
        let mut roofs = Vec::with_capacity(columns.capacity());
        for z in v.min[1]..v.max[1] {
            for x in v.min[0]..v.max[0] {
                let mut runs: Vec<Run> = Vec::new();
                let read = v.column(x, z, [floor, top]);
                let sky = std::iter::once(([top, i64::MAX], m.air));
                for (y, id) in read.into_iter().chain(sky) {
                    let cell = class(id);
                    match runs.last_mut() {
                        Some(last) if last.cell == cell => last.y[1] = y[1],
                        _ => runs.push(Run { y, cell }),
                    }
                }
                roofs.push(runs.iter().rev().find(|r| r.cell == Cell::Solid).map(|r| r.y[1].min(top) - 1));
                columns.push(runs);
            }
        }
        Self { v, rules, floor, top, wide, columns, roofs }
    }

    /// Column `(x, z)`'s index, if the window holds it.
    pub fn index(&self, x: i64, z: i64) -> Option<usize> {
        self.v.holds(x, z).then(|| ((z - self.v.min[1]) * self.wide + x - self.v.min[0]) as usize)
    }

    /// Column `c`'s run holding `y`, at or above the floor.
    pub fn run(&self, c: usize, y: i64) -> (usize, Run) {
        let runs = &self.columns[c];
        let k = runs.partition_point(|r| r.y[1] <= y);
        (k, runs[k])
    }

    pub fn cell(&self, [x, y, z]: [i64; 3]) -> Cell {
        match self.index(x, z) {
            None => Cell::Out,
            Some(_) if y < self.floor => Cell::Solid,
            Some(c) => self.run(c, y).1.cell,
        }
    }

    pub fn air(&self, at: [i64; 3]) -> bool {
        self.cell(at) == Cell::Air
    }

    /// Air cells from `at` up, `at` included, counted to [`HEADROOM`].
    pub fn headroom(&self, [x, y, z]: [i64; 3]) -> i64 {
        let Some(c) = self.index(x, z).filter(|_| y >= self.floor) else {
            return 0;
        };
        match self.run(c, y).1 {
            Run { cell: Cell::Air, y: [_, end] } => (end.saturating_sub(y)).min(HEADROOM),
            _ => 0,
        }
    }

    /// Whether `at` is outdoors: under `Roofed`, nothing solid above it.
    pub fn open(&self, [x, y, z]: [i64; 3]) -> bool {
        match self.rules.cover {
            Cover::Sealed => true,
            Cover::Roofed => self.index(x, z).and_then(|c| self.roofs[c]).is_none_or(|r| r < y),
        }
    }

    /// Whether column `c`'s run `k` is air under its roof.
    pub fn roofed(&self, c: usize, k: usize) -> bool {
        let r = self.columns[c][k];
        r.cell == Cell::Air && self.roofs[c].is_some_and(|roof| r.y[1] <= roof)
    }
}
