// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Integer-only draws, identical on every platform. A stream is seeded once
//! from `crate::draw`, so a round's draws follow from the dynamics seed.
//! The exact count draws walk member by member; the near ones (ruling 220's
//! approximate pairing draw) take each count in one step from a normal
//! draw matching its mean and variance, so their cost does not grow with
//! the members counted.

use std::collections::BTreeMap;

/// Fixed-point scale of the near draws: 16 fractional bits.
const ONE: i128 = 1 << 16;

/// How a crowd draws its counts: exactly, member by member, or near.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Counts {
    Exact,
    Near,
}

/// The integer square root, rounded down.
fn isqrt(n: u128) -> u128 {
    if n < 2 {
        return n;
    }
    let mut x = 1u128 << (n.ilog2() / 2 + 1);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

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
    pub(super) fn split_by(&mut self, how: Counts, counts: &[u64], draws: u64) -> Vec<u64> {
        match how {
            Counts::Exact => self.split(counts, draws),
            Counts::Near => self.split_near(counts, draws),
        }
    }
    pub(super) fn matching_by(
        &mut self,
        how: Counts,
        counts: &[u64],
    ) -> BTreeMap<(usize, usize), u64> {
        match how {
            Counts::Exact => self.matching(counts),
            Counts::Near => self.matching_near(counts),
        }
    }
    /// A standard normal draw in fixed point, approximated by the sum of
    /// twelve uniforms less six (Irwin-Hall), exact in integers.
    fn normal(&mut self) -> i128 {
        let sum: i128 = (0..12).map(|_| i128::from(self.below(1 << 16))).sum();
        sum - 6 * ONE
    }
    /// An integer near a count with the given mean and variance, both in
    /// fixed point, rounded and kept within `[lo, hi]`.
    fn near(&mut self, mean: i128, variance: i128, lo: u64, hi: u64) -> u64 {
        let sd = isqrt((variance.max(0) as u128) * (ONE as u128)) as i128;
        let x = mean + sd * self.normal() / ONE;
        let rounded = (x + ONE / 2).div_euclid(ONE);
        rounded.clamp(i128::from(lo), i128::from(hi)) as u64
    }
    /// How `draws` members taken at random split across categories, each
    /// count taken near its hypergeometric mean and variance in turn.
    pub(super) fn split_near(&mut self, counts: &[u64], draws: u64) -> Vec<u64> {
        let mut left: u64 = counts.iter().sum();
        let mut wanted = draws;
        let mut taken = Vec::with_capacity(counts.len());
        for &c in counts {
            let t = if wanted == 0 || c == 0 {
                0
            } else if c == left {
                wanted
            } else {
                let (c, l, d) = (i128::from(c), i128::from(left), i128::from(wanted));
                let mean = d * c * ONE / l;
                let variance = d * c * (l - c) * (l - d) * ONE / (l * l * (l - 1));
                let lo = wanted.saturating_sub(left - c as u64);
                self.near(mean, variance, lo, (c as u64).min(wanted))
            };
            taken.push(t);
            wanted -= t;
            left -= c;
        }
        taken
    }
    /// Pair counts of a random perfect matching, category by category: the
    /// pairs a category keeps to itself near the mean and variance a uniform
    /// matching gives them, and its other members' partners split near the
    /// hypergeometric among the categories after it.
    pub(super) fn matching_near(&mut self, counts: &[u64]) -> BTreeMap<(usize, usize), u64> {
        let mut left = counts.to_vec();
        let mut pairs = BTreeMap::new();
        for i in 0..left.len() {
            let r = left[i];
            if r == 0 {
                continue;
            }
            let others: u64 = left[i + 1..].iter().sum();
            let total = i128::from(r + others);
            let lo = r.saturating_sub(others).div_ceil(2);
            let within = if others == 0 || total <= 3 {
                lo
            } else {
                // Each of the r(r-1)/2 pairs inside the category is matched
                // with chance 1/(R-1), two disjoint ones together with
                // chance 1/((R-1)(R-3)); overlapping ones never are.
                let r = i128::from(r);
                let n2 = r * (r - 1) / 2;
                let disjoint = n2 * (r - 2).max(0) * (r - 3).max(0) / 2;
                let mean = n2 * ONE / (total - 1);
                let square = mean + disjoint * ONE / ((total - 1) * (total - 3));
                let variance = square - mean * mean / ONE;
                self.near(mean, variance, lo, r as u64 / 2)
            };
            left[i] = 0;
            if within > 0 {
                pairs.insert((i, i), within);
            }
            let across = self.split_near(&left[i + 1..], r - 2 * within);
            for (k, t) in across.into_iter().enumerate() {
                if t > 0 {
                    pairs.insert((i, i + 1 + k), t);
                    left[i + 1 + k] -= t;
                }
            }
        }
        pairs
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
