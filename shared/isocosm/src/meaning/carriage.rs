// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a process's systems carry (rulings 562, 566, 575, 581, 582, 588 and
//! 589), written once for both runners: what each effect part asks, the
//! total a reading reads, and the bounds a carriage sets on what the act
//! then gives a body's parts. A landing's parts ask by their room for what
//! it lands, the parts a body still grows asking with the part they grow
//! on, and never more in all than that room; a carriage that lands nothing
//! asks by cells. Where every living part is an effect and every ask is
//! carried, nothing is bounded and the act lands as it did before systems.

use super::Parties;
use crate::{
    Result,
    anatomy::{self, Caps},
    growth,
    rules::{Binding, Development, Role, Rules},
    schema::*,
    systems,
};
use std::collections::{BTreeMap, BTreeSet};

/// What each effect part of the systems `e` carries naming `function` in
/// `role` asks of `ask`, and those effect parts.
pub(crate) fn asks(
    e: &Entity,
    rules: &Rules,
    d: Option<&Development>,
    (function, role): (&str, Role),
    (ask, lands): (u64, &[Key]),
    bitten: Option<Id>,
) -> (BTreeMap<Id, u64>, BTreeSet<Id>) {
    let Some(system) = systems::union(e, function, role) else {
        return Default::default();
    };
    let effects = systems::filling(e, &system.effects, bitten);
    let cells = |p: &Part| p.cells.values().map(|n| u64::from(*n)).sum::<u64>();
    let room = |p: &Part| {
        lands
            .iter()
            .map(|k| anatomy::space(p, rules, k))
            .fold(0, u64::saturating_add)
    };
    let mut weights: Vec<(Id, u64)> = effects
        .iter()
        .map(|id| {
            let p = &e.parts[id];
            (*id, if lands.is_empty() { cells(p) } else { room(p) })
        })
        .collect();
    if !lands.is_empty()
        && let Some(d) = d
        && let Ok(Some(next)) = growth::lacking(rules, d, e)
        && let Some(w) = weights.iter_mut().find(|(id, _)| Some(*id) == next.parent)
    {
        w.1 = w.1.saturating_add(growth::lacking_mass(rules, d, e));
    }
    let total = match lands.is_empty() {
        true => ask,
        false => ask.min(weights.iter().map(|(_, w)| *w).fold(0, u64::saturating_add)),
    };
    let asked = anatomy::apportion(&weights, total).into_iter().collect();
    (asked, effects)
}

/// What the systems `e` carries naming `function` in `role` carry of
/// `ask`: the total an amount reads (`Reading::Carried`). Where it `joined`
/// another route, the ask is what the senses join of it (659 to 664).
pub(crate) fn carried(
    e: &Entity,
    rules: &Rules,
    d: Option<&Development>,
    route: (&str, Role),
    (ask, lands): (u64, &[Key]),
    bitten: Option<Id>,
    joined: Option<(&str, Role)>,
) -> Result<u64> {
    let ask = match joined {
        Some(first) => {
            let (reached, total) = systems::joined(e, rules, first, route, bitten);
            let share = u128::from(ask) * u128::from(reached) / u128::from(total.max(1));
            u64::try_from(share).unwrap_or(u64::MAX)
        },
        None => ask,
    };
    let (asked, _) = asks(e, rules, d, route, (ask, lands), bitten);
    Ok(systems::carry(e, rules, route, &asked, bitten)?.total)
}

/// `Effect::Carry`: carries `ask` to `who`'s effect parts and, where a part
/// is not an effect or was carried less than it asked, bounds what the act
/// gives each living part by what reached it.
pub(crate) fn carry(
    p: &mut impl Parties,
    rules: &Rules,
    who: Binding,
    route: (&str, Role),
    landing: (u64, &[Key]),
) -> Result<()> {
    let bitten = p.bitten();
    let lineage = p.body(who)?.lineage.clone();
    let d = p.development(&lineage).ok();
    let body = p.body(who)?;
    let (asked, effects) = asks(body, rules, d.as_ref(), route, landing, bitten);
    let carried = systems::carry(body, rules, route, &asked, bitten)?;
    let living: BTreeSet<Id> = body
        .parts
        .iter()
        .filter(|(_, q)| !q.severed)
        .map(|(id, _)| *id)
        .collect();
    let short = asked
        .iter()
        .any(|(id, n)| carried.parts.get(id).copied().unwrap_or(0) < *n);
    if short || effects != living {
        let caps: Caps = living
            .iter()
            .map(|id| (*id, carried.parts.get(id).copied().unwrap_or(0)))
            .collect();
        p.bound(who, caps)?;
    }
    Ok(())
}
