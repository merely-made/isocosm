// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Healing pays growth's cell price from reserve then tissue, before the
//! meal fills tissue or develops new parts (496, 712 and 810). A rest heals
//! one cell at a time, in any lineage, for a quarter of that price, or an
//! eighth where the lineage heals (the 2026-10-18 note to ruling 227).

use super::{Parties, spend};
use crate::{
    Result, anatomy,
    harm::{HEAL, healing},
    rules::*,
    schema::*,
};

/// What a rest leaves its body: the starving line, 2 mg, which the probe's
/// mind reads as a body one tick from death (below 1 it dies, `ecology:death`).
pub(crate) const REST_LEAVES: u64 = 2;

/// What restoring a cell of half-extent `h` costs: its mass; at rest a
/// quarter of it, or an eighth where the lineage `heals`, rounded up, a
/// milligram at least.
pub(crate) fn price(h: anatomy::Half, rules: &Rules, rest: bool, heals: bool) -> u64 {
    let mass = anatomy::cell_mass(h, rules.body());
    match (rest, heals) {
        (false, _) => mass,
        (true, false) => mass.div_ceil(4).max(1),
        (true, true) => mass.div_ceil(8).max(1),
    }
}

pub(crate) fn heal(p: &mut impl Parties, rules: &Rules, into: &Key, rest: bool) -> Result<Ledger> {
    let mut paid = Ledger::new();
    let lineage = p.body(Binding::Actor)?.lineage.clone();
    let heals = p.lineage_trait(&lineage, HEAL);
    if !rest && !heals {
        return Ok(paid);
    }
    let anatomy::Own {
        tissue, reserve, ..
    } = anatomy::own(rules, &lineage);
    let pay: Vec<Key> = reserve.into_iter().chain(tissue).collect();
    loop {
        let body = p.body(Binding::Actor)?;
        let Some((id, cell)) = healing::next(body) else {
            break;
        };
        let price = price(body.extent(id), rules, rest, heals);
        let funds = pay
            .iter()
            .map(|k| anatomy::held(body, rules, k))
            .fold(0, u64::saturating_add);
        if funds < price + if rest { REST_LEAVES } else { 0 } {
            break;
        }
        for (key, n) in spend(p, &pay, Binding::Place, Some(into), price)? {
            *paid.entry(key).or_default() += n;
        }
        healing::restore(p.body(Binding::Actor)?, id, cell)?;
        if rest {
            break;
        }
    }
    Ok(paid)
}
