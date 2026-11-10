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

/// Turn traits on and off on a surface lineage in place; no matter moves
/// (case 1223).
///
/// # Errors
/// The lineage is not a registered surface, a trait to turn on is unknown,
/// the two sets overlap, or the switch changes nothing.
pub fn switch(state: &mut State, rules: &Rules, lineage: &str, on: &BTreeSet<Key>,
    off: &BTreeSet<Key>) -> Result<String> {
    if !state.spreads.identities.contains(&Identity::Surface(lineage.into())) {
        return Err("switch names no registered surface lineage".into());
    }
    if !on.is_subset(&rules.traits) { return Err("switch turns on an unknown trait".into()); }
    if !on.is_disjoint(off) { return Err("switch turns a trait both on and off".into()); }
    let present = state.lineages.get_mut(lineage).ok_or("unknown switch lineage")?;
    let mut traits = present.traits.clone();
    traits.extend(on.iter().cloned());
    traits.retain(|key| !off.contains(key));
    if traits == present.traits { return Err("switch changes no trait".into()); }
    present.revision = present.revision.checked_add(1).ok_or("switch revision overflows")?;
    present.traits = traits;
    Ok("switched".into())
}
