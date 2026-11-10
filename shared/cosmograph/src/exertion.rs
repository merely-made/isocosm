// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exertion (case 1234): each accepted Choice act counts once in the span
//! it falls in, and an act past its body's ease spends matter from the
//! accounts its lineage's upkeep spends from, into the soil that upkeep goes
//! to. The spend is staged with the act as any effect of it, so it is
//! conserved, flowed and replayed with it. A body that cannot pay pays what
//! it has; nothing is refused. Unset, nothing here runs.
//!
//! The independent crowd (`probe::aggregate`) keeps no count per member and
//! leaves the rule inactive.

use crate::{
    climate::{paying, upkeeps},
    rules::{Binding, Query, Rules, ease},
    schema::{Entity, Key},
};

/// The function key of a cell that contracts.
const CONTRACT: &str = "function:contract";

/// What `e`'s lineage's upkeep pays from and into: the accounts it takes
/// and where they go, found as the climate's strain finds them.
pub(crate) fn upkeep_for(rules: &Rules, e: &Entity) -> Option<(Vec<Key>, Key)> {
    upkeeps(rules).into_values().find_map(|(id, _)| {
        let upkeep = &rules.processes[id];
        let mine = upkeep.requires.iter().all(|q| match q {
            Query::Trait { who: Binding::Actor, key } => e.traits.contains(key),
            _ => true,
        });
        mine.then(|| upkeep.effects.iter().find_map(paying)).flatten()
    })
}

/// The acts a span `e` makes at ease: its share of contracting cells among
/// the cells of its living parts (case 1234).
pub(crate) fn easy(rules: &Rules, e: &Entity) -> u32 {
    let Some(exertion) = &rules.exertion else { return 0 };
    let (mut contract, mut cells) = (0u64, 0u64);
    for (_, part) in e.living() {
        contract += u64::from(part.cells.get(CONTRACT).copied().unwrap_or(0));
        cells += part.cells.values().map(|n| u64::from(*n)).sum::<u64>();
    }
    ease(exertion, contract, cells)
}
