// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A body in a stage its lineage's grammar switches bring about (D43, D49).
//! It re-reads its switches when it would next grow and, where one flipped,
//! grows toward the unfolding that now holds; the parts that unfolding lacks
//! are taken back, their matter returned to the reserve as far as it has
//! room and the rest to the ground, so none is made or lost.

use crate::{
    Result, anatomy, climate,
    development::{Soma, develop},
    growth,
    rules::{Development, Rules, Situation, switched_on, unfold_on},
    schema::*,
};
use std::{borrow::Cow, collections::BTreeSet};

/// `d` with its tagmata unfolded for the switches `e` holds on, where it
/// holds any; else as it is.
pub fn staged<'a>(d: &'a Development, e: &Entity) -> Cow<'a, Development> {
    let Some(grammar) = d.grammar.as_ref().filter(|_| !e.switched.is_empty()) else {
        return Cow::Borrowed(d);
    };
    let on: Vec<usize> = e.switched.iter().map(|i| usize::from(*i)).collect();
    match unfold_on(grammar, &on) {
        Ok(tagmata) => {
            let mut stage = d.clone();
            stage.recipe.tagmata = tagmata;
            Cow::Owned(stage)
        },
        Err(_) => Cow::Borrowed(d),
    }
}

/// What `e` and its place show its switches at `now`, `crowd` of its lineage
/// sharing the place and `reserve` the account its stores keep.
pub fn situation(rules: &Rules, e: &Entity, now: Tick, crowd: u32, reserve: Option<&Key>) -> Situation {
    let full = |r: &Key| anatomy::held(e, rules, r) > 0 && anatomy::room(e, rules, r) == 0;
    Situation {
        age: now.saturating_sub(e.born),
        reserves_full: reserve.is_some_and(full),
        crowd,
        year: climate::year_phase(rules, now),
    }
}

/// Re-reads `d`'s switches against `shown`, keeping what holds on `e`:
/// whether any flipped.
pub fn reread(d: &Development, e: &mut Entity, shown: &Situation) -> bool {
    let Some(grammar) = &d.grammar else { return false };
    let held = switched_on(grammar, shown);
    let on: Vec<u16> = held.into_iter().filter_map(|i| u16::try_from(i).ok()).collect();
    let flipped = on != e.switched;
    e.switched = on;
    flipped
}

/// Takes off `e` the parts the stage it is in lacks, leaves first, the root
/// kept, and returns the matter they held.
///
/// # Errors
/// Fails if the stage does not develop.
pub fn resorb(rules: &Rules, d: &Development, e: &mut Entity) -> Result<Ledger> {
    let stage = staged(d, e);
    let soma = Soma { segments: growth::target(&stage, e), absent: vec![], seed: 0 };
    let ideal = develop(rules, &stage, &soma)?;
    let wanted: BTreeSet<[u8; 3]> = ideal.doc.parts.iter().filter_map(|g| g.situs).collect();
    let mut matter = Ledger::new();
    loop {
        let root = e.body.as_ref().map(|b| b.root);
        let leaf = e.living().map(|(id, _)| id).find(|id| {
            Some(*id) != root
                && e.situs(*id).is_some_and(|s| !wanted.contains(&s))
                && e.children_of(*id).iter().all(|c| !e.lives(*c))
        });
        let Some((_, part)) = leaf.and_then(|id| e.take_part(id)) else { return Ok(matter) };
        for (key, n) in part.matter {
            *matter.entry(key).or_default() += n;
        }
    }
}
