// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The viable propagules each lineage holds (case 1193): bodies carrying the
//! world's dormant trait, counted and weighed apart from living adults. A
//! pure function of the state, never kept; empty where the world has no
//! dormancy.

use crate::{schema::*, simulation::Simulation};
use std::collections::BTreeMap;

/// One lineage's dormant stock.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Propagules {
    /// Living propagules.
    pub count: u64,
    /// The matter they carry between them.
    pub matter: u128,
}

/// Each lineage's dormant stock now, lineages with none left out.
pub fn propagules(sim: &Simulation) -> BTreeMap<Key, Propagules> {
    let rules = &sim.genesis().rules;
    let mut out: BTreeMap<Key, Propagules> = BTreeMap::new();
    for g in sim.state().population.groups.values().filter(|g| g.entity.alive && rules.dormant(&g.entity)) {
        let mass = crate::meaning::mass(&crate::anatomy::books(&g.entity), rules);
        let pool: i128 = g.pool.iter().filter(|(k, _)| crate::meaning::matter(rules, k)).map(|(_, v)| *v).sum();
        let tally = out.entry(g.entity.lineage.clone()).or_default();
        tally.count += g.count;
        tally.matter += (mass * u128::from(g.count)).saturating_add_signed(pool);
    }
    out
}
