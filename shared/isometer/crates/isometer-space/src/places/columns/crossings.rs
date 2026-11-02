// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Passages from the columns: the crossings `passages::of` finds cell by
//! cell, found from each stance's steps and each water run's sides.

use super::stances::Stances;
use super::{Cell, HEADROOM, Profile};
use crate::places::cells::DIRECTIONS;
use crate::places::passages::{Crossings, assemble, file};
use crate::places::{Passage, Place, PlaceId};
use std::collections::BTreeMap;

/// Which place labels each run and each stance.
pub(super) struct Labels<'a> {
    /// Every water and room-air run by column and index, sorted.
    pub runs: &'a [(usize, usize)],
    pub run_place: &'a [Option<PlaceId>],
    pub stance_place: &'a [Option<PlaceId>],
}

impl Labels<'_> {
    /// The place labelling exactly `at`.
    fn of(&self, p: &Profile<'_>, s: &Stances, at: [i64; 3]) -> Option<PlaceId> {
        if let Some(q) = s.id(p, at).and_then(|i| self.stance_place[i]) {
            return Some(q);
        }
        let c = p.index(at[0], at[2]).filter(|_| at[1] >= p.floor)?;
        let k = p.run(c, at[1]).0;
        self.runs.binary_search(&(c, k)).ok().and_then(|r| self.run_place[r])
    }
}

pub(super) fn of(
    p: &Profile<'_>,
    s: &Stances,
    labels: &Labels<'_>,
    places: &BTreeMap<PlaceId, Place>,
) -> Vec<Passage> {
    let mut crossings = Crossings::new();
    let mut cross = |pq: [PlaceId; 2], a: [i64; 3], b: [i64; 3], height: i64, step: i64| {
        let way = [b[0] - a[0], b[2] - a[2]];
        file(&mut crossings, pq, [a, b], [way, way.map(|v| -v)], height, step);
    };
    // Stances level with water, or above it over clear air: below its top
    // cell only level, from its top as high as the air over it allows.
    for (r, &(c, k)) in labels.runs.iter().enumerate() {
        let run = p.columns[c][k];
        let Some(id) = labels.run_place[r].filter(|_| run.cell == Cell::Water) else { continue };
        let x = p.v.min[0] + c as i64 % (p.v.max[0] - p.v.min[0]);
        let z = p.v.min[1] + c as i64 / (p.v.max[0] - p.v.min[0]);
        let top = run.y[1] - 1;
        let clear = (1 + p.headroom([x, top + 1, z])).min(HEADROOM);
        for [dx, dz] in DIRECTIONS {
            for b in s.within(p, [x + dx, z + dz], [run.y[0], top + clear]) {
                let Some(q) = labels.of(p, s, s.at[b]) else { continue };
                let a = [x, s.at[b][1].min(top), z];
                cross([id, q], a, s.at[b], s.room[b], s.at[b][1] - a[1]);
            }
        }
    }
    for (i, &a) in s.at.iter().enumerate() {
        let Some(id) = labels.of(p, s, a) else { continue };
        let room = s.room[i];
        for &(b, rise) in s.steps(i) {
            if let Some(q) = labels.of(p, s, s.at[b]) {
                cross([id, q], a, s.at[b], room.min(s.room[b]), rise);
            }
        }
        // Water level with this stance, or the first thing below it across
        // clear air.
        for [dx, dz] in DIRECTIONS {
            let mut w = [a[0] + dx, a[1], a[2] + dz];
            let limit = p.floor.max(a[1] - HEADROOM + 1);
            if w[1] > limit && p.air(w) {
                let c = p.index(w[0], w[2]).expect("air lies in the window");
                w[1] = (p.run(c, w[1]).1.y[0] - 1).max(limit);
            }
            if p.cell(w) == Cell::Water
                && let Some(q) = labels.of(p, s, w)
            {
                cross([id, q], a, w, room, w[1] - a[1]);
            }
        }
    }
    assemble(crossings, places)
}
