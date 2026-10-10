// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Passages: every crossing between two places, gathered into runs side by
//! side, each run's clearance the narrowest headroom and steepest step along
//! it (ruling 418). A crossing is a step between stances in different
//! places, or a stance beside water.

use super::cells::{Cell, Cells, DIRECTIONS};
use super::flood::Found;
use super::{Clearance, Kind, Passage, PlaceId, Places};
use std::collections::{BTreeMap, BTreeSet};

/// One crossing, from the lesser place's cell to the greater's.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Crossing {
    cells: [[i64; 3]; 2],
    height: u32,
    step: u32,
}

/// The passages touching any of `found`, read against every labelled cell.
pub(super) fn of(c: &Cells<'_>, places: &Places, found: &[Found]) -> Vec<Passage> {
    let mut crossings: BTreeMap<[PlaceId; 2], BTreeSet<Crossing>> = BTreeMap::new();
    let mut cross = |p: PlaceId, q: PlaceId, a: [i64; 3], b: [i64; 3], height: i64, step: i64| {
        if p == q {
            return;
        }
        let (between, cells) = if p < q { ([p, q], [a, b]) } else { ([q, p], [b, a]) };
        crossings.entry(between).or_default().insert(Crossing {
            cells,
            height: height as u32,
            step: step.unsigned_abs() as u32,
        });
    };
    for f in found {
        let id = f.place.id;
        for &a in &f.cells {
            if f.place.kind == Kind::Water {
                // Stances level with this water, or above it over clear air.
                let clear = 1 + c.headroom([a[0], a[1] + 1, a[2]]);
                for [dx, dz] in DIRECTIONS {
                    for dy in 0..clear {
                        let b = [a[0] + dx, a[1] + dy, a[2] + dz];
                        if let Some(q) = places.label(b).filter(|_| c.stance(b)) {
                            cross(id, q, a, b, c.headroom(b), dy);
                        }
                    }
                }
                continue;
            }
            if !c.stance(a) {
                continue;
            }
            let room = c.headroom(a);
            for (b, rise) in c.steps(a) {
                if let Some(q) = places.label(b) {
                    cross(id, q, a, b, room.min(c.headroom(b)), rise);
                }
            }
            // Water level with this stance, or the first thing below it
            // across clear air.
            for [dx, dz] in DIRECTIONS {
                let mut w = [a[0] + dx, a[1], a[2] + dz];
                while w[1] > c.floor && c.air(w) {
                    w[1] -= 1;
                }
                if c.cell(w) == Cell::Water
                    && let Some(q) = places.label(w)
                {
                    cross(id, q, a, w, room, w[1] - a[1]);
                }
            }
        }
    }
    crossings
        .into_iter()
        .map(|(between, set)| passage(places, between, set))
        .collect()
}

fn passage(places: &Places, between: [PlaceId; 2], set: BTreeSet<Crossing>) -> Passage {
    let all: Vec<Crossing> = set.into_iter().collect();
    // Runs: crossings the same way whose near cells sit side by side.
    let way = |x: &Crossing| [x.cells[1][0] - x.cells[0][0], x.cells[1][2] - x.cells[0][2]];
    let mut parent: Vec<usize> = (0..all.len()).collect();
    fn root(p: &mut [usize], i: usize) -> usize {
        let mut i = i;
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    let index: BTreeMap<([i64; 2], [i64; 3]), usize> =
        all.iter().enumerate().map(|(i, x)| ((way(x), x.cells[0]), i)).collect();
    for (i, x) in all.iter().enumerate() {
        let [wx, wz] = way(x);
        let a = x.cells[0];
        for side in [[wz, wx], [-wz, -wx]] {
            for dy in -1..=1 {
                let n = [a[0] + side[0], a[1] + dy, a[2] + side[1]];
                if let Some(&j) = index.get(&([wx, wz], n)) {
                    let (ri, rj) = (root(&mut parent, i), root(&mut parent, j));
                    parent[ri.max(rj)] = ri.min(rj);
                }
            }
        }
    }
    let mut runs: BTreeMap<usize, Vec<&Crossing>> = BTreeMap::new();
    for (i, x) in all.iter().enumerate() {
        runs.entry(root(&mut parent, i)).or_default().push(x);
    }
    let mut front: Vec<Clearance> = Vec::new();
    for run in runs.values() {
        front.push(Clearance {
            width: run.len() as u32,
            height: run.iter().map(|x| x.height).min().unwrap_or(0),
            step: run.iter().map(|x| x.step).max().unwrap_or(0),
            cells: run[0].cells,
        });
        for x in run {
            front.push(Clearance {
                width: 1,
                height: x.height,
                step: x.step,
                cells: x.cells,
            });
        }
    }
    front.sort_unstable();
    let dominated = |c: &Clearance, d: &Clearance| {
        d.width >= c.width && d.height >= c.height && d.step <= c.step
            && (d.width, d.height, d.step) != (c.width, c.height, c.step)
    };
    let mut kept: Vec<Clearance> = Vec::new();
    for c in &front {
        let same = kept.iter().any(|k| (k.width, k.height, k.step) == (c.width, c.height, c.step));
        if !same && !front.iter().any(|d| dominated(c, d)) {
            kept.push(*c);
        }
    }
    let centre = |id: &PlaceId| places.places.get(id).map(|p| p.centre).unwrap_or(id.cell);
    let (a, b) = (centre(&between[0]), centre(&between[1]));
    let wet = between
        .iter()
        .any(|id| places.places.get(id).is_some_and(|p| p.kind == Kind::Water));
    Passage {
        between,
        wet,
        clearances: kept,
        cost: (0..3).map(|k| (a[k] - b[k]).unsigned_abs()).sum::<u64>() + 1,
    }
}
