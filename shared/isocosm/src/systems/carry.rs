// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a body's systems carry (rulings 560 to 565, 581 and 582), routed by
//! wing-functions: each living part a junction whose cells carry the
//! world's capacity a tick, a conduct cell by its part's cross-section over
//! the reference segment's; attachments joining parent and child both ways;
//! the sources supplying what a process asks by their cells, and each
//! effect part asking its share, all in one operation, so a part carries
//! everything bound for the parts beyond it.

use super::{filling, union};
use crate::{
    Result, anatomy,
    rules::{Measure, Role, Rules},
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};
use wing_functions::{Edge, FunctionalNetwork, Node, NodeId, NodeKind, PartRef};

const CONDUCT: &str = "function:conduct";
/// An attachment carries whatever its parts can; only cells limit a route.
const OPEN: u64 = u64::MAX / 4;

/// What one process's routes carried.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Carried {
    /// What each effect part asked was carried to it, by part.
    pub parts: BTreeMap<PartId, u64>,
    pub total: u64,
}

/// What a part's cells carry a tick (560, 564, 565), nought where the world
/// sets no carriage.
pub fn capacity(h: anatomy::Half, p: &Part, rules: &Rules) -> u64 {
    let Some(c) = rules.carriage else { return 0 };
    let cells: u64 = p.cells.values().map(|n| u64::from(*n)).sum();
    let conduct = u64::from(p.cells.get(CONDUCT).copied().unwrap_or(0));
    let side = cube_root(rules.body().reference_segment_voxels.max(1));
    let face = u128::from(side * side).max(1);
    let section = anatomy::measure(h, Measure::CrossSection);
    let conducted = u128::from(conduct) * u128::from(c.per_cell) * section / face;
    let plain = u128::from(cells - conduct) * u128::from(c.per_cell);
    u64::try_from(plain + conducted).unwrap_or(u64::MAX)
}

/// The integer cube root: the reference segment's side, its face the
/// cross-section a conduct cell is measured against (25 voxels for
/// Mesocosm's 125, 565's reading).
fn cube_root(n: u64) -> u64 {
    let mut r = (n as f64).cbrt() as u64;
    while (r + 1).saturating_pow(3) <= n {
        r += 1;
    }
    while r > 0 && r.pow(3) > n {
        r -= 1;
    }
    r
}

fn node(id: u32, part: PartId) -> Node {
    Node {
        id: NodeId(id),
        kind: NodeKind::Effect { part: at(part) },
    }
}

fn at(part: PartId) -> PartRef {
    PartRef {
        subject: 0,
        part: part.0,
    }
}

/// What the systems `e` carries naming `function` in `role` carry of
/// `asks`, each effect part's share, in one operation (581). Sources supply
/// the asks' total by their cells of the function, every living part by all
/// its cells; `bitten` is the part a bite landed on. Nothing is carried
/// where no carried system names the function there.
pub fn carry(
    e: &Entity,
    rules: &Rules,
    (function, role): (&str, Role),
    asks: &BTreeMap<PartId, u64>,
    bitten: Option<PartId>,
) -> Result<Carried> {
    let Some(system) = union(e, function, role) else {
        return Ok(Carried::default());
    };
    let sources = filling(e, &system.sources, bitten);
    let effects = filling(e, &system.effects, bitten);
    let living: Vec<PartId> = e.living().map(|(id, _)| id).collect();
    let index: BTreeMap<PartId, u32> = living
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, u32::try_from(i).unwrap_or(u32::MAX)))
        .collect();
    let (jin, jout) = (|i: u32| 2 * i, |i: u32| 2 * i + 1);
    let mut nodes = vec![];
    let mut edges = vec![];
    for (id, &i) in &index {
        nodes.extend([node(jin(i), *id), node(jout(i), *id)]);
        let room = capacity(e.extent(*id), &e.parts[id], rules);
        if room > 0 {
            edges.push(edge(jin(i), jout(i), room));
        }
        if let Some(&j) = e.parent_of(*id).and_then(|q| index.get(&q)) {
            edges.extend([edge(jout(j), jin(i), OPEN), edge(jout(i), jin(j), OPEN)]);
        }
    }
    let wanted: Vec<(PartId, u64)> = asks
        .iter()
        .filter(|(id, n)| **n > 0 && effects.contains(id))
        .map(|(id, n)| (*id, *n))
        .collect();
    let total: u64 = wanted.iter().map(|(_, n)| n).sum();
    let supply = supplies(e, &sources, function, total);
    let mut next = 2 * u32::try_from(living.len()).unwrap_or(u32::MAX);
    for (id, charge) in supply.into_iter().filter(|(_, c)| *c > 0) {
        nodes.push(Node {
            id: NodeId(next),
            kind: NodeKind::Source {
                part: at(id),
                capacity: charge,
                charge,
            },
        });
        edges.push(edge(next, jin(index[&id]), charge));
        next += 1;
    }
    let mut network = FunctionalNetwork::new(nodes, edges).map_err(|e| e.to_string())?;
    let sites: Vec<(NodeId, u64)> = wanted
        .iter()
        .map(|(id, n)| (NodeId(jout(index[id])), *n))
        .collect();
    let live: BTreeSet<PartRef> = living.iter().map(|id| at(*id)).collect();
    let hops = u32::try_from(network.edges.len()).unwrap_or(u32::MAX);
    let receipt = network
        .carry(&sites, hops, &live)
        .map_err(|e| e.to_string())?;
    let parts: BTreeMap<PartId, u64> = wanted
        .iter()
        .zip(&receipt.delivered)
        .map(|((id, _), (_, n))| (*id, *n))
        .collect();
    Ok(Carried {
        total: parts.values().sum(),
        parts,
    })
}

fn edge(from: u32, to: u32, capacity: u64) -> Edge {
    Edge {
        from: NodeId(from),
        to: NodeId(to),
        capacity,
    }
}

/// `total` split over the source parts by their cells of `function`, or by
/// all their cells where the sources are every living part, the units left
/// over to the parts first in order.
fn supplies(
    e: &Entity,
    sources: &BTreeSet<PartId>,
    function: &str,
    total: u64,
) -> Vec<(PartId, u64)> {
    let cells = |p: &Part| -> u64 {
        match p.cells.get(function) {
            Some(n) => u64::from(*n),
            None => p.cells.values().map(|n| u64::from(*n)).sum(),
        }
    };
    let weights: Vec<(PartId, u64)> = sources
        .iter()
        .map(|id| (*id, cells(&e.parts[id])))
        .collect();
    let all: u128 = weights.iter().map(|(_, w)| u128::from(*w)).sum();
    if all == 0 {
        return vec![];
    }
    let mut out: Vec<(PartId, u64)> = weights
        .iter()
        .map(|(id, w)| (*id, (u128::from(*w) * u128::from(total) / all) as u64))
        .collect();
    let mut left = total - out.iter().map(|(_, n)| n).sum::<u64>();
    for ((_, n), (_, w)) in out.iter_mut().zip(&weights) {
        if left > 0 && *w > 0 {
            *n += 1;
            left -= 1;
        }
    }
    out
}
