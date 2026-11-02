// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The places: water and rooms as runs joined across their sides, patches
//! as walks over the stance graph, split at necks and cut by the cap as
//! `flood` splits and cuts them.

use super::stances::Stances;
use super::{Cell, Profile, crossings};
use crate::Result;
use crate::places::flood::{Found, made};
use crate::places::{Cover, Kind, PlaceId, Places, Rules};
use crate::volume::Volume;
use std::collections::{BTreeMap, VecDeque};

/// Every place of `volume`, derived from its columns.
pub(in crate::places) fn derive(volume: &Volume, rules: Rules) -> Result<Places> {
    let p = Profile::new(volume, rules);
    let s = Stances::new(&p);
    let (runs, joined) = bodies(&p);
    let mut found = Vec::new();
    let mut run_place = vec![None; runs.len()];
    for (sky, members) in joined {
        let kind = match p.columns[runs[members[0]].0][runs[members[0]].1].cell {
            Cell::Water => Kind::Water,
            _ if sky => continue,
            _ => Kind::Room,
        };
        let mut cells = Vec::new();
        for &r in &members {
            let (c, k) = runs[r];
            let run = p.columns[c][k];
            let [x, z] = column(&p, c);
            cells.extend((run.y[0]..run.y[1]).map(|y| [x, y, z]));
        }
        let f = made(volume.site, kind, cells, None);
        for &r in &members {
            run_place[r] = Some(f.place.id);
        }
        found.push(f);
    }
    // Walkable seeds: open stances, or under `Sealed` those under the sky.
    let sky_stance = |i: usize| match rules.cover {
        Cover::Roofed => s.open[i],
        Cover::Sealed => {
            let a = s.at[i];
            let c = p.index(a[0], a[2]).expect("a stance's column");
            let k = p.run(c, a[1]).0;
            let r = runs.binary_search(&(c, k)).expect("an air run");
            run_place[r].is_none()
        },
    };
    let mut stood = vec![false; s.at.len()];
    let mut scratch = Scratch::new(s.at.len());
    let mut patches = Vec::new();
    for i in 0..s.at.len() {
        if !stood[i] && s.open[i] && sky_stance(i) {
            let component = walk(&p, &s, i, &mut stood, |_| true);
            patches.extend(cut(&p, &s, &mut scratch, component));
        }
    }
    let mut stance_place = vec![None; s.at.len()];
    for (ids, f) in patches {
        for i in ids {
            stance_place[i] = Some(f.place.id);
        }
        found.push(f);
    }
    let mut labels: Vec<([i64; 3], PlaceId)> = Vec::new();
    for f in &found {
        labels.extend(f.cells.iter().map(|&[x, y, z]| ([x, z, y], f.place.id)));
    }
    labels.sort_unstable();
    let places: BTreeMap<_, _> = found.into_iter().map(|f| (f.place.id, f.place)).collect();
    let labelled = crossings::Labels { runs: &runs, run_place: &run_place, stance_place: &stance_place };
    let passages = crossings::of(&p, &s, &labelled, &places);
    Ok(Places {
        site: volume.site,
        rules,
        places,
        passages: passages.into_iter().map(|q| (q.between, q)).collect(),
        labels: labels.into_iter().collect(),
    })
}

fn column(p: &Profile<'_>, c: usize) -> [i64; 2] {
    let wide = p.v.max[0] - p.v.min[0];
    [p.v.min[0] + c as i64 % wide, p.v.min[1] + c as i64 / wide]
}

