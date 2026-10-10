// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A part's cells by identity (wing ruling 766, re-expressing Mesocosm's
//! mosaic): beside the count each function holds, which cells it holds.
//! The cells are a lattice read from the part's box (460), one cell per two
//! voxels of half-extent plus one, at most four an axis, addressed
//! `x + dx * (y + dy * z)`, adjacent along each axis; nothing stores a
//! coordinate. A tract is the cells one function holds: sorted, disjoint
//! from the part's other tracts, never a lost cell. Counts stay what the
//! readings read; [`sync`] keeps the tracts to them, and [`propose`] places
//! cells exactly, the counts following.

use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub mod ports;

/// A cell's address in its part's lattice. Stable while the box is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellId(pub u16);

/// The cells one function holds on a part.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tract {
    pub function: Key,
    pub cells: Vec<CellId>,
}

/// The lattice a box of `half` reads as, cells along each axis.
pub fn dims(half: [i32; 3]) -> [u16; 3] {
    half.map(|h| (h.unsigned_abs().max(1) / 2 + 1).clamp(1, 4) as u16)
}

fn at(d: [u16; 3], c: CellId) -> [u16; 3] {
    let i = c.0;
    [i % d[0], (i / d[0]) % d[1], i / (d[0] * d[1])]
}

fn id(d: [u16; 3], p: [u16; 3]) -> CellId {
    CellId(p[0] + d[0] * (p[1] + d[1] * p[2]))
}

/// Every cell, along a path whose each step is to a neighbour (a
/// serpentine through the lattice), so a run along it is connected.
pub fn path(d: [u16; 3]) -> Vec<CellId> {
    let mut out = vec![];
    for z in 0..d[2] {
        for row in 0..d[1] {
            let y = if z % 2 == 0 { row } else { d[1] - 1 - row };
            let flip = (z * d[1] + row) % 2 == 1;
            for col in 0..d[0] {
                let x = if flip { d[0] - 1 - col } else { col };
                out.push(id(d, [x, y, z]));
            }
        }
    }
    out
}

/// The cells adjacent to `c`.
pub fn neighbours(d: [u16; 3], c: CellId) -> Vec<CellId> {
    let p = at(d, c);
    let mut out = vec![];
    for axis in 0..3 {
        if p[axis] > 0 {
            let mut q = p;
            q[axis] -= 1;
            out.push(id(d, q));
        }
        if p[axis] + 1 < d[axis] {
            let mut q = p;
            q[axis] += 1;
            out.push(id(d, q));
        }
    }
    out
}

/// Whether `cells` form one connected region of the lattice.
pub fn connected(d: [u16; 3], cells: &[CellId]) -> bool {
    let all: BTreeSet<CellId> = cells.iter().copied().collect();
    let Some(&first) = all.iter().next() else {
        return true;
    };
    let mut seen = BTreeSet::from([first]);
    let mut queue = vec![first];
    while let Some(c) = queue.pop() {
        for n in neighbours(d, c) {
            if all.contains(&n) && seen.insert(n) {
                queue.push(n);
            }
        }
    }
    seen.len() == all.len()
}

