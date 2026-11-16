// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Growth with the parts full grows the parts a body lacks (rulings 479
//! and 510): one at a time, nearest the root first, each paying PD2's price
//! (weighed by its kingdom, case 1232) from the reserve and then the tissue into the ground, and then filled by
//! what the act holds in hand, given to the parts by their room (464).

use super::{Parties, convert, spend};
use crate::{Result, anatomy, growth, pooling, rules::*, schema::*, switching};

/// What growth moved: what each account paid toward the prices, and what
/// of the hand each conversion took.
#[derive(Default)]
pub(crate) struct Grown {
    pub paid: Ledger,
    pub taken: Vec<(Ledger, Key, u64)>,
    /// What parts taken back (D49) gave the ground, by the account it left.
    pub spilled: Vec<(Key, u64)>,
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
    let kingdom = p.kingdom(&lineage);
    let anatomy::Own {
        tissue, reserve, ..
    } = anatomy::own(rules, &lineage);
    let Some(tissue) = tissue else {
        return Err(format!("{lineage} has no tissue to grow"));
    };
    if development.grammar.is_some() {
        // The switches are read when the body would next grow (D49); parts
        // the stage now lacks go to the reserve as far as it has room, the
        // rest to the ground, as rent does.
        let (now, crowd) = (p.tick(), p.crowd(&lineage));
        let body = p.body(Binding::Actor)?;
        let shown = switching::situation(rules, body, now, crowd, reserve.as_ref());
        if switching::reread(&development, body, &shown) {
            for (key, n) in switching::resorb(rules, &development, body)? {
                let room = match &reserve {
                    Some(r) => p.room(Binding::Actor, r)?,
                    None => 0,
                };
                let put = n.min(room);
                if let (Some(r), true) = (&reserve, put > 0) {
                    p.give(Binding::Actor, r, put)?;
                }
                if n > put {
                    p.give(Binding::Place, into, n - put)?;
                    grown.spilled.push((key, n - put));
                }
            }
        }
    }
    if let (true, Some(r)) = (development.grammar.is_some(), &reserve) {
        // Fused runs hold their reserve as one (D50).
        let body = p.body(Binding::Actor)?;
        let stage = switching::staged(&development, body).into_owned();
        pooling::share(rules, &stage, body, r);
    }
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
        let Some((frame, part)) = growth::lacking(rules, &development, body)? else {
            break;
        };
        let price = growth::price(rules, &kingdom, &frame, &part);
        let funds: u64 = pay
            .iter()
            .map(|k| anatomy::held(body, rules, k))
            .fold(0, u64::saturating_add);
        if funds < price {
            break;
        }
        body.add_part(&frame, part)?;
        for (key, paid) in spend(p, &pay, Binding::Place, Some(into), price)? {
            *grown.paid.entry(key).or_default() += paid;
        }
        let room = p.room(Binding::Actor, &tissue)?;
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
