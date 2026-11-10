// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What the world's conditions raised between two moments (case 1216): a
//! pure reading of two simulations, never kept. Participants and lineages
//! of the world kingdom are left out; they are not living populations.

use super::regions::regions;
use crate::{schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The account the ecology treats as soil.
const SOIL: &str = "world:soil";

/// Something the world's conditions raised between two moments (case 1216).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Raised {
    /// A lineage that had living members has none.
    Extinct { lineage: Key },
    /// A region that stood has collapsed.
    Fell { sites: BTreeSet<Id> },
    /// A site's soil ran out.
    Barren { site: Id },
    /// A lineage's living matter at least doubled.
    Bloomed { lineage: Key, held: u128 },
    /// The year turned from one half to the other at the equator's reckoning.
    Turned { rising: bool },
}

/// Each living lineage's matter and members, as `boundary::standing` counts them.
fn living(sim: &Simulation) -> BTreeMap<Key, (u128, u64)> {
    let state = sim.state();
    let rules = &sim.genesis().rules;
    let mut by: BTreeMap<Key, (u128, u64)> = BTreeMap::new();
    for g in state.population.groups.values().filter(|g| g.entity.alive) {
        let line = &g.entity.lineage;
        if state.lineages.get(line).is_none_or(|l| l.kingdom == "kingdom:world") { continue; }
        let mass = crate::meaning::mass(&crate::anatomy::books(&g.entity), rules);
        let pool: i128 = g.pool.iter().filter(|(k, _)| crate::meaning::matter(rules, k))
            .map(|(_, v)| *v).sum();
        let held = (mass * u128::from(g.count)).saturating_add_signed(pool);
        let tally = by.entry(line.clone()).or_default();
        tally.0 += held;
        tally.1 += g.count;
    }
    by
}

/// Whether the year stands in its rising half at `tick`: the seasonal wave
/// is not below nought. A tidally locked sun never leaves one half.
fn rising(sim: &Simulation, tick: Tick) -> bool {
    crate::climate::wave(&sim.genesis().rules, tick) >= 0
}

/// What the world raised from `before` to `after`, in the order of the
/// variants, then by key (case 1216). The year's turn is read from the two
/// ticks alone, so a span longer than half a year may hide turns.
pub fn raised(before: &Simulation, after: &Simulation) -> Vec<Raised> {
    let mut out = Vec::new();
    let (was, is) = (living(before), living(after));
    for line in was.iter().filter(|(_, (_, members))| *members > 0).map(|(line, _)| line) {
        if is.get(line).is_none_or(|(_, members)| *members == 0) {
            out.push(Raised::Extinct { lineage: line.clone() });
        }
    }
    let stood: Vec<_> = regions(before).into_iter().filter(|r| r.collapsed).collect();
    for region in regions(after).into_iter().filter(|r| r.collapsed) {
        if !stood.iter().any(|old| !old.sites.is_disjoint(&region.sites)) {
            out.push(Raised::Fell { sites: region.sites });
        }
    }
    let soil = |sim: &Simulation, site: &Id| {
        sim.state().sites.get(site).and_then(|s| s.accounts.get(SOIL)).copied().unwrap_or(0)
    };
    for site in after.state().sites.keys() {
        if soil(before, site) > 0 && soil(after, site) == 0 { out.push(Raised::Barren { site: *site }); }
    }
    for (line, (held, _)) in &is {
        let prior = was.get(line).map_or(0, |(held, _)| *held);
        if prior > 0 && *held >= prior.saturating_mul(2) {
            out.push(Raised::Bloomed { lineage: line.clone(), held: *held });
        }
    }
    let (from, to) = (rising(before, before.state().tick), rising(after, after.state().tick));
    if from != to { out.push(Raised::Turned { rising: to }); }
    out
}
