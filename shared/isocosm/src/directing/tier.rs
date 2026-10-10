// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The tier line (ruling 742, from legacy `places::near`): which mind runs
//! an agent, embodied near the focus or statistical far from it, with
//! hysteresis. At site grain its hops are along site routes.

use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Which mind runs an agent: embodied, or the statistical ecology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Near,
    Far,
}

/// Promote within `promote_hops` of the focus; demote only past
/// `demote_hops`. The band between is memory.
#[derive(Clone, Copy, Debug)]
pub struct TierLine {
    pub promote_hops: u32,
    pub demote_hops: u32,
}

impl Default for TierLine {
    fn default() -> Self {
        // A 3x3 enclosure has diameter two, so three would leave the far
        // tier unreachable; the outer ring is the demotion boundary.
        Self {
            promote_hops: 1,
            demote_hops: 2,
        }
    }
}

impl TierLine {
    /// The next tier, `hops` from the focus; none is out of reach.
    pub fn next(&self, current: Tier, hops: Option<u32>) -> Tier {
        let hops = hops.unwrap_or(u32::MAX);
        match current {
            Tier::Far if hops <= self.promote_hops => Tier::Near,
            Tier::Near if hops >= self.demote_hops => Tier::Far,
            _ => current,
        }
    }
}

/// Hops from one site to another along routes, if it can be reached.
pub fn hops(sites: &BTreeMap<Id, Site>, from: Id, to: Id) -> Option<u32> {
    let mut seen = BTreeSet::from([from]);
    let mut queue = VecDeque::from([(from, 0u32)]);
    while let Some((at, n)) = queue.pop_front() {
        if at == to {
            return Some(n);
        }
        for route in sites.get(&at).map_or(&[][..], |s| &s.routes) {
            if seen.insert(route.to) {
                queue.push_back((route.to, n + 1));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(n: u64) -> BTreeMap<Id, Site> {
        (0..n)
            .map(|i| {
                let routes = [i.checked_sub(1), (i + 1 < n).then_some(i + 1)]
                    .into_iter()
                    .flatten()
                    .map(|to| Route {
                        to,
                        travel: 1,
                        transmission: 1_000_000,
                        border: None,
                    })
                    .collect();
                let site = Site {
                    terrain_seed: 0,
                    conditions: BTreeMap::new(),
                    accounts: BTreeMap::new(),
                    routes,
                };
                (i, site)
            })
            .collect()
    }

    #[test]
    fn the_band_between_promotion_and_demotion_is_memory() {
        let sites = line(4);
        let t = TierLine::default();
        assert_eq!(hops(&sites, 0, 3), Some(3));
        assert_eq!(t.next(Tier::Far, hops(&sites, 0, 1)), Tier::Near);
        assert_eq!(t.next(Tier::Near, hops(&sites, 0, 1)), Tier::Near);
        assert_eq!(t.next(Tier::Near, hops(&sites, 0, 2)), Tier::Far);
        assert_eq!(t.next(Tier::Far, hops(&sites, 0, 2)), Tier::Far);
        assert_eq!(t.next(Tier::Near, None), Tier::Far);
    }
}
