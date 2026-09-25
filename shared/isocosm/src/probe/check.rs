// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 113's check between two ways of running the same drawn worlds,
//! reading by reading (bench statistics, not the sim, so floats are fine).
//! Distance is the two-sample Kolmogorov-Smirnov distance over the draws.
//! The difference test permutes arm labels within each drawn world, which is
//! exact for paired draws and for tied, discrete readings. The equivalence
//! test certifies distance below the reading's bound by the DKW-Massart
//! inequality applied to each arm. Both families are Holm-corrected.

use super::draws::Stream;
use crate::schema::Key;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ReadingResult {
    pub key: Key,
    pub bound: f64,
    pub distance: f64,
    pub mean_a: f64,
    pub mean_b: f64,
    pub p_difference: f64,
    pub p_difference_holm: f64,
    pub detected: bool,
    pub p_equivalence: f64,
    pub p_equivalence_holm: f64,
    pub certified: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Settings {
    pub alpha: f64,
    pub permutations: u64,
    pub seed: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    pub name: String,
    pub draws: usize,
    pub settings: Settings,
    pub readings: Vec<ReadingResult>,
    /// Every reading certified within its bound.
    pub equivalent: bool,
    /// Some reading's difference detected beyond chance.
    pub different: bool,
    pub pass: bool,
}

/// Value indices into the pooled, sorted distinct values.
fn indices(a: &[u64], b: &[u64]) -> (Vec<usize>, Vec<usize>, usize) {
    let mut values: Vec<u64> = a.iter().chain(b).copied().collect();
    values.sort_unstable();
    values.dedup();
    let at = |x: &u64| values.binary_search(x).expect("value present");
    (
        a.iter().map(at).collect(),
        b.iter().map(at).collect(),
        values.len(),
    )
}

/// The largest gap between the two empirical distributions, in draws; each
/// pair adds its sign at its A value and removes it at its B value.
fn gap(ia: &[usize], ib: &[usize], width: usize, signs: impl Iterator<Item = i64>) -> u64 {
    let mut diff = vec![0i64; width];
    for ((&x, &y), s) in ia.iter().zip(ib).zip(signs) {
        diff[x] += s;
        diff[y] -= s;
    }
    let mut run = 0i64;
    let mut max = 0;
    for d in diff {
        run += d;
        max = max.max(run.unsigned_abs());
    }
    max
}

pub fn distance(a: &[u64], b: &[u64]) -> f64 {
    let (ia, ib, width) = indices(a, b);
    gap(&ia, &ib, width, std::iter::repeat(1)) as f64 / a.len().max(1) as f64
}

/// P-value of the observed gap under label swaps within pairs.
fn permutation(a: &[u64], b: &[u64], permutations: u64, seed: u64) -> f64 {
    let (ia, ib, width) = indices(a, b);
    let observed = gap(&ia, &ib, width, std::iter::repeat(1));
    let mut stream = Stream::new(seed);
    let mut extreme = 0u64;
    for _ in 0..permutations {
        let signs = (0..a.len()).map(|_| if stream.coin() { 1 } else { -1 });
        if gap(&ia, &ib, width, signs) >= observed {
            extreme += 1;
        }
    }
    (1 + extreme) as f64 / (1 + permutations) as f64
}

/// Each arm's empirical distribution lies within t of its true one except
/// with probability 2 exp(-2 n t^2); both do with the remainder, and then
/// the true distance is below the observed distance plus 2t.
pub fn equivalence(observed: f64, bound: f64, draws: usize) -> f64 {
    if observed >= bound {
        return 1.0;
    }
    let t = (bound - observed) / 2.0;
    (4.0 * (-2.0 * draws as f64 * t * t).exp()).min(1.0)
}

pub fn holm(p: &[f64]) -> Vec<f64> {
    let m = p.len();
    let mut order: Vec<usize> = (0..m).collect();
    order.sort_by(|&i, &j| p[i].total_cmp(&p[j]));
    let mut adjusted = vec![0.0; m];
    let mut running = 0.0f64;
    for (rank, &i) in order.iter().enumerate() {
        running = running.max(((m - rank) as f64 * p[i]).min(1.0));
        adjusted[i] = running;
    }
    adjusted
}

/// `a[k][r]` and `b[k][r]`: reading `r` of the world drawn `k`, each arm;
/// `readings` pairs each reading's key with its bound, per mille.
pub fn compare(
    name: &str,
    readings: &[(Key, u32)],
    a: &[Vec<u64>],
    b: &[Vec<u64>],
    settings: Settings,
) -> Comparison {
    let draws = a.len();
    let column = |arm: &[Vec<u64>], r: usize| arm.iter().map(|row| row[r]).collect::<Vec<u64>>();
    let mean = |v: &[u64]| v.iter().sum::<u64>() as f64 / v.len().max(1) as f64;
    let mut rows = Vec::new();
    for (r, (key, per_mille)) in readings.iter().enumerate() {
        let (x, y) = (column(a, r), column(b, r));
        let d = distance(&x, &y);
        let bound = f64::from(*per_mille) / 1000.0;
        let seed = crate::draw(settings.seed, "probe-permutation", &[r as u64]);
        rows.push(ReadingResult {
            key: key.clone(),
            bound,
            distance: d,
            mean_a: mean(&x),
            mean_b: mean(&y),
            p_difference: permutation(&x, &y, settings.permutations, seed),
            p_difference_holm: 0.0,
            detected: false,
            p_equivalence: equivalence(d, bound, draws),
            p_equivalence_holm: 0.0,
            certified: false,
        });
    }
    let difference = holm(&rows.iter().map(|r| r.p_difference).collect::<Vec<_>>());
    let equivalent = holm(&rows.iter().map(|r| r.p_equivalence).collect::<Vec<_>>());
    for (row, (pd, pe)) in rows.iter_mut().zip(difference.into_iter().zip(equivalent)) {
        row.p_difference_holm = pd;
        row.detected = pd <= settings.alpha;
        row.p_equivalence_holm = pe;
        row.certified = pe <= settings.alpha;
    }
    let equivalent = rows.iter().all(|r| r.certified);
    let different = rows.iter().any(|r| r.detected);
    Comparison {
        name: name.into(),
        draws,
        settings,
        readings: rows,
        equivalent,
        different,
        pass: equivalent && !different,
    }
}
