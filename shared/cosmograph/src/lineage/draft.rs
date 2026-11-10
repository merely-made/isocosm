// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The player's draft (cases 1200, 1203; lineage-rounds D10 to D12): what a
//! line must eat whole before it learns a kind, which kinds the played
//! line's review offers, and the mass a variant may add. All of it sits
//! behind the world's `draft` rule; unset, a kind is learned at the first
//! meal and nothing is budgeted.

use super::tree;
use crate::{
    Result, Session,
    development::{Soma, develop},
    geometry::Body,
    rules::{Development, Rules},
    schema::*,
};
use std::collections::{BTreeMap, BTreeSet};

/// How many whole meals of `kind` the eater's line needs: one, plus one for
/// another kingdom, plus none, one or two as the donor is kin, a distant
/// relation or unrelated, less one if the kind carries only functions the
/// eater's recipe already has the kinds for; never below one (case 1203;
/// lineage-rounds D10). A missing line or kind counts as unfamiliar,
/// unrelated and of another kingdom.
pub fn needed(
    lines: &BTreeMap<Key, Lineage>,
    rules: &Rules,
    eater: &str,
    donor: &str,
    kind: &str,
) -> u8 {
    let (mine, theirs) = (lines.get(eater), lines.get(donor));
    let kingdom = match (mine, theirs) {
        (Some(a), Some(b)) if a.kingdom == b.kingdom => 0,
        _ => 1,
    };
    let kin: u8 = if eater == donor {
        0
    } else {
        match tree::distance(lines, eater, donor) {
            Some(d) if d <= 1 => 0,
            Some(_) => 1,
            None => 2,
        }
    };
    let functions = |k: &str| rules.kinds.get(k).map(|t| t.cells.keys().collect::<BTreeSet<_>>());
    let known: BTreeSet<&Key> = mine
        .and_then(|l| l.development.as_ref())
        .map(|d| d.recipe.kinds())
        .unwrap_or_default()
        .iter()
        .filter_map(|k| functions(k))
        .flatten()
        .collect();
    let familiar = functions(kind).is_some_and(|f| !f.is_empty() && f.is_subset(&known));
    (1 + kingdom + kin - u8::from(familiar)).max(1)
}

/// Teaches `eater`'s line `kind`, eaten whole of `donor`'s at `tick`. Unset,
/// the first meal teaches it; set, the line must have eaten it `needed`
/// times (case 1203; lineage-rounds D10).
pub fn learn(
    lines: &mut BTreeMap<Key, Lineage>,
    rules: &Rules,
    eater: &str,
    donor: &str,
    kind: &Key,
    tick: Tick,
) {
    let counted = rules.directing.as_ref().is_some_and(|d| d.draft.is_some());
    let asked = counted.then(|| needed(lines, rules, eater, donor, kind));
    let Some(line) = lines.get_mut(eater) else {
        return;
    };
    if eater != donor && rules.directing.as_ref().is_some_and(|d| d.regard.is_some()) {
        line.ate.insert(donor.to_string());
    }
    let Some(asked) = asked else {
        if let Some(d) = line.development.as_mut() {
            d.lexicon.insert(kind.clone());
        }
        return;
    };
    let Some(d) = line.development.as_mut() else {
        return;
    };
    let meal = line.meals.entry(kind.clone()).or_insert(Eaten {
        count: 0,
        needed: asked,
        last: tick,
    });
    meal.count = meal.count.saturating_add(1);
    meal.needed = meal.needed.min(asked);
    meal.last = tick;
    if meal.count >= u32::from(meal.needed) {
        d.lexicon.insert(kind.clone());
    }
}

/// The kinds the line ate whole at or after `since`.
pub fn eaten_since(lineage: &Lineage, since: Tick) -> BTreeSet<Key> {
    lineage
        .meals
        .iter()
        .filter(|(_, m)| m.last >= since)
        .map(|(k, _)| k.clone())
        .collect()
}

/// The fully grown body `d` develops, each tagma at its segments.
///
/// # Errors
/// Fails if `d` cannot develop a body.
pub(super) fn grown_body(rules: &Rules, d: &Development) -> Result<Body> {
    let soma = Soma {
        segments: d.recipe.tagmata.iter().map(|t| t.segments).collect(),
        absent: vec![],
        seed: 0,
    };
    develop(rules, d, &soma)
}

/// The adult mass of the body `d` grows into.
fn grown(rules: &Rules, d: &Development) -> Result<u128> {
    let b = rules.body();
    Ok(grown_body(rules, d)?
        .doc
        .parts
        .iter()
        .map(|p| u128::from(crate::anatomy::ceiling(p.half_extent, b)))
        .sum())
}

/// The adult mass `moved` adds to a fully grown body over `had`, nought if
/// it adds none (case 1203; lineage-rounds D11).
///
/// # Errors
/// Fails if either development cannot develop a body.
pub fn cost(rules: &Rules, had: &Development, moved: &Development) -> Result<u128> {
    Ok(grown(rules, moved)?.saturating_sub(grown(rules, had)?))
}

/// What the played line may spend on a variant: the rule's share of what
/// its average living member holds; none unless the rule is set (case 1203;
/// lineage-rounds D11).
pub fn budget(session: &Session, lineage: &str) -> Option<u128> {
    let draft = session.sim.genesis().rules.directing().draft?;
    let (held, members) = super::boundary::standing(session, lineage);
    if members == 0 {
        return Some(0);
    }
    let [top, bottom] = draft.share.map(u128::from);
    Some(held * top / (u128::from(members) * bottom))
}
