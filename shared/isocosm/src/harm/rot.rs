// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Result,
    meaning::{credit, debit},
    rules::Rules,
    schema::*,
};

pub(crate) fn rot(
    e: &mut Entity,
    rules: &Rules,
    amount: u64,
) -> Result<Vec<(Option<PartId>, Ledger)>> {
    if e.alive {
        return Err("rot requires a dead body".into());
    }
    let taken = crate::meaning::share(&crate::anatomy::books(e), rules, amount);
    let mut moved = vec![];
    for (key, mut left) in taken {
        let own = e.accounts.get(&key).copied().unwrap_or(0).min(left);
        if own > 0 {
            debit(&mut e.accounts, &key, own)?;
            moved.push((None, Ledger::from([(key.clone(), own)])));
            left -= own;
        }
        for (id, p) in &mut e.parts {
            let n = p.matter.get(&key).copied().unwrap_or(0).min(left);
            if n > 0 {
                debit(&mut p.matter, &key, n)?;
                moved.push((Some(*id), Ledger::from([(key.clone(), n)])));
                left -= n;
            }
        }
        if left > 0 {
            return Err("rot did not reach all its matter".into());
        }
    }
    Ok(moved)
}

pub(crate) fn deposit(accounts: &mut Ledger, moved: &[(Option<PartId>, Ledger)]) -> Result<()> {
    for (_, ledger) in moved {
        for (key, n) in ledger {
            credit(accounts, key, *n)?;
        }
    }
    Ok(())
}

pub(crate) fn legs(
    entity: Id,
    site: Id,
    moved: &[(Option<PartId>, Ledger)],
) -> Vec<crate::flows::Leg> {
    use crate::flows::{Holder, Leg};
    moved
        .iter()
        .flat_map(|(part, l)| {
            let from = part.map_or(Holder::Entity(entity), |p| Holder::Part(entity, p));
            l.iter().map(move |(key, n)| Leg {
                from: (from, key.clone()),
                to: (Holder::Site(site), key.clone()),
                amount: *n,
            })
        })
        .collect()
}
