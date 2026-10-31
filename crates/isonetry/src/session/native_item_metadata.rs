// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Marked physical items require native custody and equipment admission.

use super::*;
use isocosm_campaign::{Inventory, ItemId};
use std::collections::BTreeSet;

fn bound_items(state:&GameSnapshot)->Result<BTreeSet<ItemId>,GameError> {
    if state.native.is_none() {return Ok(BTreeSet::new());}
    let Some(table)=&state.native_table else {return Ok(BTreeSet::new());};
    let game=isocosm_native::native::GameBindings::decode(&table.game).map_err(GameError::Native)?;
    Ok(game.item_records().iter().filter_map(|record|record.binding.as_ref().map(|binding|binding.item.clone())).collect())
}

pub(super) fn transfer(state:&GameSnapshot,item:&ItemId)->Result<(),GameError> {
    if bound_items(state)?.contains(item) {
        return Err(GameError::Native("physically bound items require native custody admission before carrying or transfer".into()));
    }
    Ok(())
}

pub(super) fn inventory(state:&GameSnapshot,token:TokenId,next:&Inventory)->Result<(),GameError> {
    let old=state.inventories.get(&token);
    for item in bound_items(state)? {
        let before=old.and_then(|inventory|inventory.items.get(&item));
        let after=next.items.get(&item);
        let changed_membership=before.is_some()!=after.is_some();
        let before_slots:BTreeSet<_>=old.into_iter().flat_map(|inventory|inventory.equipped.iter())
            .filter_map(|(slot,held)|(held==&item).then_some(slot)).collect();
        let after_slots:BTreeSet<_>=next.equipped.iter().filter_map(|(slot,held)|
            (held==&item).then_some(slot)).collect();
        if changed_membership || before_slots!=after_slots {
            return Err(GameError::Native("physically bound items require native custody and capacity admission before carrying or equipment changes".into()));
        }
    }
    Ok(())
}
