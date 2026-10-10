// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, rules::*, schema::Part};
use std::collections::BTreeSet;

/// Recorded allocations name actual lost cells and catalogue functions.
/// Unrecorded old losses remain unknown; no allocation is inferred.
pub(super) fn losses(rules: &Rules, part: &Part, half: [i32; 3]) -> Result<()> {
    let lost: BTreeSet<_> = part.lost.iter().copied().collect();
    let capacity = crate::anatomy::capacity(half);
    if lost.len() != part.lost.len() || lost.iter().any(|c| u32::from(c.0) >= capacity) {
        return Err("invalid lost cell identities".into());
    }
    for (cell, allocation) in &part.losses {
        if !lost.contains(cell) {
            return Err("a loss allocation names a present cell".into());
        }
        if let Some(f) = allocation
            && !rules.functions.contains_key(f)
        {
            return Err(format!("a loss allocation names unknown function {f}"));
        }
    }
    Ok(())
}

/// A body's changing cell count bounds a wound's part draw. Its slot
/// cannot also stand for a fixed expression draw or another wound.
pub(super) fn draws(p: &Process) -> Result<()> {
    fn slots(x: &Expr, out: &mut BTreeSet<u8>) {
        if let Expr::Draw { slot, .. } = x {
            out.insert(*slot);
        }
        for child in x.children() {
            slots(child, out);
        }
    }
    let effects = p
        .commitments
        .iter()
        .chain(&p.effects)
        .chain(p.risk.iter().flat_map(|r| &r.effects));
    let mut fixed = BTreeSet::new();
    let mut wounds = BTreeSet::new();
    for e in effects.flat_map(|e| std::iter::once(e).chain(e.branches())) {
        if let Some(x) = e.computed() {
            slots(x, &mut fixed);
        }
        for a in e.amounts() {
            if let Amount::Computed(x) = a {
                slots(x, &mut fixed);
            }
        }
        if let Effect::Wound { slot, .. } = e
            && !wounds.insert(*slot)
        {
            return Err(format!("{} repeats a wound's part draw slot", p.id));
        }
    }
    if !fixed.is_disjoint(&wounds) {
        return Err(format!("{} reuses a wound's part draw slot", p.id));
    }
    Ok(())
}
