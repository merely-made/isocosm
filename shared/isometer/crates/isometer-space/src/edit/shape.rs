// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The cells of a shape, column by column: the half-open runs `[y0, y1)` a
//! vertical line through a column's centre spends inside it. Every shape is
//! a union of convex pieces, so each piece gives one run and a route's runs
//! are merged. Doubled coordinates keep cell centres integral.

use super::Shape;

/// The runs of column `(x, z)` inside `shape`, sorted and disjoint.
pub(crate) fn runs(shape: &Shape, x: i64, z: i64) -> Vec<[i64; 2]> {
    match shape {
        Shape::Sphere { centre, radius } => sphere(*centre, *radius, x, z).into_iter().collect(),
        Shape::Box { min, max } => {
            let inside = min[0] <= x && x < max[0] && min[2] <= z && z < max[2];
            (inside && min[1] < max[1])
                .then_some([min[1], max[1]])
                .into_iter()
                .collect()
        },
        Shape::Route { points, radius } => {
            let mut runs: Vec<[i64; 2]> = match points.as_slice() {
                [] => vec![],
                [p] => sphere(*p, *radius, x, z).into_iter().collect(),
                _ => points
                    .windows(2)
                    .filter_map(|w| capsule(w[0], w[1], *radius, x, z))
                    .collect(),
            };
            runs.sort_unstable();
            let mut merged: Vec<[i64; 2]> = Vec::with_capacity(runs.len());
            for run in runs {
                match merged.last_mut() {
                    Some(last) if run[0] <= last[1] => last[1] = last[1].max(run[1]),
                    _ => merged.push(run),
                }
            }
            merged
        },
    }
}

fn sphere(c: [i64; 3], r: u64, x: i64, z: i64) -> Option<[i64; 2]> {
    let (dx, dz) = (i128::from(2 * x + 1 - 2 * c[0]), i128::from(2 * z + 1 - 2 * c[2]));
    let r2 = 2 * i128::from(r);
    let k = r2 * r2 - dx * dx - dz * dz;
    if k < 0 {
        return None;
    }
    let m = isqrt(k as u128) as i64;
    let y0 = (2 * c[1] - m).div_euclid(2);
    let y1 = (2 * c[1] + m - 1).div_euclid(2) + 1;
    (y0 < y1).then_some([y0, y1])
}

/// The run inside the capsule from `a` to `b`. Its squared distance along
/// the column is convex in `y`, so the nearest cell is found by bisecting
/// the forward difference and each end by bisecting the bound.
fn capsule(a: [i64; 3], b: [i64; 3], r: u64, x: i64, z: i64) -> Option<[i64; 2]> {
    let d = [0, 1, 2].map(|k| i128::from(2 * (b[k] - a[k])));
    let den: i128 = d.iter().map(|v| v * v).sum();
    if den == 0 {
        return sphere(a, r, x, z);
    }
    let r2 = 2 * i128::from(r);
    let limit = r2 * r2 * den;
    let g = |y: i64| -> i128 {
        let p = [2 * x + 1, 2 * y + 1, 2 * z + 1].map(i128::from);
        let w = [0, 1, 2].map(|k| p[k] - 2 * i128::from(a[k]));
        let tn: i128 = (0..3).map(|k| w[k] * d[k]).sum();
        let ww: i128 = w.iter().map(|v| v * v).sum();
        if tn <= 0 {
            ww * den
        } else if tn >= den {
            let v = [0, 1, 2].map(|k| p[k] - 2 * i128::from(b[k]));
            v.iter().map(|v| v * v).sum::<i128>() * den
        } else {
            ww * den - tn * tn
        }
    };
    let r = r as i64;
    let (lo, hi) = (a[1].min(b[1]) - r - 1, a[1].max(b[1]) + r + 1);
    // The least y whose forward difference is not negative.
    let (mut l, mut h) = (lo, hi);
    while l < h {
        let m = l + (h - l) / 2;
        if g(m + 1) >= g(m) { h = m } else { l = m + 1 }
    }
    let best = l;
    if g(best) > limit {
        return None;
    }
    let (mut l, mut h) = (lo, best);
    while l < h {
        let m = l + (h - l) / 2;
        if g(m) <= limit { h = m } else { l = m + 1 }
    }
    let first = l;
    let (mut l, mut h) = (best, hi);
    while l < h {
        let m = l + (h - l + 1) / 2;
        if g(m) <= limit { l = m } else { h = m - 1 }
    }
    Some([first, l + 1])
}

fn isqrt(n: u128) -> u128 {
    n.isqrt()
}