/// Every water and room-air run, by column and index, sorted; and the
/// bodies they join into, each with whether it reaches the sky.
fn bodies(p: &Profile<'_>) -> (Vec<(usize, usize)>, Vec<(bool, Vec<usize>)>) {
    let class = |c: usize, k: usize| match p.columns[c][k].cell {
        Cell::Water => Some(0),
        Cell::Air if p.rules.cover == Cover::Sealed || p.roofed(c, k) => Some(1),
        _ => None,
    };
    let mut runs = Vec::new();
    for (c, column) in p.columns.iter().enumerate() {
        runs.extend((0..column.len()).filter(|&k| class(c, k).is_some()).map(|k| (c, k)));
    }
    let mut parent: Vec<usize> = (0..runs.len()).collect();
    fn root(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let id = |c: usize, k: usize| runs.binary_search(&(c, k)).ok();
    for c in 0..p.columns.len() {
        let [x, z] = column(p, c);
        for n in [p.index(x + 1, z), p.index(x, z + 1)].into_iter().flatten() {
            let (a, b) = (&p.columns[c], &p.columns[n]);
            let (mut i, mut j) = (0, 0);
            while i < a.len() && j < b.len() {
                let overlap = a[i].y[0] < b[j].y[1] && b[j].y[0] < a[i].y[1];
                if overlap && class(c, i).is_some() && class(c, i) == class(n, j) {
                    let (ri, rj) = (id(c, i).unwrap(), id(n, j).unwrap());
                    let (ri, rj) = (root(&mut parent, ri), root(&mut parent, rj));
                    parent[ri.max(rj)] = ri.min(rj);
                }
                if a[i].y[1] <= b[j].y[1] { i += 1 } else { j += 1 }
            }
        }
    }
    let mut joined: BTreeMap<usize, (bool, Vec<usize>)> = BTreeMap::new();
    for r in 0..runs.len() {
        let (c, k) = runs[r];
        let entry = joined.entry(root(&mut parent, r)).or_default();
        entry.0 |= p.columns[c][k].y[1] == i64::MAX;
        entry.1.push(r);
    }
    (runs, joined.into_values().collect())
}

/// Marks kept across a derivation's walks, each cleared by its walker.
pub(super) struct Scratch {
    pub stood: Vec<bool>,
    pub inside: Vec<bool>,
    pub wide: Vec<bool>,
    /// Which group a stance is kept in, `usize::MAX` for none.
    pub group: Vec<usize>,
}

impl Scratch {
    pub fn new(n: usize) -> Self {
        Self { stood: vec![false; n], inside: vec![false; n], wide: vec![false; n], group: vec![usize::MAX; n] }
    }
}

/// Whether a step of `rise` across one unit is within the world's climb.
pub(super) fn climbable(p: &Profile<'_>, rise: i64) -> bool {
    rise.unsigned_abs() * u64::from(p.rules.climb.run) <= u64::from(p.rules.climb.rise)
}

/// The walkable component of stance `from`: stances joined by climbable
/// steps onto open stances, restricted by `keep`.
pub(super) fn walk(
    p: &Profile<'_>,
    s: &Stances,
    from: usize,
    stood: &mut [bool],
    keep: impl Fn(usize) -> bool,
) -> Vec<usize> {
    let mut out = vec![from];
    let mut queue = VecDeque::from([from]);
    stood[from] = true;
    while let Some(a) = queue.pop_front() {
        for &(b, rise) in s.steps(a) {
            if !stood[b] && climbable(p, rise) && s.open[b] && keep(b) {
                stood[b] = true;
                out.push(b);
                queue.push_back(b);
            }
        }
    }
    out
}

/// A walkable component as patches, each with its stances.
fn cut(p: &Profile<'_>, s: &Stances, scratch: &mut Scratch, component: Vec<usize>) -> Vec<(Vec<usize>, Found)> {
    let least = component.iter().map(|&i| s.at[i]).min().expect("a component has its seed");
    let pieces = match p.rules.neck {
        Some(w) if w >= 2 => super::necks::split(p, s, scratch, &component, i64::from(w)),
        _ => vec![component],
    };
    pieces.into_iter().flat_map(|piece| capped(p, s, scratch, piece, least)).collect()
}

fn capped(
    p: &Profile<'_>,
    s: &Stances,
    scratch: &mut Scratch,
    piece: Vec<usize>,
    least: [i64; 3],
) -> Vec<(Vec<usize>, Found)> {
    let patch = |ids: Vec<usize>| {
        let f = made(p.v.site, Kind::Patch, ids.iter().map(|&i| s.at[i]).collect(), Some(least));
        (ids, f)
    };
    let span = |k: usize| {
        let (lo, hi) = piece.iter().fold((i64::MAX, i64::MIN), |(lo, hi), &i| (lo.min(s.at[i][k]), hi.max(s.at[i][k])));
        hi - lo + 1
    };
    let cap = p.rules.cap.sides().map(i64::from);
    if span(0) <= cap[0] && span(2) <= cap[1] {
        return vec![patch(piece)];
    }
    let mut by: BTreeMap<[i64; 2], Vec<usize>> = BTreeMap::new();
    for &i in &piece {
        by.entry([s.at[i][0].div_euclid(cap[0]), s.at[i][2].div_euclid(cap[1])]).or_default().push(i);
    }
    let mut out = Vec::new();
    for (g, ids) in by.values().enumerate() {
        for &i in ids {
            scratch.group[i] = g;
        }
        for &i in ids {
            if !scratch.stood[i] {
                let group = &scratch.group;
                out.push(patch(walk(p, s, i, &mut scratch.stood, |b| group[b] == g)));
            }
        }
    }
    for &i in &piece {
        scratch.stood[i] = false;
        scratch.group[i] = usize::MAX;
    }
    out
}
