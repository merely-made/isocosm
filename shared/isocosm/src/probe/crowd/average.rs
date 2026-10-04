// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The negative control (rulings 209 and 507): the averaged crowd, every
//! class of living members at a site holding its class's average of each
//! account its lineage owns, filled to one level as far as each body may
//! hold it. Kin are averaged as members of one.

use super::{Crowd, Who, normalize};
use crate::{rules::AccountKind, schema::*};
use std::collections::BTreeMap;

/// A class's members: each state, how many it holds, and whom it is.
type Class = Vec<(Entity, u64, Who)>;

impl Crowd<'_> {
    /// The negative control (ruling 209): every class of living members at
    /// a site has what it holds of an account replaced by the class's
    /// average. The account is the body a competition sizes a kind up by,
    /// or, in a world without competitions, each of a lineage's own matter
    /// accounts in turn, tissue and reserve alike (ruling 507).
    pub(super) fn average(&mut self) {
        let kinds = self.world.kinds();
        let rules = &self.world.genesis.rules;
        let averaged = |e: &Entity| -> Vec<Key> {
            if kinds.is_empty() {
                let own = |k: &AccountKind| matches!(k, AccountKind::Matter { lineage, .. } if *lineage == e.lineage);
                let found = rules.accounts.iter().filter(|(_, k)| own(k));
                found.map(|(key, _)| key.clone()).collect()
            } else {
                let kind = kinds.iter().find(|k| e.traits.contains(&k.identity));
                kind.map(|k| k.body.clone()).into_iter().collect()
            }
        };
        let all = self.bins.keys().chain(self.kin.values());
        let accounts = all.map(|e| averaged(e).len()).max();
        for which in 0..accounts.unwrap_or(0) {
            let mut classes: BTreeMap<(Key, Id), Class> = BTreeMap::new();
            let bins = self.bins.iter().map(|(e, &n)| (e, n, Who::Bin));
            let kin = self.kin.iter().map(|(id, e)| (e, 1, Who::Kin(*id)));
            for (e, n, who) in bins.chain(kin) {
                if let Some(key) = averaged(e).into_iter().nth(which).filter(|_| e.alive) {
                    let class = classes.entry((key, e.place)).or_default();
                    class.push((e.clone(), n, who));
                }
            }
            for ((key, _), members) in classes {
                average_class((&mut self.bins, &mut self.kin), rules, &key, &members);
            }
        }
    }
}

/// One class's average of `key` set into its bins and kin. Each member keeps
/// everything else. A body keeping matter in parts holds only what its
/// parts may hold, so the class is filled to one level as far as each
/// member's bound allows, the remainder a unit each to members with room
/// above it, and nothing is lost; bodies alike in their parts all hold the
/// average.
fn average_class(
    (bins, kin): (&mut BTreeMap<Entity, u64>, &mut BTreeMap<Id, Entity>),
    rules: &crate::rules::Rules,
    key: &Key,
    members: &[(Entity, u64, Who)],
) {
    let total: u64 = members
        .iter()
        .map(|(e, m, _)| crate::anatomy::held(e, rules, key) * m)
        .sum();
    for (e, m, who) in members {
        if let Who::Bin = who {
            let slot = bins.get_mut(e).expect("member bin exists");
            *slot -= m;
            if *slot == 0 {
                bins.remove(e);
            }
        }
    }
    // Kin are written back to their own identity, a bin's members to bins.
    let mut place = |e: Entity, count: u64, who: Who| match who {
        Who::Bin => *bins.entry(normalize(e)).or_default() += count,
        Who::Kin(id) => {
            kin.insert(id, normalize(e));
        },
    };
    // What each member may hold of the key: its parts' bound, or without
    // parts, anything.
    let cap = |e: &Entity| match crate::anatomy::anatomical(e, rules, key) {
        true => crate::anatomy::held(e, rules, key) + crate::anatomy::room(e, rules, key),
        false => u64::MAX,
    };
    let fill = |level: u64| -> u128 {
        let each = members
            .iter()
            .map(|(e, m, _)| u128::from(cap(e).min(level)) * u128::from(*m));
        each.sum()
    };
    // The highest level the class's total fills (507).
    let (mut lo, mut hi) = (0u64, total);
    while lo < hi {
        let mid = lo + (hi - lo).div_ceil(2);
        match fill(mid) <= u128::from(total) {
            true => lo = mid,
            false => hi = mid - 1,
        }
    }
    let mut extra = u128::from(total) - fill(lo);
    for (e, m, who) in members {
        let room_above = cap(e) > lo;
        let high = if room_above {
            extra.min(u128::from(*m)) as u64
        } else {
            0
        };
        extra -= u128::from(high);
        let base = cap(e).min(lo);
        for (amount, count) in [(base + 1, high), (base, m - high)] {
            if count > 0 {
                let (e, left) = averaged_into(e, rules, key, amount);
                debug_assert_eq!(left, 0, "filled within its bound");
                place(e, count, *who);
            }
        }
    }
    debug_assert_eq!(extra, 0, "an average fits its class");
}

/// `e` holding `amount` of `key`: in its ledger, or for a body keeping
/// matter in parts emptied from them and given back up to their room.
/// Returns it and what did not fit.
fn averaged_into(e: &Entity, rules: &crate::rules::Rules, key: &Key, amount: u64) -> (Entity, u64) {
    let mut e = e.clone();
    if !crate::anatomy::anatomical(&e, rules, key) {
        e.accounts.insert(key.clone(), amount);
        return (e, 0);
    }
    for part in e.parts.values_mut() {
        part.matter.remove(key);
    }
    let fits = amount.min(crate::anatomy::room(&e, rules, key));
    crate::anatomy::give(&mut e, rules, key, fits)
        .expect("the key lives in parts")
        .expect("given within room");
    (e, amount - fits)
}
