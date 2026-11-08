// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A rising hazard's death tick drawn once (the execution plan's E7b): the
//! per-tick odds hold through each whole year of age, so survival over a
//! year is one power and the tick within it a search. Survival is integer
//! fixed point, so the draw is the same on every machine.

use super::Aging;
use crate::schema::Tick;

/// Certainty in survival's fixed point; products of two stay in a `u128`.
const ONE: u128 = 1 << 63;
/// The per-tick odds' denominator (`Aging::per_tick`).
const PER: u128 = 1_000_000_000_000;
/// Years past maturity searched before a hazard is taken never to fall.
const YEARS: u32 = 4096;

fn times(a: u128, b: u128) -> u128 {
    (a * b) >> 63
}

fn power(mut base: u128, mut n: u64) -> u128 {
    let mut out = ONE;
    while n > 0 {
        if n & 1 == 1 {
            out = times(out, base);
        }
        base = times(base, base);
        n >>= 1;
    }
    out
}

impl Aging {
    /// The ticks from `age` until the hazard falls, for one uniform draw
    /// `u` (all 64 bits): the first tick by which survival, at the per-tick
    /// odds from `age` on, has fallen to `u` or below. So the tick has the
    /// distribution of a draw made every tick at those odds. None where the
    /// odds never rise above nothing.
    pub fn falls_after(&self, age: Tick, u: u64) -> Option<Tick> {
        let year = self.year.max(1);
        let u = u128::from(u >> 1);
        let (mut alive, mut at) = (ONE, age.max(self.maturity));
        for _ in 0..YEARS {
            let years = (at - self.maturity) / year;
            let end = self.maturity + (years + 1) * year;
            let n = end - at;
            let odds = u128::from(self.per_tick(at).unwrap_or(0)).min(PER);
            if odds == 0 && self.per_million_per_year == 0 {
                return None;
            }
            let each = ONE - odds * ONE / PER;
            let left = times(alive, power(each, n));
            if left > u {
                alive = left;
                at = end;
                continue;
            }
            // The least `k` in 1..=n whose survival is at or below `u`.
            let (mut low, mut high) = (1, n);
            while low < high {
                let mid = low + (high - low) / 2;
                if times(alive, power(each, mid)) <= u {
                    high = mid;
                } else {
                    low = mid + 1;
                }
            }
            return Some(at + low - 1 - age);
        }
        None
    }
}
