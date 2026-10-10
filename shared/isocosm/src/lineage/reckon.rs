// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What each line amounts to at a boundary (legacy `score::readings`),
//! read from the world and never accumulated. A reading of nothing is no
//! reading, so a line that holds nothing has no growth mark.

use crate::{schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const GROWTH: &str = "feat:growth";
pub const SPREAD: &str = "feat:spread";
pub const ENDURANCE: &str = "feat:endurance";

/// One measurement of one line.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Reading {
    pub lineage: Key,
    pub feat: Key,
    pub value: u128,
}

#[derive(Default)]
struct Tally {
    held: u128,
    sites: BTreeSet<Id>,
    oldest: Tick,
}

/// Every reading worth noting now, by line then feat: living matter held,
/// sites lived at, and the oldest living member's age.
pub fn readings(sim: &Simulation) -> Vec<Reading> {
    let s = sim.state();
    let rules = &sim.genesis().rules;
    let mut lines: BTreeMap<&Key, Tally> = BTreeMap::new();
    let living = s.population.groups.values().filter(|g| g.entity.alive);
    for g in living.filter(|g| s.lineages.contains_key(&g.entity.lineage)) {
        let e = &g.entity;
        let t = lines.entry(&e.lineage).or_default();
        let mass = crate::meaning::mass(&crate::anatomy::books(e), rules);
        t.held += mass * u128::from(g.count);
        t.sites.insert(e.place);
        t.oldest = t.oldest.max(s.tick.saturating_sub(e.born));
    }
    let mut out = vec![];
    for (lineage, t) in lines {
        let feats = [
            (ENDURANCE, u128::from(t.oldest)),
            (GROWTH, t.held),
            (SPREAD, t.sites.len() as u128),
        ];
        for (feat, value) in feats.into_iter().filter(|(_, v)| *v > 0) {
            out.push(Reading {
                lineage: lineage.clone(),
                feat: feat.into(),
                value,
            });
        }
    }
    out
}
