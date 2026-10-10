// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Routing and walking over places (ruling 702): a search over passages
//! weighted by travel cost, keeping only those a body fits (ruling 418).
//! Within a place a body moves freely; a walk is the crossings between.

use super::{Clearance, Passage, PlaceId, Places};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};

/// What a passage asks of a body: its width and height in base units, the
/// step it can climb, and whether it goes into water.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    pub width: u32,
    pub height: u32,
    pub climb: u32,
    pub wades: bool,
}

impl Body {
    /// The clearance of `passage` this body fits, if any.
    pub fn fits<'p>(&self, passage: &'p Passage) -> Option<&'p Clearance> {
        if passage.wet && !self.wades {
            return None;
        }
        passage
            .clearances
            .iter()
            .find(|c| c.width >= self.width && c.height >= self.height && c.step <= self.climb)
    }
}

impl Places {
    /// The cheapest route from `from` to `to` for `body`, with its cost.
    pub fn route(&self, from: PlaceId, to: PlaceId, body: &Body) -> Option<(u64, Vec<PlaceId>)> {
        self.route_by(from, to, |p| body.fits(p).is_some())
    }

    /// The same search with a caller's filter, so a broken one can be shown
    /// to let through what [`Body::fits`] refuses.
    pub fn route_by(
        &self,
        from: PlaceId,
        to: PlaceId,
        admit: impl Fn(&Passage) -> bool,
    ) -> Option<(u64, Vec<PlaceId>)> {
        let mut next: BTreeMap<PlaceId, Vec<(PlaceId, u64)>> = BTreeMap::new();
        for p in self.passages.values().filter(|p| admit(p)) {
            let [a, b] = p.between;
            next.entry(a).or_default().push((b, p.cost));
            next.entry(b).or_default().push((a, p.cost));
        }
        let mut best: BTreeMap<PlaceId, (u64, Option<PlaceId>)> = BTreeMap::from([(from, (0, None))]);
        let mut queue = BinaryHeap::from([Reverse((0u64, from))]);
        while let Some(Reverse((cost, at))) = queue.pop() {
            if at == to {
                let mut path = vec![to];
                while let Some((_, Some(prev))) = best.get(path.last()?) {
                    path.push(*prev);
                }
                path.reverse();
                return Some((cost, path));
            }
            if best.get(&at).is_some_and(|b| b.0 < cost) {
                continue;
            }
            for &(n, step) in next.get(&at).into_iter().flatten() {
                let c = cost + step;
                if best.get(&n).is_none_or(|b| c < b.0) {
                    best.insert(n, (c, Some(at)));
                    queue.push(Reverse((c, n)));
                }
            }
        }
        None
    }

    /// Waypoints from cell `from` to cell `to` for `body`: each crossing's
    /// two cells along the cheapest route, then `to`.
    pub fn walk(&self, from: [i64; 3], to: [i64; 3], body: &Body) -> Option<Vec<[i64; 3]>> {
        let (a, b) = (self.place_at(from)?, self.place_at(to)?);
        let (_, path) = self.route(a, b, body)?;
        let mut out = Vec::new();
        for pair in path.windows(2) {
            let key = if pair[0] < pair[1] { [pair[0], pair[1]] } else { [pair[1], pair[0]] };
            let c = body.fits(self.passages.get(&key)?)?;
            let [near, far] = if pair[0] < pair[1] { c.cells } else { [c.cells[1], c.cells[0]] };
            out.extend([near, far]);
        }
        out.push(to);
        Some(out)
    }
}
