// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Integer-only draws, identical on every platform. A stream is seeded once
//! from `crate::draw`, so a round's draws follow from the dynamics seed.

use std::collections::BTreeMap;

/// SplitMix64: wrapping integer arithmetic only.
pub(super) struct Stream(u64);

impl Stream {
    pub(super) fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform below `n`, by rejection so no value is favoured.
    pub(super) fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "empty range");
        let zone = u64::MAX - u64::MAX % n;
        loop {
            let x = self.next();
            if x < zone {
                return x % n;
            }
        }
    }
    pub(super) fn coin(&mut self) -> bool {
        self.next() >> 63 == 1
    }
    pub(super) fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i as u64 + 1) as usize;
            items.swap(i, j);
        }
    }
    /// Successes in `draws` taken without replacement from `total` items of
    /// which `successes` succeed.
    pub(super) fn hypergeometric(&mut self, successes: u64, total: u64, draws: u64) -> u64 {
        let (mut good, mut left, mut hits) = (successes, total, 0);
        for _ in 0..draws {
            if self.below(left) < good {
                good -= 1;
                hits += 1;
            }
            left -= 1;
        }
        hits
    }
    /// How `draws` members taken at random split across categories.
    pub(super) fn split(&mut self, counts: &[u64], draws: u64) -> Vec<u64> {
        let mut left: u64 = counts.iter().sum();
        let mut wanted = draws;
        let mut taken = Vec::with_capacity(counts.len());
        for &c in counts {
            let t = if wanted == 0 {
                0
            } else if c == left {
                wanted
            } else {
                self.hypergeometric(c, left, wanted)
            };
            taken.push(t);
            wanted -= t;
            left -= c;
        }
        taken
    }
    /// Pair counts of a uniform random perfect matching over members counted
    /// by category: match any unmatched member to a uniformly random other.
    pub(super) fn matching(&mut self, counts: &[u64]) -> BTreeMap<(usize, usize), u64> {
        let mut left = counts.to_vec();
        let mut total: u64 = left.iter().sum();
        assert!(
            total.is_multiple_of(2),
            "a perfect matching needs an even count"
        );
        let mut pairs = BTreeMap::new();
        while total > 0 {
            let a = left.iter().position(|&c| c > 0).unwrap();
            left[a] -= 1;
            let mut pick = self.below(total - 1);
            let b = left
                .iter()
                .position(|&c| {
                    if pick < c {
                        true
                    } else {
                        pick -= c;
                        false
                    }
                })
                .unwrap();
            left[b] -= 1;
            total -= 2;
            *pairs.entry((a.min(b), a.max(b))).or_default() += 1;
        }
        pairs
    }
}
