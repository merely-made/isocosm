// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Reading;
use isocosm::{asserted::{Place, Route}, schema::{Id, Tick}};
use std::collections::{BTreeMap, BTreeSet};

impl<'a> Reading<'a> {
    /// Authored labels and map anchors borrowed from native sites, in key order.
    pub fn authored_places(&self) -> BTreeMap<&'a str, (Id, &'a Place)> {
        self.state().sites.iter().filter_map(|(id, site)| {
            site.authored.as_ref().map(|place| (place.key.as_str(), (*id, place)))
        }).collect()
    }

    /// One borrowed authored fill per route key; native endpoints keep both sides.
    pub fn authored_routes(&self) -> BTreeMap<&'a str, &'a Route> {
        self.state().sites.values().flat_map(|site| &site.routes)
            .filter_map(|route| route.authored.as_ref())
            .map(|route| (route.key.as_str(), route)).collect()
    }

    /// Cheapest native site path and its raw travel weight. Rules resolve the trip.
    pub fn travel_path(&self, from: Id, to: Id) -> Option<(Vec<Id>, Tick)> {
        let sites = &self.state().sites;
        if !sites.contains_key(&from) || !sites.contains_key(&to) {
            return None;
        }
        let mut best = BTreeMap::from([(from, 0)]);
        let mut previous = BTreeMap::new();
        let mut queue = BTreeSet::from([(0u64, from)]);
        while let Some((cost, at)) = queue.pop_first() {
            if best.get(&at) != Some(&cost) {
                continue;
            }
            if at == to {
                let mut path = vec![to];
                let mut cursor = to;
                while cursor != from {
                    cursor = *previous.get(&cursor)?;
                    path.push(cursor);
                }
                path.reverse();
                return Some((path, cost));
            }
            for route in &sites.get(&at)?.routes {
                if !sites.contains_key(&route.to) {
                    continue;
                }
                let Some(next) = cost.checked_add(route.travel) else {
                    continue;
                };
                if best.get(&route.to).is_none_or(|old| next < *old) {
                    best.insert(route.to, next);
                    previous.insert(route.to, at);
                    queue.insert((next, route.to));
                }
            }
        }
        None
    }
}
