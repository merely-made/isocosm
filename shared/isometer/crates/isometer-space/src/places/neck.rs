// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Necks (ruling 738): a walkable component splits where a body as wide as
//! the world's neck width could not stand. A stance is wide when some
//! square of that many columns a side, holding it, is covered by stances
//! it reaches by climbable steps within the square; the rest are necks.
//! Wide and narrow stances flood apart, so a neck becomes its own patch and
//! its narrowness a passage's clearance.

use super::cells::Cells;
use super::flood::walk;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) fn split(c: &Cells<'_>, component: &[[i64; 3]], w: i64) -> Vec<Vec<[i64; 3]>> {
    let inside: BTreeSet<[i64; 3]> = component.iter().copied().collect();
    let steps: BTreeMap<[i64; 3], Vec<[i64; 3]>> = component
        .iter()
        .map(|&a| {
            let next = c
                .steps(a)
                .into_iter()
                .filter(|(b, rise)| c.climbable(*rise) && inside.contains(b))
                .map(|(b, _)| b)
                .collect();
            (a, next)
        })
        .collect();
    let wide: BTreeSet<[i64; 3]> = component
        .iter()
        .copied()
        .filter(|&s| (0..w * w).any(|k| covers(&steps, s, [s[0] - k % w, s[2] - k / w], w)))
        .collect();
    let mut stood = BTreeSet::new();
    let mut pieces = Vec::new();
    for &s in component {
        if !stood.contains(&s) {
            let class = wide.contains(&s);
            pieces.push(walk(c, s, &mut stood, |b| inside.contains(&b) && wide.contains(&b) == class));
        }
    }
    pieces
}

/// Whether stances reached from `s` within the `w`-square at `corner`
/// stand in every one of its columns.
fn covers(steps: &BTreeMap<[i64; 3], Vec<[i64; 3]>>, s: [i64; 3], corner: [i64; 2], w: i64) -> bool {
    let holds = |a: [i64; 3]| (corner[0]..corner[0] + w).contains(&a[0]) && (corner[1]..corner[1] + w).contains(&a[2]);
    let mut seen = BTreeSet::from([s]);
    let mut columns = BTreeSet::from([[s[0], s[2]]]);
    let mut queue = VecDeque::from([s]);
    while let Some(a) = queue.pop_front() {
        for &b in steps.get(&a).into_iter().flatten() {
            if holds(b) && seen.insert(b) {
                columns.insert([b[0], b[2]]);
                queue.push_back(b);
            }
        }
    }
    columns.len() as i64 == w * w
}
