// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::schema::Tick;
use std::collections::BTreeMap;

pub(crate) type Counts = BTreeMap<Tick, u64>;

pub(crate) fn record(counts: &mut Option<Counts>, tick: Tick) {
    *counts.get_or_insert_with(Counts::new).entry(tick).or_default() += 1;
}

pub(crate) fn since(counts: Option<&Counts>, tick: Tick) -> u64 {
    counts.into_iter().flat_map(|counts| counts.range(tick..).map(|(_, n)| n)).sum()
}

pub(crate) fn merge(into: &mut Option<Counts>, from: Option<&Counts>) {
    if let Some(from) = from {
        let into = into.get_or_insert_with(Counts::new);
        for (&tick, &n) in from {
            *into.entry(tick).or_default() += n;
        }
    }
}

/// Cohort counts carry no individual attribution. Divide them in proportion
/// to the two remaining intervals and leave rounding residue with the lifted
/// member, as with the population pool.
pub(crate) fn split(
    counts: Option<&Counts>,
    before: u64,
    after: u64,
    total: u64,
) -> [Option<Counts>; 3] {
    let mut pieces = [None, None, None];
    let Some(counts) = counts else { return pieces; };
    let share = |n, size| (u128::from(n) * u128::from(size) / u128::from(total)) as u64;
    for (&tick, &n) in counts {
        let lo = share(n, before);
        let hi = share(n, after);
        for (piece, count) in pieces.iter_mut().zip([lo, n - lo - hi, hi]) {
            if count > 0 {
                piece.get_or_insert_with(Counts::new).insert(tick, count);
            }
        }
    }
    pieces
}
