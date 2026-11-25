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
pub const SUPPLIERS: &str = "feat:neighbouring-mineral-suppliers";
pub const EXPORTERS: &str = "feat:neighbouring-sugar-exporters";

/// Other lineages with complementary live organs somewhere in this line's
/// occupied range. Counts describe opportunities, before price and stock.
pub(super) fn neighbours(sim: &Simulation, lineage: &str) -> Vec<Reading> {
    use crate::{ecology::trades, process::Trade};
    let residents = || sim.state().population.groups.values().map(|g| &g.entity).filter(|e| e.alive);
    let range: BTreeSet<_> = residents().filter(|e| e.lineage == lineage)
        .filter_map(|e| e.resident_site()).collect();
    let mut suppliers = BTreeSet::new();
    let mut exporters = BTreeSet::new();
    for e in residents().filter(|e| e.lineage != lineage && e.resident_site().is_some_and(|site| range.contains(&site))) {
        if trades(e, Trade::GivesNutrient) && trades(e, Trade::TakesSugar) {
            suppliers.insert(&e.lineage);
        }
        if trades(e, Trade::GivesSugar) && crate::trophic::fixing_plate(e).is_some() {
            exporters.insert(&e.lineage);
        }
    }
    [(SUPPLIERS, suppliers.len()), (EXPORTERS, exporters.len())].into_iter()
        .map(|(feat, n)| Reading { lineage: lineage.to_owned(), feat: feat.into(), value: n as u128 }).collect()
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
