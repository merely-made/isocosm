// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Healing pays growth's cell price from reserve then tissue, before the
//! meal fills tissue or develops new parts (496, 712 and 810).

use super::{Parties, spend};
use crate::{
    Result, anatomy,
    harm::{HEAL, healing},
    rules::*,
    schema::*,
};

pub(crate) fn heal(p: &mut impl Parties, rules: &Rules, into: &Key) -> Result<Ledger> {
    let mut paid = Ledger::new();
    let lineage = p.body(Binding::Actor)?.lineage.clone();
    if !p.lineage_trait(&lineage, HEAL) {
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
        let price = u64::from(anatomy::cell_mass(body.extent(id), rules.body()));
        let funds = pay
            .iter()
            .map(|k| anatomy::held(body, rules, k))
            .fold(0, u64::saturating_add);
        if funds < price {
            break;
        }
        for (key, n) in spend(p, &pay, Binding::Place, Some(into), price)? {
            *paid.entry(key).or_default() += n;
        }
        healing::restore(p.body(Binding::Actor)?, id, cell)?;
    }
    Ok(paid)
}
