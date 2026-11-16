// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Other bodies' boxes as the physical checks read them. A watched set records
//! every region a check asked about, so a remembered search can
//! be reused exactly while those answers stand (D54).
use super::*;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};

/// Side of a recorded cell. A coarser record only invalidates more often.
const CELL: i64 = 16;
/// A clear query spanning more cells than this is kept as one exact box.
const WIDE: i64 = 4096;
type Cell = [i64; 3];

fn overlap(a: &Box3, b: &Box3) -> bool {
    (0..3).all(|k| a[0][k] < b[1][k] && b[0][k] < a[1][k])
}
/// Inclusive cell ranges covering `[lo, hi)`; a flat box covers its own cell.
fn span(b: &Box3) -> [[i64; 2]; 3] {
    [0, 1, 2].map(|k| {
        [
            b[0][k].div_euclid(CELL),
            (b[1][k].max(b[0][k] + 1) - 1).div_euclid(CELL),
        ]
    })
}
fn count(s: &[[i64; 2]; 3]) -> i64 {
    s.iter()
        .map(|[a, b]| b - a + 1)
        .fold(1i64, |n, c| n.saturating_mul(c))
}

/// What a search asked of the other bodies: the regions it read, and the
/// bodies standing in them when it finished. Every query counts, found or not.
#[derive(Clone, Debug, Default)]
pub struct Footprint {
    /// Cells covering every query.
    cells: HashSet<Cell>,
    /// Wide queries, kept exact.
    wide: BTreeSet<Box3>,
    /// The boxes in those regions at the end, sorted.
    inside: Vec<Box3>,
    last: Option<[[i64; 2]; 3]>,
}
impl Footprint {
    fn record(&mut self, q: &Box3) {
        let s = span(q);
        if self.last == Some(s) {
            return;
        }
        self.last = Some(s);
        if count(&s) > WIDE {
            self.wide.insert(*q);
            return;
        }
        for x in s[0][0]..=s[0][1] {
            for y in s[1][0]..=s[1][1] {
                for z in s[2][0]..=s[2][1] {
                    self.cells.insert([x, y, z]);
                }
            }
        }
    }
    /// Notes which of `blocked` stand in the regions read.
    pub fn seal(&mut self, blocked: &[Box3]) {
        self.inside = self.standing(blocked);
    }
    fn standing(&self, blocked: &[Box3]) -> Vec<Box3> {
        let mut found: Vec<Box3> = blocked.iter().filter(|b| self.reaches(b)).copied().collect();
        found.sort_unstable();
        found
    }
    /// True when the boxes in every region read are the ones recorded, so a
    /// fresh search with `blocked` asks the same questions and gets the same
    /// answers. A body entering a region, leaving it or moving within it ends
    /// that; one in no region read does not.
    pub fn holds_for(&self, blocked: &[Box3]) -> bool {
        self.standing(blocked) == self.inside
    }
    /// A recorded cell as a box, for tests.
    #[cfg(test)]
    pub fn a_cell(&self) -> Option<Box3> {
        self.cells
            .iter()
            .min()
            .map(|c| [*c, c.map(|v| v + 1)].map(|p| p.map(|v| v * CELL)))
    }
    fn reaches(&self, b: &Box3) -> bool {
        // One wider each side, so flat boxes on a cell edge are caught.
        let wide = [b[0].map(|v| v - 1), b[1].map(|v| v + 1)];
        if self.wide.iter().any(|q| overlap(q, &wide)) {
            return true;
        }
        let s = span(&wide);
        if count(&s) <= self.cells.len() as i64 {
            (s[0][0]..=s[0][1]).any(|x| {
                (s[1][0]..=s[1][1]).any(|y| (s[2][0]..=s[2][1]).any(|z| self.cells.contains(&[x, y, z])))
            })
        } else {
            self.cells.iter().any(|c| {
                let cell = [*c, c.map(|v| v + 1)].map(|p| p.map(|v| v * CELL));
                overlap(&cell, &wide)
            })
        }
    }
}

/// The boxes of the other bodies, optionally watched.
#[derive(Clone, Copy)]
pub struct Blockers<'a> {
    boxes: &'a [Box3],
    log: Option<&'a RefCell<Footprint>>,
}
impl<'a> Blockers<'a> {
    pub fn new(boxes: &'a [Box3]) -> Self {
        Self { boxes, log: None }
    }
    pub fn none() -> Self {
        Self::new(&[])
    }
    pub fn watched(boxes: &'a [Box3], log: &'a RefCell<Footprint>) -> Self {
        Self {
            boxes,
            log: Some(log),
        }
    }
    /// As `intersects(query, boxes)`, noting each box asked about.
    pub fn hits(&self, query: &[Box3]) -> bool {
        let Some(log) = self.log else {
            return intersects(query, self.boxes);
        };
        for q in query {
            let hit = intersects(std::slice::from_ref(q), self.boxes);
            log.borrow_mut().record(q);
            if hit {
                return true;
            }
        }
        false
    }
}
