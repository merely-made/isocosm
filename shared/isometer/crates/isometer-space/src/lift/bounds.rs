// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exact surface extrema of a procedural window without materializing its columns.
use super::{Lattice, SPANS};

impl Lattice {
    pub(crate) fn window_heights(&self, min: [i64; 2], max: [i64; 2]) -> [i64; 2] {
        // Within a lattice cell the unstepped terrain is bilinear. Separate the
        // knot itself because a cliff span excludes both ends.
        let cuts = |axis: usize| {
            let mut cuts = vec![min[axis]];
            for i in 0..=i64::from(SPANS) {
                for c in [i * self.spacing, i * self.spacing + 1] {
                    if min[axis] < c && c < max[axis] {
                        cuts.push(c);
                    }
                }
            }
            cuts.push(max[axis]);
            cuts.dedup();
            cuts
        };
        let xs = cuts(0);
        let zs = cuts(1);
        let length = self.spacing * i64::from(SPANS);
        let mut bounds = [i64::MAX, i64::MIN];
        for z in zs.windows(2) {
            for x in xs.windows(2) {
                let min = [x[0], z[0]];
                let max = [x[1] - 1, z[1] - 1];
                let ends = |axis: usize| {
                    let mut ends = vec![min[axis], max[axis]];
                    // Each active cliff drops monotonically along its inward axis.
                    // The first and last coordinate of each constant integer drop
                    // delimit rectangles on which the extrema are at corners.
                    for cliff in &self.cliffs {
                        let at = |v| {
                            let mut p = min;
                            p[axis] = v;
                            cliff.at(p[0], p[1], length)
                        };
                        let mut lo = min[axis];
                        while at(lo) != at(max[axis]) {
                            let initial = at(lo);
                            let mut first = lo + 1;
                            let mut last = max[axis];
                            while first < last {
                                let mid = first + (last - first) / 2;
                                if at(mid) == initial {
                                    first = mid + 1;
                                } else {
                                    last = mid;
                                }
                            }
                            ends.extend([first - 1, first]);
                            lo = first;
                        }
                    }
                    ends.sort_unstable();
                    ends.dedup();
                    ends
                };
                let x_ends = ends(0);
                for z in ends(1) {
                    for &x in &x_ends {
                        let top =
                            self.numerator(x as u64, z as u64)
                                .div_euclid(self.denominator()) as i64;
                        bounds = [bounds[0].min(top), bounds[1].max(top)];
                    }
                }
            }
        }
        bounds
    }
}
