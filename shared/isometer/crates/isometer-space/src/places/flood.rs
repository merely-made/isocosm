// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Components: rooms (air cut off from the sky), water bodies, and walkable
//! patches cut by the cap. Each flood runs to its component's end, so the
//! places found from any seed are the places a whole derivation finds.

use super::cells::{Cell, Cells};
use super::{Cover, Kind, Place, PlaceId};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A place and the cells it labels.
pub(super) struct Found {
    pub place: Place,
    pub cells: Vec<[i64; 3]>,
}

const SIX: [[i64; 3]; 6] = [
    [1, 0, 0],
    [-1, 0, 0],
    [0, 1, 0],
    [0, -1, 0],
    [0, 0, 1],
    [0, 0, -1],
];

fn add(a: [i64; 3], d: [i64; 3]) -> [i64; 3] {
    [a[0] + d[0], a[1] + d[1], a[2] + d[2]]
}

/// Every component reached from `seeds`, least id first.
pub(super) fn components(c: &Cells<'_>, seeds: Vec<[i64; 3]>) -> Vec<Found> {
    let mut seen = BTreeSet::new();
    let mut sky = BTreeSet::new();
    let mut walkable = Vec::new();
    let mut found = Vec::new();
    for s in seeds {
        if seen.contains(&s) || sky.contains(&s) {
            continue;
        }
        match c.cell(s) {
            Cell::Water => {
                let cells = flood6(s, &mut seen, |a| c.cell(a) == Cell::Water, |_| false).0;
                found.push(made(c, Kind::Water, cells, None));
            },
            Cell::Air => match c.rules.cover {
                Cover::Sealed => {
                    let (cells, open) = flood6(s, &mut seen, |a| c.air(a), |a| a[1] >= c.top || sky.contains(&a));
                    if open {
                        walkable.extend(cells.iter().copied().filter(|&a| c.stance(a)));
                        sky.extend(cells);
                    } else {
                        found.push(made(c, Kind::Room, cells, None));
                    }
                },
                Cover::Roofed if !c.open(s) => {
                    let cells = flood6(s, &mut seen, |a| c.air(a) && !c.open(a), |_| false).0;
                    found.push(made(c, Kind::Room, cells, None));
                },
                Cover::Roofed => {
                    if c.stance(s) {
                        walkable.push(s);
                    }
                },
            },
            Cell::Solid | Cell::Out => {},
        }
    }
    let mut stood = BTreeSet::new();
    for s in walkable {
        if stood.contains(&s) || !c.open(s) {
            continue;
        }
        let component = walk(c, s, &mut stood, |_| true);
        found.extend(cut(c, component));
    }
    found.sort_by_key(|f| f.place.id);
    found
}

/// A 6-connected flood over `inside` from `s`, with whether it touched
/// `open`. It stops early once open: an open flood is the sky, which is no
/// place and need not be walked whole.
fn flood6(
    s: [i64; 3],
    seen: &mut BTreeSet<[i64; 3]>,
    inside: impl Fn([i64; 3]) -> bool,
    open: impl Fn([i64; 3]) -> bool,
) -> (Vec<[i64; 3]>, bool) {
    let mut cells = vec![s];
    let mut queue = VecDeque::from([s]);
    let mut local = BTreeSet::from([s]);
    while let Some(a) = queue.pop_front() {
        if open(a) {
            return (cells, true);
        }
        for d in SIX {
            let b = add(a, d);
            if !local.contains(&b) && !seen.contains(&b) && inside(b) {
                local.insert(b);
                cells.push(b);
                queue.push_back(b);
            }
        }
    }
    seen.extend(cells.iter().copied());
    (cells, false)
}

/// The walkable component of stance `s`: stances joined by steps within the
/// climb, through outdoor air, restricted by `keep`.
fn walk(
    c: &Cells<'_>,
    s: [i64; 3],
    stood: &mut BTreeSet<[i64; 3]>,
    keep: impl Fn([i64; 3]) -> bool,
) -> Vec<[i64; 3]> {
    let mut cells = vec![s];
    let mut queue = VecDeque::from([s]);
    stood.insert(s);
    while let Some(a) = queue.pop_front() {
        for (b, rise) in c.steps(a) {
            if c.climbable(rise) && !stood.contains(&b) && c.open(b) && keep(b) {
                stood.insert(b);
                cells.push(b);
                queue.push_back(b);
            }
        }
    }
    cells
}

/// A walkable component as patches: whole, or cut on the cap's grid when it
/// is wider than the cap either way (ruling 417).
fn cut(c: &Cells<'_>, component: Vec<[i64; 3]>) -> Vec<Found> {
    let least = *component.iter().min().expect("a component has its seed");
    let span = |k: usize| {
        let (lo, hi) = component.iter().fold((i64::MAX, i64::MIN), |(lo, hi), a| (lo.min(a[k]), hi.max(a[k])));
        hi - lo + 1
    };
    let cap = c.rules.cap.sides().map(i64::from);
    if span(0) <= cap[0] && span(2) <= cap[1] {
        return vec![made(c, Kind::Patch, component, Some(least))];
    }
    let grid = |a: [i64; 3]| [a[0].div_euclid(cap[0]), a[2].div_euclid(cap[1])];
    let mut by: BTreeMap<[i64; 2], BTreeSet<[i64; 3]>> = BTreeMap::new();
    for &a in &component {
        by.entry(grid(a)).or_default().insert(a);
    }
    let mut out = Vec::new();
    for cells in by.values() {
        let mut stood = BTreeSet::new();
        for &s in cells {
            if !stood.contains(&s) {
                let piece = walk(c, s, &mut stood, |b| cells.contains(&b));
                out.push(made(c, Kind::Patch, piece, Some(least)));
            }
        }
    }
    out
}

fn made(c: &Cells<'_>, kind: Kind, mut cells: Vec<[i64; 3]>, component: Option<[i64; 3]>) -> Found {
    cells.sort_unstable();
    let least = cells[0];
    let n = cells.len() as i64;
    let sum = cells.iter().fold([0i64; 3], |s, a| [s[0] + a[0], s[1] + a[1], s[2] + a[2]]);
    let id = PlaceId {
        site: c.v.site,
        cell: least,
    };
    Found {
        place: Place {
            id,
            kind,
            cells: n as u64,
            centre: sum.map(|v| v.div_euclid(n)),
            component: component.unwrap_or(least),
        },
        cells,
    }
}
