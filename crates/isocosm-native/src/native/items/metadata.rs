// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::GameBindings;
use isocosm_campaign::Inventory;
use std::collections::BTreeSet;

/// Explicit physical item identities leave legacy slot/modifier arithmetic.
/// Native process effects, custody and mass remain native readings.
pub fn unmarked_inventory(game:&GameBindings,inventory:&Inventory)->Inventory {
    let marked:BTreeSet<_>=game.item_records().iter().filter_map(|record|record.binding.as_ref())
        .map(|binding|binding.item.clone()).collect();
    let mut remaining=inventory.clone();
    remaining.items.retain(|key,item|!marked.contains(key) && !marked.contains(&item.id));
    remaining.equipped.retain(|_,item|remaining.items.contains_key(item));
    remaining
}
