// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Every declared Eat has its own addressed pass quota. Whole-prey
//! budgets precede selected-part budgets, including explicit graft takes.

use super::*;
use std::collections::{BTreeSet, VecDeque};

/// The executed declaration and its resolved bite, not merely the next
/// meal encountered. Branch sides are part of the structural address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MealSpec {
    pub(crate) actual_part: bool,
    pub(crate) at: Vec<usize>,
    pub(crate) from: Holder,
    pub(crate) part: Option<Holder>,
    pub(crate) asked: u64,
    pub(crate) into: Key,
    pub(crate) of: Vec<Key>,
    /// Actual food in the selected offering after native port admission.
    /// Kept apart from the authored selector for replay-path matching.
    pub(crate) eligible: Vec<Key>,
    pub(crate) whole: bool,
}

/// An empty portion is still an explicit quota for this exact Eat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MealShare {
    pub(crate) spec: MealSpec,
    pub(crate) portion: Ledger,
}

pub(super) struct Allocated {
    pub(super) meals: Vec<VecDeque<MealShare>>,
    pub(super) bitten: BTreeMap<(Holder, Key), u128>,
    pub(super) graft_parts: BTreeSet<Holder>,
}

/// Retain the original whole-prey rounding for a shared account selector.
/// Different selectors share any overlapping accounts before part caps.
pub(super) fn allocate(
    plans: &[Plan],
    wanted: &BTreeMap<(Holder, Key), u128>,
    ground: &impl Fn(Holder) -> Option<Ledger>,
    rules: &Rules,
) -> Allocated {
    let specs: Vec<_> = plans.iter().flat_map(|p| &p.demands.meals).collect();
    let mut groups: BTreeMap<(Holder, Vec<Key>), Vec<usize>> = BTreeMap::new();
    for (i, spec) in specs.iter().enumerate() {
        let mut of = spec.eligible.clone();
        of.sort();
        of.dedup();
        let source = if spec.actual_part {
            spec.part.unwrap_or(spec.from)
        } else {
            spec.from
        };
        groups.entry((source, of)).or_default().push(i);
    }
    let mut portions = vec![Ledger::new(); specs.len()];
    for ((holder, of), group) in groups {
        // These are explicit eligible keys: an empty set admits nothing,
        // unlike an empty authored `of`, which names every matter account.
        let mut offered = if of.is_empty() {
            Ledger::new()
        } else {
            ground(holder).map_or_else(Ledger::new, |l| edible(&l, &of))
        };
        offered.retain(|k, _| meaning::matter(rules, k));
        let total = offered.values().map(|n| u128::from(*n)).sum::<u128>();
        let total = u64::try_from(total).unwrap_or(u64::MAX);
        let asked = group.iter().map(|i| u128::from(specs[*i].asked)).sum();
        let grants: Vec<_> = group
            .iter()
            .map(|i| fraction(specs[*i].asked, total, asked))
            .collect();
        let mut left = share(&offered, rules, grants.iter().sum());
        for (i, grant) in group.into_iter().zip(grants) {
            let portion = share(&left, rules, grant);
            for (key, n) in &portion {
                *left.get_mut(key).expect("a portion of what is left") -= n;
            }
            portions[i] = portion;
        }
    }
    // Two selectors may name the same source account. Such an overlap
    // cannot reserve that account's pass-start stock more than once.
    let mut by_account: BTreeMap<(Holder, Key), u128> = BTreeMap::new();
    for (spec, portion) in specs.iter().zip(&portions) {
        for (key, n) in portion {
            *by_account.entry((spec.from, key.clone())).or_default() += u128::from(*n);
        }
    }
    let held = |at: &(Holder, Key)| ground(at.0).map_or(0, |l| value(&l, &at.1));
    for (spec, portion) in specs.iter().zip(&mut portions) {
        for (key, n) in portion.iter_mut() {
            let at = (spec.from, key.clone());
            *n = fraction(*n, held(&at), by_account[&at]);
        }
        portion.retain(|_, n| *n > 0);
    }
    let graft_parts: BTreeSet<_> = plans
        .iter()
        .flat_map(|p| p.demands.graft_parts.iter().copied())
        .collect();
    let mut bitten: BTreeMap<(Holder, Key), u128> = BTreeMap::new();
    for (spec, portion) in specs.iter().zip(&portions) {
        if let Some(part) = spec.part {
            for (key, n) in portion {
                *bitten.entry((part, key.clone())).or_default() += u128::from(*n);
            }
        }
    }
    for (spec, portion) in specs.iter().zip(&mut portions) {
        if let Some(part) = spec.part {
            for (key, n) in portion.iter_mut() {
                let at = (part, key.clone());
                let carrying = if graft_parts.contains(&part) {
                    wanted.get(&at).copied().unwrap_or(0)
                } else {
                    0
                };
                *n = fraction(*n, held(&at), bitten[&at] + carrying);
            }
            portion.retain(|_, n| *n > 0);
        }
    }
    let mut portions = portions.into_iter();
    let meals = plans
        .iter()
        .map(|p| {
            p.demands
                .meals
                .iter()
                .map(|spec| MealShare {
                    spec: spec.clone(),
                    portion: portions.next().expect("a quota for every addressed meal"),
                })
                .collect()
        })
        .collect();
    Allocated {
        meals,
        bitten,
        graft_parts,
    }
}
