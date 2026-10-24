// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Deterministic horizontal frames for sampled turns; authored yaw stays exact.
use super::{Bounds, Point, UNIT};
use crate::Yaw;

const SCALE: i64 = 1 << 30;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rotation {
    pub cosine: i64,
    pub sine: i64,
}
impl Rotation {
    pub fn quarter_turn(from: Yaw, to: Yaw) -> Option<i8> {
        let n = |yaw| match yaw {
            Yaw::Zero => 0,
            Yaw::Quarter => 1,
            Yaw::Half => 2,
            Yaw::ThreeQuarter => 3,
        };
        match (n(to) - n(from) + 4) % 4 {
            1 => Some(1),
            3 => Some(-1),
            _ => None,
        }
    }
    pub fn at(yaw: Yaw, turn: i64) -> Self {
        let quarter = match yaw {
            Yaw::Zero => 0,
            Yaw::Quarter => 1,
            Yaw::Half => 2,
            Yaw::ThreeQuarter => 3,
        };
        let angle = (quarter * UNIT + turn.rem_euclid(4 * UNIT)).rem_euclid(4 * UNIT);
        let t = angle % UNIT;
        let x = (UNIT - t) * SCALE / UNIT;
        let z = t * SCALE / UNIT;
        let length = ((i128::from(x).pow(2) + i128::from(z).pow(2)) as u128).isqrt() as i128;
        let c = (i128::from(x) * i128::from(SCALE) / length) as i64;
        let s = (i128::from(z) * i128::from(SCALE) / length) as i64;
        let (cosine, sine) = match angle / UNIT {
            0 => (c, s),
            1 => (-s, c),
            2 => (-c, -s),
            _ => (s, -c),
        };
        Self { cosine, sine }
    }
    pub fn rotate(self, [x, y, z]: Point) -> Option<Point> {
        let c = i128::from(self.cosine);
        let s = i128::from(self.sine);
        let round = |n: i128| {
            i64::try_from((n + n.signum() * i128::from(SCALE / 2)) / i128::from(SCALE)).ok()
        };
        Some([
            round(c * i128::from(x) + s * i128::from(z))?,
            y,
            round(c * i128::from(z) - s * i128::from(x))?,
        ])
    }
    pub fn inverse(self, p: Point) -> Option<Point> {
        Self {
            sine: -self.sine,
            ..self
        }
        .rotate(p)
    }
    pub fn bounds(self, [lo, hi]: Bounds, origin: Point) -> Option<Bounds> {
        let mut out = [[i64::MAX; 3], [i64::MIN; 3]];
        for x in [lo[0], hi[0]] {
            for y in [lo[1], hi[1]] {
                for z in [lo[2], hi[2]] {
                    let p = self.rotate([x, y, z])?;
                    for k in 0..3 {
                        let v = p[k].checked_add(origin[k])?;
                        out[0][k] = out[0][k].min(v);
                        out[1][k] = out[1][k].max(v);
                    }
                }
            }
        }
        Some(out)
    }
    pub fn radians(self) -> f32 {
        (self.sine as f32).atan2(self.cosine as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_quarters_and_intermediate_inverse_preserve_planted_points() {
        for yaw in [Yaw::Zero, Yaw::Quarter, Yaw::Half, Yaw::ThreeQuarter] {
            assert_eq!(
                Rotation::at(yaw, 0)
                    .rotate([7, 2, -9])
                    .unwrap()
                    .map(|n| n as i32),
                yaw.rotate([7, 2, -9])
            );
            for t in -UNIT..=UNIT {
                let r = Rotation::at(yaw, t);
                let p = [17 * UNIT, -3 * UNIT, 11 * UNIT];
                let q = r.rotate(r.inverse(p).unwrap()).unwrap();
                assert!((0..3).all(|k| q[k].abs_diff(p[k]) <= 1));
            }
        }
    }
}
