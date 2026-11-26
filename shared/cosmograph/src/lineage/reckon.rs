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
/// The readings of who a line could trade with: this, then a good's key.
pub const PARTNERS: &str = "feat:partners:";

/// For each good a member of `lineage` could be given in a settled exchange
/// where it now lives, how many other lineages there could give it: what a
/// body's organs make possible, before price and stock.
pub(super) fn neighbours(sim: &Simulation, lineage: &str) -> Vec<Reading> {
    let rules = &sim.genesis().rules;
    let residents: Vec<_> = sim.state().population.groups.values().map(|g| &g.entity).filter(|e| e.alive).collect();
    let mut givers: BTreeMap<&str, BTreeSet<&Key>> = BTreeMap::new();
    for mine in residents.iter().filter(|e| e.lineage == lineage) {
        let Some(site) = mine.resident_site() else { continue };
        for other in residents.iter().filter(|e| e.lineage != lineage && e.resident_site() == Some(site)) {
            let got = [
                crate::ecology::bargain(rules, mine, other).map(|[_, drawn]| drawn),
                crate::ecology::bargain(rules, other, mine).map(|[paid, _]| paid),
            ];
            for good in got.into_iter().flatten().flatten() {
                givers.entry(good).or_default().insert(&other.lineage);
            }
        }
    }
    let reading = |(good, lines): (&str, BTreeSet<&Key>)| Reading {
        lineage: lineage.to_owned(),
        feat: format!("{PARTNERS}{good}"),
        value: lines.len() as u128,
    };
    givers.into_iter().map(reading).collect()
}

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
        t.sites.extend(e.resident_site());
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
