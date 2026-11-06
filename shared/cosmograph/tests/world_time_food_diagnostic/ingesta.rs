// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native registered digestion stores, separate from feeding throughput.
//! Dead bodies and signed cohort pools remain explicit physical holdings.

use cosmograph::{
    Session,
    schema::{Key, PartId},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Store {
    pub living_members: u64,
    pub dead_members: u64,
    pub living_part_matter: u128,
    pub dead_part_matter: u128,
    pub living_entity_ledger_matter: u128,
    pub dead_entity_ledger_matter: u128,
    pub living_unassigned_cohort_pool: i128,
    pub dead_unassigned_cohort_pool: i128,
    /// Part ids are local anatomy addresses, grouped here by current lineage.
    pub living_part_locations: BTreeMap<PartId, u128>,
    pub dead_part_locations: BTreeMap<PartId, u128>,
}

pub type Stores = BTreeMap<Key, BTreeMap<Key, Store>>;

pub fn read(s: &Session) -> Stores {
    let accounts: BTreeSet<_> = s
        .sim
        .genesis()
        .rules
        .processes
        .values()
        .filter_map(|p| {
            p.port_feeding
                .as_ref()?
                .meal
                .as_ref()
                .map(|m| m.ingesta.clone())
        })
        .collect();
    let mut out = Stores::new();
    for group in s.sim.state().population.groups.values() {
        let e = &group.entity;
        if e.kingdom == "kingdom:world" {
            continue;
        }
        for key in &accounts {
            let t = out
                .entry(e.lineage.clone())
                .or_default()
                .entry(key.clone())
                .or_default();
            let (members, parts, ledger, pool, locations) = if e.alive {
                (
                    &mut t.living_members,
                    &mut t.living_part_matter,
                    &mut t.living_entity_ledger_matter,
                    &mut t.living_unassigned_cohort_pool,
                    &mut t.living_part_locations,
                )
            } else {
                (
                    &mut t.dead_members,
                    &mut t.dead_part_matter,
                    &mut t.dead_entity_ledger_matter,
                    &mut t.dead_unassigned_cohort_pool,
                    &mut t.dead_part_locations,
                )
            };
            *members += group.count;
            let copies = u128::from(group.count);
            for (&id, part) in &e.parts {
                let held = u128::from(part.matter.get(key).copied().unwrap_or(0)) * copies;
                *parts += held;
                if held > 0 {
                    *locations.entry(id).or_default() += held;
                }
            }
            *ledger += u128::from(e.accounts.get(key).copied().unwrap_or(0)) * copies;
            *pool += group.pool.get(key).copied().unwrap_or(0);
        }
    }
    out
}
