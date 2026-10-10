// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, rules::*};
use std::collections::BTreeSet;

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
