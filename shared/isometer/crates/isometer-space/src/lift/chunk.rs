// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A lift is a window of columns over one site at one cell size (ruling
//! 402): each column's surface top, rounded down from the exact surface, and
//! the rule for what lies above and below it, named by the product's
//! material ids (ruling 403). Edits add a sparse list of cells that differ
//! from that rule (ruling 415). Nothing is kept, so lifting a chunk again
//! gives the same bytes and an unvisited site costs nothing.

use crate::{Atlas, Result, SiteId};
use serde::{Deserialize, Serialize};

/// Columns along a chunk's side.
pub const CHUNK: u32 = 32;

/// The product's ids for what a chunk's columns hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Materials {
    pub air: u8,
    pub water: u8,
    /// The loose layer over rock: the voxel material of rulings 403 and 412.
    pub soil: u8,
    pub rock: u8,
}

/// A cell whose material differs from its column's rule (ruling 415).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Exception {
    /// Its column within the chunk, across and down.
    pub column: [u32; 2],
    /// Cells up, at the chunk's cell size.
    pub y: i64,
    pub material: u8,
}

/// A window of one site's columns at one cell size.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chunk {
    pub site: SiteId,
    /// Cells are `2^level` base units a side.
    pub level: u8,
    /// Its place among the site's chunks at this level.
    pub at: [u32; 2],
    /// Columns across and down: `CHUNK`, or fewer at the site's far edges.
    pub columns: [u32; 2],
    /// Each column's highest solid cell, row by row.
    pub surface: Vec<i64>,
    /// The water top, in cells: a column whose surface lies below it holds
    /// water up to it.
    pub water: i64,
    /// Cells of soil at the top of each column; rock lies below them.
    pub soil: i64,
    pub materials: Materials,
    /// Edited cells, sorted by column then height. Absent from an unedited
    /// chunk, which keeps the bytes it had before edits existed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<Exception>,
}

impl Chunk {
    /// The material id of the cell `y` cells up in column `(x, z)`.
    pub fn material(&self, x: u32, z: u32, y: i64) -> u8 {
        let probe = |e: &Exception| (e.column, e.y).cmp(&([x, z], y));
        match self.exceptions.binary_search_by(probe) {
            Ok(i) => self.exceptions[i].material,
            Err(_) => self.rule(x, z, y),
        }
    }

    /// What the column rule alone gives, edits aside.
    pub fn rule(&self, x: u32, z: u32, y: i64) -> u8 {
        let top = self.surface[(z * self.columns[0] + x) as usize];
        let m = self.materials;
        if y > top {
            if y <= self.water { m.water } else { m.air }
        } else if y > top - self.soil {
            m.soil
        } else {
            m.rock
        }
    }
}

/// Chunks along each side of a site at `level`.
pub(crate) fn chunks<A: Atlas + ?Sized>(a: &A, level: u8) -> u32 {
    let cells = a.footprint().side.div_ceil(1 << level);
    cells.div_ceil(u64::from(CHUNK)) as u32
}

/// Chunk `at` of `site` at `level`, its columns sampled at their origins.
pub(crate) fn chunk<A: Atlas + ?Sized>(a: &A, site: SiteId, level: u8, at: [u32; 2]) -> Result<Chunk> {
    if level > 16 {
        return Err("a cell outside the lift's exact range".into());
    }
    let lattice = a.lattice(site)?;
    let cell = 1u64 << level;
    let cells = a.footprint().side.div_ceil(cell);
    let origin = at.map(|a| u64::from(a) * u64::from(CHUNK));
    if origin[0] >= cells || origin[1] >= cells {
        return Err("a chunk outside its site".into());
    }
    let columns = origin.map(|o| (cells - o).min(u64::from(CHUNK)) as u32);
    let per_cell = lattice.denominator() * i128::from(cell);
    let mut surface = Vec::with_capacity((columns[0] * columns[1]) as usize);
    for z in 0..u64::from(columns[1]) {
        for x in 0..u64::from(columns[0]) {
            let exact = lattice.numerator((origin[0] + x) * cell, (origin[1] + z) * cell);
            surface.push(exact.div_euclid(per_cell) as i64);
        }
    }
    let cell = cell as i64;
    Ok(Chunk {
        site,
        level,
        at,
        columns,
        surface,
        water: a.water(site)?.div_euclid(cell),
        soil: soil_depth(a.relief(site)?, cell),
        materials: a.materials()?,
        exceptions: Vec::new(),
    })
}

/// Cells of soil over rock: an eighth of the relief, at least one cell.
pub(crate) fn soil_depth(relief: i64, cell: i64) -> i64 {
    ((relief / 8).max(1) / cell).max(1)
}
