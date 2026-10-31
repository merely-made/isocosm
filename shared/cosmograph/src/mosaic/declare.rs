// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A declared tract laid on a developing part (765).

use crate::schema::*;

/// Gives `p`, a part of geometry `f`, a declared tract (765) where its
/// shape is the one declared: the declared cells of the function, or as
/// many as the part holds, taken from its free cells first and then from
/// its fullest functions. A declared name stands for the box where given.
pub fn declare(p: &mut Part, f: &crate::geometry::Frame, d: &crate::rules::Declared) {
    let shape = match f.shape.is_empty() {
        true => crate::anatomy::boxed(f.half_extent),
        false => f.shape.as_str(),
    };
    if shape != d.shape {
        return;
    }
    let capacity = crate::anatomy::capacity(f.half_extent);
    let want = d.cells.min(capacity);
    let used: u32 = p.cells.values().sum();
    let mut have = p.cells.get(&d.function).copied().unwrap_or(0);
    let free = capacity.saturating_sub(used);
    let from_free = free.min(want.saturating_sub(have));
    have += from_free;
    while have < want {
        let fullest = p
            .cells
            .iter()
            .filter(|(k, n)| **k != d.function && **n > 0)
            .max_by_key(|(_, n)| **n)
            .map(|(k, _)| k.clone());
        let Some(k) = fullest else { break };
        let n = p.cells.get_mut(&k).expect("found above");
        *n -= 1;
        if *n == 0 {
            p.cells.remove(&k);
            p.functions.remove(&k);
        }
        have += 1;
    }
    if have > 0 {
        p.cells.insert(d.function.clone(), have);
        p.functions.insert(d.function.clone());
    }
}
