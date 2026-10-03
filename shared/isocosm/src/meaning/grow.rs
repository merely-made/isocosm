// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Growth with the parts full grows the parts a body lacks (rulings 479
//! and 510): one at a time, nearest the root first, each paying PD2's price
//! from the reserve and then the tissue into the ground, and then filled by
//! what the act holds in hand, given to the parts by their room (464).

use super::{Parties, convert, spend};
use crate::{Result, anatomy, growth, rules::*, schema::*};

/// What growth moved: what each account paid toward the prices, and what
/// of the hand each conversion took.
#[derive(Default)]
pub(crate) struct Grown {
    pub paid: Ledger,
    pub taken: Vec<(Ledger, Key, u64)>,
}

/// The actor's own tissue and reserve accounts.
fn own(rules: &Rules, lineage: &str) -> (Option<Key>, Option<Key>) {
    let mut tissue = None;
    let mut reserve = None;
    for (key, kind) in &rules.accounts {
        if let AccountKind::Matter {
            lineage: l,
            reserve: r,
            provision: false,
        } = kind
            && l == lineage
        {
            match r {
                true => reserve = reserve.or(Some(key.clone())),
                false => tissue = tissue.or(Some(key.clone())),
            }
        }
    }
    (tissue, reserve)
}

pub(crate) fn grow(
    p: &mut impl Parties,
    rules: &Rules,
    from: &Key,
    into: &Key,
    conversion: Conversion,
) -> Result<Grown> {
    let mut grown = Grown::default();
    let lineage = p.body(Binding::Actor)?.lineage.clone();
    let development = p.development(&lineage)?;
    let (Some(tissue), reserve) = own(rules, &lineage) else {
        return Err(format!("{lineage} has no tissue to grow"));
    };
    let pay: Vec<Key> = reserve.into_iter().chain([tissue.clone()]).collect();
    loop {
        let hand = p.held(Binding::Actor, from)?;
        if hand == 0 {
            break;
        }
        let body = p.body(Binding::Actor)?;
        // Only growth that finds no room left in the parts grows a part.
        if anatomy::room(body, rules, &tissue) > 0 {
            break;
        }
        let Some(part) = growth::lacking(rules, &development, body)? else {
            break;
        };
        let price = growth::price(rules, &part);
        let funds: u64 = pay
            .iter()
            .map(|k| anatomy::held(body, rules, k))
            .fold(0, u64::saturating_add);
        if funds < price {
            break;
        }
        let id = body.parts.keys().next_back().map_or(0, |last| last + 1);
        body.parts.insert(id, part);
        for (key, paid) in spend(p, &pay, Binding::Place, Some(into), price)? {
            *grown.paid.entry(key).or_default() += paid;
        }
        let body = p.body(Binding::Actor)?;
        let room = anatomy::room(body, rules, &tissue);
        let fill = room.min(p.held(Binding::Actor, from)?);
        if fill > 0 {
            let taken = convert(
                p,
                rules,
                Binding::Actor,
                (std::slice::from_ref(from), &tissue),
                fill,
                conversion,
            )?;
            grown.taken.push((taken, tissue.clone(), fill));
        }
    }
    Ok(grown)
}
