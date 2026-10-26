// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Conservative integer capsule/voxel queries. Adaptive subdivision proves
//! empty space; unresolved contacts within 1/32 voxel remain occupied.
use super::{Bounds, Point};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Capsule {
    pub a: Point,
    pub b: Point,
    pub radius: i64,
}

impl Capsule {
    pub fn bounds(self) -> Bounds {
        [
            [0, 1, 2].map(|k| self.a[k].min(self.b[k]) - self.radius),
            [0, 1, 2].map(|k| self.a[k].max(self.b[k]) + self.radius),
        ]
    }

    pub fn hits(self, bounds: Bounds) -> bool {
        fn squared(v: Point) -> i128 {
            v.into_iter().map(|v| i128::from(v).pow(2)).sum()
        }
        fn visit(a: Point, b: Point, radius: i64, bounds: Bounds) -> bool {
            let mid = [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) / 2);
            let delta = [0, 1, 2].map(|k| mid[k] - mid[k].clamp(bounds[0][k], bounds[1][k]));
            let distance = squared(delta);
            if distance <= i128::from(radius).pow(2) {
                return true;
            }
            let half = [a, b]
                .into_iter()
                .map(|p| (squared([0, 1, 2].map(|k| p[k] - mid[k])) as u128).isqrt() as i64 + 1)
                .max()
                .unwrap();
            if distance > i128::from(radius + half).pow(2) {
                return false;
            }
            if half <= 32 {
                return true;
            }
            visit(a, mid, radius, bounds) || visit(mid, b, radius, bounds)
        }
        visit(self.a, self.b, self.radius, bounds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::legs::UNIT as U;
    #[test]
    fn empty_shin_box_corners_clear_but_real_capsule_contacts_do_not() {
        let shin = Capsule {
            a: [0, 8 * U, 0],
            b: [U, 0, 0],
            radius: U / 3,
        };
        assert!(!shin.hits([[0, 0, 0], [U / 2, U, U]]));
        assert!(shin.hits([[U / 2, 0, -U / 2], [U, U, U / 2]]));
        assert!(shin.hits([[-U / 2, 7 * U, -U / 2], [U / 2, 8 * U, U / 2]]));
        let point = Capsule {
            a: [0; 3],
            b: [0; 3],
            radius: U,
        };
        assert!(point.hits([[U, -U, -U], [2 * U, U, U]]));
        assert!(!point.hits([[2 * U, -U, -U], [3 * U, U, U]]));
    }
}