/// Keeps `p`'s tracts to its counts, in a box of `half`: a function with no
/// cells loses its tract; one holding too many gives up the cells latest
/// along the lattice's path; one holding too few takes free cells, first
/// those next to what it holds, along the path. Cells outside the lattice
/// or lost leave their tracts. *Reading:* this keeps identity where counts
/// move by a cell or two, and is not required to keep a tract connected;
/// only a proposal is.
pub fn sync(p: &mut Part, half: [i32; 3]) {
    let d = dims(half);
    let order = path(d);
    let rank = |c: &CellId| order.iter().position(|o| o == c).unwrap_or(usize::MAX);
    let lost: BTreeSet<CellId> = p.lost.iter().copied().collect();
    let total = order.len();
    p.tracts
        .retain(|t| p.cells.get(&t.function).copied().unwrap_or(0) > 0);
    for t in &mut p.tracts {
        t.cells
            .retain(|c| usize::from(c.0) < total && !lost.contains(c));
    }
    for (function, n) in p.cells.clone() {
        if !p.tracts.iter().any(|t| t.function == function) {
            p.tracts.push(Tract {
                function: function.clone(),
                cells: vec![],
            });
        }
        let t = p
            .tracts
            .iter_mut()
            .find(|t| t.function == function)
            .unwrap();
        t.cells.sort_by_key(rank);
        t.cells.truncate(n as usize);
    }
    for (function, n) in p.cells.clone() {
        loop {
            let held: BTreeSet<CellId> = p.tracts.iter().flat_map(|t| t.cells.clone()).collect();
            let t = p.tracts.iter().find(|t| t.function == function).unwrap();
            if t.cells.len() >= n as usize {
                break;
            }
            let free = |c: &&CellId| !held.contains(*c) && !lost.contains(*c);
            let near: BTreeSet<CellId> = t.cells.iter().flat_map(|c| neighbours(d, *c)).collect();
            let pick = order
                .iter()
                .filter(free)
                .find(|c| near.contains(*c))
                .or_else(|| order.iter().find(free));
            let Some(&c) = pick else { break };
            let t = p
                .tracts
                .iter_mut()
                .find(|t| t.function == function)
                .unwrap();
            t.cells.push(c);
        }
    }
    for t in &mut p.tracts {
        t.cells.sort();
    }
    p.tracts.retain(|t| !t.cells.is_empty());
    p.tracts.sort();
}

/// Places `p`'s cells exactly, in a box of `half` (Mesocosm's allocation
/// proposal): each named function the cells listed, every tract non-empty,
/// connected, disjoint from the others, inside the lattice and never on a
/// lost cell. The counts and expressed functions follow. Returns how many
/// cells changed what they do, free cells included, which is what a
/// development costs.
pub fn propose(p: &mut Part, half: [i32; 3], tracts: &[(Key, Vec<CellId>)]) -> Result<u32, String> {
    let d = dims(half);
    let total = path(d).len();
    let lost: BTreeSet<CellId> = p.lost.iter().copied().collect();
    let mut taken = BTreeSet::new();
    for (function, cells) in tracts {
        if cells.is_empty() {
            return Err(format!("an empty tract for {function}"));
        }
        if !connected(d, cells) {
            return Err(format!("the tract for {function} is not one region"));
        }
        for c in cells {
            if usize::from(c.0) >= total || lost.contains(c) || !taken.insert(*c) {
                return Err(format!("cell {} cannot go to {function}", c.0));
            }
        }
    }
    let was = |c: CellId| {
        p.tracts
            .iter()
            .find(|t| t.cells.contains(&c))
            .map(|t| t.function.clone())
    };
    let now = |c: CellId| {
        tracts
            .iter()
            .find(|t| t.1.contains(&c))
            .map(|t| t.0.clone())
    };
    let changed = path(d).into_iter().filter(|c| was(*c) != now(*c)).count() as u32;
    p.tracts = tracts
        .iter()
        .map(|(function, cells)| {
            let mut cells = cells.clone();
            cells.sort();
            Tract {
                function: function.clone(),
                cells,
            }
        })
        .collect();
    p.tracts.sort();
    p.cells = p
        .tracts
        .iter()
        .map(|t| (t.function.clone(), t.cells.len() as u32))
        .collect();
    p.functions = p.cells.keys().cloned().collect();
    Ok(changed)
}

/// Whether `p`'s tracts agree with its counts in a box of `half`: what the
/// world's check asks of every part with cells.
pub fn agrees(p: &Part, half: [i32; 3]) -> Result<(), String> {
    let total = path(dims(half)).len();
    let mut seen = BTreeSet::new();
    for t in &p.tracts {
        let n = p.cells.get(&t.function).copied().unwrap_or(0);
        if t.cells.len() != n as usize {
            return Err(format!(
                "{} holds {n} cells but a tract of {}",
                t.function,
                t.cells.len()
            ));
        }
        for c in &t.cells {
            if usize::from(c.0) >= total || p.lost.contains(c) || !seen.insert(*c) {
                return Err(format!("cell {} is not {}'s to hold", c.0, t.function));
            }
        }
    }
    let laid = |f: &Key| p.tracts.iter().any(|t| &t.function == f);
    match p.cells.iter().find(|(f, n)| **n > 0 && !laid(f)) {
        Some((f, _)) => Err(format!("{f} holds cells no tract lays out")),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
