// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Rooted tissue uses native part ledgers (1110, 1114). The allocation is
//! saved world data; geometry decides bite exposure, never the store role.

use super::*;
use crate::rules::RootStore;
use std::collections::BTreeMap;

/// The adult shares of this actual body, with its dedicated store root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allocation {
    pub rule: RootStore,
    pub store: PartId,
    pub exposed: BTreeMap<PartId, u64>,
}

pub(super) fn above(e: &Entity, id: PartId, ground_y: i32) -> bool {
    let Some(doc) = &e.body else { return false };
    let Some(pivot) = doc.world_pivot(id) else {
        return false;
    };
    i64::from(pivot[1]) + i64::from(e.extent(id)[1].unsigned_abs()) > i64::from(ground_y)
}

/// A registered tissue's own root, also valid after its body speciates:
/// tissue keeps its source provenance while the body gains a new lineage.
pub fn contract<'r>(e: &Entity, rules: &'r Rules, key: &str) -> Option<(&'r RootStore, PartId)> {
    let rule = rules.root_stores.get(key)?;
    if !e.traits.contains(&rule.identity) {
        return None;
    }
    let store = e.body.as_ref()?.root;
    let root = e.parts.get(&store).filter(|_| e.lives(store))?;
    if root.cells.get(STORE).copied().unwrap_or(0) == 0 {
        return None;
    }
    Some((rule, store))
}

pub fn allocation(e: &Entity, rules: &Rules, key: &str) -> Option<Allocation> {
    let (rule, store) = contract(e, rules, key)?;
    let weights: Vec<_> = bodies(e)
        .filter(|(id, _, _)| *id != store && above(e, *id, rule.ground_y))
        .map(|(id, h, _)| (id, ceiling(h, rules.body())))
        .collect();
    let exposed = apportion(&weights, rule.exposed).into_iter().collect();
    Some(Allocation {
        rule: rule.clone(),
        store,
        exposed,
    })
}

/// Whether the part reaches above the rooted body's saved ground plane.
/// Bodies without a registered rooted allocation retain their old draw.
pub fn exposed(e: &Entity, rules: &Rules, id: PartId) -> bool {
    ground(e, rules).is_none_or(|ground_y| above(e, id, ground_y))
}

pub(super) fn ground(e: &Entity, rules: &Rules) -> Option<i32> {
    rules
        .root_stores
        .keys()
        .find_map(|key| contract(e, rules, key).map(|(r, _)| r.ground_y))
}

fn held_at(e: &Entity, id: PartId, key: &str) -> u64 {
    e.parts
        .get(&id)
        .and_then(|p| p.matter.get(key))
        .copied()
        .unwrap_or(0)
}

fn spaces(e: &Entity, rules: &Rules, key: &str, a: &Allocation) -> Vec<(PartId, u64)> {
    let mut spaces: Vec<_> = a
        .exposed
        .iter()
        .map(|(&id, &share)| {
            let p = &e.parts[&id];
            (
                id,
                share
                    .saturating_sub(held_at(e, id, key))
                    .min(space(e.extent(id), p, rules, key)),
            )
        })
        .collect();
    // The most depleted exposed share fills first; part order breaks ties.
    spaces.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(&y.0)));
    let p = &e.parts[&a.store];
    let room = a
        .rule
        .stored
        .saturating_add(a.rule.seed)
        .saturating_sub(held_at(e, a.store, key))
        .min(space(e.extent(a.store), p, rules, key));
    spaces.push((a.store, room));
    spaces
}

pub(super) fn room(e: &Entity, rules: &Rules, key: &str, caps: Option<&Caps>) -> Option<u64> {
    let a = allocation(e, rules, key)?;
    Some(
        spaces(e, rules, key, &a)
            .into_iter()
            .map(|(id, room)| capped(room, id, caps))
            .fold(0, u64::saturating_add),
    )
}

pub(super) fn give(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
    mut caps: Option<&mut Caps>,
) -> Option<Result<Vec<(PartId, u64)>>> {
    let a = allocation(e, rules, key)?;
    let spaces: Vec<_> = spaces(e, rules, key, &a)
        .into_iter()
        .map(|(id, room)| (id, capped(room, id, caps.as_deref())))
        .collect();
    if spaces.iter().map(|(_, n)| u128::from(*n)).sum::<u128>() < u128::from(amount) {
        return Some(Err(format!(
            "no room in the rooted body for {amount} of {key}"
        )));
    }
    let mut left = amount;
    let mut split = vec![];
    for (id, room) in spaces {
        let n = room.min(left);
        if n > 0 {
            split.push((id, n));
        }
        left -= n;
        if left == 0 {
            break;
        }
    }
    if let Some(caps) = caps.as_deref_mut() {
        for (id, n) in &split {
            *caps.entry(*id).or_default() -= n;
        }
    }
    Some(apply(e, key, &split, credit).map(|()| split))
}

/// Internal draws spend the store. Grazing debits the geometrically
/// selected part directly, so this never turns exposed tissue into food.
pub(super) fn take(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
    caps: Option<&mut Caps>,
) -> Option<Result<Vec<(PartId, u64)>>> {
    let (_, store) = contract(e, rules, key)?;
    if held_at(e, store, key) < amount {
        return Some(Err(format!("insufficient {key} in the root store")));
    }
    let split = if amount > 0 {
        vec![(store, amount)]
    } else {
        vec![]
    };
    if let Some(caps) = caps {
        let slot = caps.entry(store).or_default();
        *slot = slot.saturating_add(amount);
    }
    Some(apply(e, key, &split, debit).map(|()| split))
}

/// Founder and seed allocation: the stored fraction is paid from the
/// supplied tissue, with the rest filling exposed shares. No matter is made.
pub(crate) fn initialize(
    e: &mut Entity,
    rules: &Rules,
    key: &str,
    amount: u64,
) -> Option<Result<Vec<(PartId, u64)>>> {
    let a = allocation(e, rules, key)?;
    let adult = u128::from(a.rule.stored) + u128::from(a.rule.exposed);
    let stored = (u128::from(amount) * u128::from(a.rule.stored) / adult) as u64;
    let stored = stored.min(
        a.rule
            .stored
            .saturating_add(a.rule.seed)
            .saturating_sub(held_at(e, a.store, key)),
    );
    let p = &e.parts[&a.store];
    let stored = stored.min(space(e.extent(a.store), p, rules, key));
    if room(e, rules, key, None)? < amount {
        return Some(Err(format!(
            "no room for rooted founder or seed's {amount} of {key}"
        )));
    }
    let split = vec![(a.store, stored)];
    Some(apply(e, key, &split, credit).and_then(|()| {
        let mut given =
            give(e, rules, key, amount - stored, None).ok_or("rooted allocation disappeared")??;
        given.extend(split.into_iter().filter(|(_, n)| *n > 0));
        Ok(given)
    }))
}

/// How much a conservative regrowth can move now, limited by the store
/// and the actual body's exposed deficit rather than its total biomass.
pub(crate) fn regrowth(e: &Entity, rules: &Rules, key: &str, amount: u64) -> Result<u64> {
    let a = allocation(e, rules, key).ok_or("regrowth needs a rooted body")?;
    let missing = spaces(e, rules, key, &a)
        .into_iter()
        .filter(|(id, _)| *id != a.store)
        .map(|(_, n)| n)
        .fold(0, u64::saturating_add);
    let moveable = amount.min(held_at(e, a.store, key)).min(missing);
    if moveable == 0 {
        return Err("no stored tissue or exposed deficit to regrow".into());
    }
    Ok(moveable)
}
