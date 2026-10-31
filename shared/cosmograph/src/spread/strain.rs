// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Result, rules::Rules, schema::Key, simulation::State};
use std::collections::BTreeSet;

/// An explicit native selected trait set, at a native host or place. No
/// spontaneous mutation rate or generation trigger is inferred here (499).
pub fn strain_key(seed: u64, parent: &str, at: Substrate, traits: &BTreeSet<Key>) -> Key {
    format!("lineage:strain:{}", crate::digest(&(seed, parent, at, traits)))
}

pub fn fork_strain(state: &mut State, rules: &Rules, seed: u64, parent: &str,
    at: Substrate, amount: u64, traits: BTreeSet<Key>) -> Result<Key> {
    owned::operation(state,rules,crate::projects::SpreadDebit::ForkStrain,&Identity::Surface(parent.into()),at,
        |candidate| fork_inner(candidate,rules,seed,parent,at,amount,traits))
}

fn fork_inner(state:&mut State, rules:&Rules, seed:u64, parent:&str, at:Substrate, amount:u64, traits:BTreeSet<Key>) -> Result<Key> {
    if !traits.is_subset(&rules.traits) { return Err("strain selects an unknown trait".into()); }
    let original = Identity::Surface(parent.into());
    if amount == 0 || held(state, at, &original)? < amount { return Err("strain has no selected local living matter".into()); }
    crate::projects::guard_debit(state,crate::projects::debit::substrate_source(at),&original.key(),amount)?;
    let before = state.lineages.get(parent).ok_or("unknown strain parent")?;
    let key = strain_key(seed, parent, at, &traits);
    if rules.accounts.contains_key(&key) { return Err("strain key aliases an ordinary account".into()); }
    let child = crate::schema::Lineage { parent: Some(parent.into()), traits,
        revision: before.revision.checked_add(1).ok_or("strain revision overflows")?, ..before.clone() };
    if state.lineages.get(&key).is_some_and(|present| present != &child) {
        return Err("strain identity already has different native genes".into());
    }
    let mut candidate = state.clone();
    let target = Identity::Surface(key.clone());
    crate::meaning::debit(stock::ledger_mut(&mut candidate, at)?, &original.key(), amount)?;
    crate::meaning::credit(stock::ledger_mut(&mut candidate, at)?, &target.key(), amount)?;
    candidate.lineages.insert(key.clone(), child);
    candidate.spreads.identities.insert(target.clone());
    owned::strain_output(&candidate,at,target,amount)?;
    *state = candidate;
    Ok(key)
}
