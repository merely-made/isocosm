// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Monthly native ledgers of registered rooted tissue. Cohort remainders
//! stay explicit; they cannot be assigned to a representative body's root.

use cosmograph::{Session, anatomy, schema::Key};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Tissue {
    pub members: u64,
    pub root_matter: u128,
    pub exposed_matter: u128,
    pub other_part_matter: u128,
    pub entity_ledger_matter: u128,
    pub unassigned_cohort_pool: i128,
}

/// Outer key is the current lineage, inner key the tissue's saved provenance.
pub fn read(s: &Session) -> BTreeMap<Key, BTreeMap<Key, Tissue>> {
    let rules = &s.sim.genesis().rules;
    let mut out: BTreeMap<Key, BTreeMap<Key, Tissue>> = BTreeMap::new();
    for group in s.sim.state().population.groups.values() {
        let e = &group.entity;
        if !e.alive {
            continue;
        }
        for account in rules.root_stores.keys() {
            let Some((_, root)) = anatomy::root_store::contract(e, rules, account) else {
                continue;
            };
            let t = out
                .entry(e.lineage.clone())
                .or_default()
                .entry(account.clone())
                .or_default();
            t.members += group.count;
            let copies = u128::from(group.count);
            for (&id, part) in &e.parts {
                let matter = u128::from(part.matter.get(account).copied().unwrap_or(0)) * copies;
                if id == root {
                    t.root_matter += matter;
                } else if anatomy::root_store::exposed(e, rules, id) {
                    t.exposed_matter += matter;
                } else {
                    t.other_part_matter += matter;
                }
            }
            t.entity_ledger_matter +=
                u128::from(e.accounts.get(account).copied().unwrap_or(0)) * copies;
            t.unassigned_cohort_pool += group.pool.get(account).copied().unwrap_or(0);
        }
    }
    out
}
