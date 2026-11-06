// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native matter provenance for interpreting actual source-account takes.
//! A selected host's lineage cannot identify foreign or grafted food matter.

use cosmograph::{Session, rules::AccountKind, schema::Key};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Lineage {
    pub parent: Option<Key>,
    pub kingdom: Key,
    pub traits: BTreeSet<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Catalog {
    pub accounts: BTreeMap<Key, AccountKind>,
    pub lineages: BTreeMap<Key, Lineage>,
    pub basis: &'static str,
}

pub fn read(s: &Session) -> Catalog {
    Catalog {
        accounts: s.sim.genesis().rules.accounts.clone(),
        lineages: s
            .sim
            .state()
            .lineages
            .iter()
            .map(|(key, line)| {
                (
                    key.clone(),
                    Lineage {
                        parent: line.parent.clone(),
                        kingdom: line.kingdom.clone(),
                        traits: line.traits.clone(),
                    },
                )
            })
            .collect(),
        basis: "native account Matter.lineage plus saved lineage kingdom, traits and parent; unknown/world provenance remains explicit; selected host lineage is separate",
    }
}
