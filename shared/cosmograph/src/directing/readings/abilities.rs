// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a body's parts let it do, grown or borrowed (cases 1233 and 1243).
//! A pure reading of its living parts; a borrowed ability ends with its part.

use crate::{matter::scruple, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A function a body expresses, and whether only borrowed parts carry it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ability {
    pub function: Key,
    pub cells: u32,
    pub borrowed: bool,
    pub from: BTreeSet<Key>,
}

/// Whether most of `part`'s matter is another lineage's, and whose; a part
/// holding no matter is its body's own.
fn donors(sim: &Simulation, own: &str, part: &Part) -> Option<BTreeSet<Key>> {
    let mix = scruple::of(part, &sim.genesis().rules).ok()?;
    let total: u128 = mix.values().map(|mg| u128::from(*mg)).sum();
    let theirs: u128 = mix.iter().filter(|(line, _)| *line != own).map(|(_, mg)| u128::from(*mg)).sum();
    (theirs * 2 > total).then(|| mix.into_keys().filter(|line| line != own).collect())
}

/// The functions `critter`'s living parts express, in function-key order
/// (cases 1233 and 1243): the cells they hold in all, whether every part
/// expressing one is borrowed, and the lineages those lend from.
pub fn abilities(sim: &Simulation, critter: Id) -> Vec<Ability> {
    let Some(body) = sim.state().population.get(critter) else {
        return vec![];
    };
    let mut found = BTreeMap::<&Key, Ability>::new();
    for (_, part) in body.living() {
        let lent = donors(sim, &body.lineage, part);
        for function in &part.functions {
            let ability = found.entry(function).or_insert_with(|| Ability {
                function: function.clone(),
                cells: 0,
                borrowed: true,
                from: BTreeSet::new(),
            });
            ability.cells += part.cells.get(function).copied().unwrap_or(0);
            ability.borrowed &= lent.is_some();
            ability.from.extend(lent.iter().flatten().cloned());
        }
    }
    found.into_values().collect()
}
