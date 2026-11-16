// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Fused runs share one reserve (D44, D50). The grammar's `Fuse` and
//! `Radiate { fused }` give runs a pool; the parts of a pooled run, with what
//! they bear, hold the pool's reserve as one: each part's share of it is its
//! share of what the pool's parts can store, so the total is untouched and a
//! part lost takes only its own share with it.

use crate::{
    anatomy,
    rules::{Development, Rules},
    schema::*,
};
use std::collections::BTreeMap;

/// Redistributes the `reserve` held by each pool's parts over them by what
/// each can store; returns how much moved between parts.
pub fn share(rules: &Rules, d: &Development, e: &mut Entity, reserve: &Key) -> u64 {
    let mut pools: BTreeMap<u8, Vec<PartId>> = BTreeMap::new();
    for (id, _) in e.living() {
        let pool = e.situs(id).and_then(|s| d.recipe.tagmata.get(usize::from(s[0]))?.pool);
        if let Some(pool) = pool {
            pools.entry(pool).or_default().push(id);
        }
    }
    let mut moved = 0;
    for ids in pools.values().filter(|ids| ids.len() > 1) {
        let held = |e: &Entity, id: &PartId| e.parts[id].matter.get(reserve).copied().unwrap_or(0);
        let total: u64 = ids.iter().map(|id| held(e, id)).sum();
        let weights: Vec<(PartId, u64)> = ids
            .iter()
            .map(|id| (*id, held(e, id) + anatomy::space(e.extent(*id), &e.parts[id], rules, reserve)))
            .collect();
        let shares: BTreeMap<PartId, u64> = anatomy::apportion(&weights, total).into_iter().collect();
        for id in ids {
            let now = shares.get(id).copied().unwrap_or(0);
            moved += held(e, id).abs_diff(now);
            let Some(part) = e.part_mut(id) else { continue };
            match now {
                0 => part.matter.remove(reserve),
                n => part.matter.insert(reserve.clone(), n),
            };
        }
    }
    moved / 2
}
