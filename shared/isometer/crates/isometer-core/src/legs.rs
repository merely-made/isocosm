// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Paid serial leg geometry, shared by native admission and presentation.
//! Fixed-point arithmetic keeps the native answer independent of float math.
use crate::{BodyDocument, PartId, Role, classify};
use std::collections::BTreeSet;
mod collision;
pub use collision::Capsule;
mod stride;
pub use stride::{Stage, Stride, StrideFrame};
mod rotation;
pub use rotation::Rotation;
pub mod support;

pub const UNIT: i64 = 1024;
pub type Point = [i64; 3];
pub type Bounds = [Point; 2];

#[derive(Clone, Copy, Debug)]
pub struct Skin {
    pub upper: Capsule,
    pub lower: Capsule,
    pub pad: Bounds,
    pub connector: Capsule,
}
impl Skin {
    /// Conservative bounds of these already solved links, pad and connector.
    pub fn occupied(&self) -> [Bounds; 3] {
        let neck = self.connector.bounds();
        [
            self.upper.bounds(),
            self.lower.bounds(),
            [
                [0, 1, 2].map(|k| self.pad[0][k].min(neck[0][k])),
                [0, 1, 2].map(|k| self.pad[1][k].max(neck[1][k])),
            ],
        ]
    }
}

#[derive(Clone, Debug)]
pub struct Leg {
    pub upper: PartId,
    pub lower: PartId,
    pub foot: PartId,
    pub shoulder: Point,
    pub knee: Point,
    pub sole: Point,
    pub half: Point,
    pub radius: i64,
    pub lengths: [i64; 2],
    pub ankle_offset: Point,
}

fn sub(a: Point, b: Point) -> Point {
    [0, 1, 2].map(|k| a[k] - b[k])
}
fn dot(a: Point, b: Point) -> i128 {
    (0..3).map(|k| i128::from(a[k]) * i128::from(b[k])).sum()
}
fn length(a: Point) -> i64 {
    (dot(a, a) as u128).isqrt() as i64
}
fn scale(a: Point, n: i64, d: i64) -> Point {
    a.map(|v| (i128::from(v) * i128::from(n) / i128::from(d)) as i64)
}
fn centre(doc: &BodyDocument, id: PartId) -> Option<Point> {
    let p = doc.part(id)?;
    Some(doc.place(id, p.half_extent)?.map(|v| i64::from(v) * UNIT))
}

/// The existing terminal-foot, upright-shin, lateral-thigh graph convention.
/// A severed link withdraws the chain; names and volume tags are irrelevant.
pub fn chains(doc: &BodyDocument) -> Vec<Leg> {
    let parents: BTreeSet<_> = doc
        .living()
        .filter_map(|p| p.attachment.map(|a| a.parent))
        .collect();
    let mut out = Vec::new();
    let mut used = BTreeSet::new();
    for foot in doc.living().filter(|p| !parents.contains(&p.id)) {
        let Some(ankle) = foot.attachment else {
            continue;
        };
        let Some(lower) = doc.part(ankle.parent).filter(|p| !p.severed) else {
            continue;
        };
        let Some(knee) = lower.attachment else {
            continue;
        };
        let Some(upper) = doc.part(knee.parent).filter(|p| !p.severed) else {
            continue;
        };
        let Some(hip) = upper.attachment else {
            continue;
        };
        if [upper, lower, foot].iter().any(|p| {
            p.half_extent.iter().any(|h| *h <= 0 || *h > 1024)
                || classify(p.half_extent) != Role::Limb
        }) || lower.half_extent[1] <= lower.half_extent[0].max(lower.half_extent[2])
            || knee.offset[1] >= 0
            || ankle.offset[1] >= 0
            || hip.offset[0] == 0
            || used.contains(&upper.id)
        {
            continue;
        }
        let Some(c) = centre(doc, upper.id) else {
            continue;
        };
        let Some(l) = centre(doc, lower.id) else {
            continue;
        };
        let Some(f) = centre(doc, foot.id) else {
            continue;
        };
        let Some(a) = doc.place(upper.id, [0; 3]) else {
            continue;
        };
        let Some(b) = doc.place(upper.id, [1, 0, 0]) else {
            continue;
        };
        let outward = [0, 1, 2].map(|k| i64::from(b[k] - a[k]) * i64::from(hip.offset[0].signum()));
        let radius = [upper, lower, foot]
            .iter()
            .flat_map(|p| p.half_extent)
            .min()
            .unwrap() as i64
            * UNIT
            / 3;
        let hx = i64::from(upper.half_extent[0]) * UNIT;
        let shoulder = [0, 1, 2].map(|k| c[k] - outward[k] * (hx - radius));
        let rest_knee = [0, 1, 2].map(|k| c[k] + outward[k] * (hx - radius * 3 / 2));
        let sole = [f[0], f[1] - i64::from(foot.half_extent[1]) * UNIT, f[2]];
        let rest_ankle = [l[0], sole[1] + radius * 3, l[2]];
        // Axis permutation from the foot's exact attachment yaw.
        let Some(fa) = doc.place(foot.id, [0; 3]) else {
            continue;
        };
        let Some(fb) = doc.place(foot.id, foot.half_extent.map(|h| h * 2)) else {
            continue;
        };
        let mut half = [0, 1, 2].map(|k| i64::from(fb[k] - fa[k]).abs() * UNIT / 2);
        half[1] = radius * 3 / 2;
        out.push(Leg {
            upper: upper.id,
            lower: lower.id,
            foot: foot.id,
            shoulder,
            knee: rest_knee,
            sole,
            half,
            radius,
            ankle_offset: sub(rest_ankle, sole),
            lengths: [
                length(sub(rest_knee, shoulder)),
                length(sub(rest_ankle, rest_knee)),
            ],
        });
        used.insert(upper.id);
    }
    out
}

impl Leg {
    /// Fixed lengths, with at most fixed-point rounding error. Refuse targets
    /// outside the paid chain's reach rather than stretching a link.
    pub fn solve(&self, sole: Point) -> Option<[Point; 2]> {
        let target = [0, 1, 2].map(|k| sole[k] + self.ankle_offset[k]);
        let delta = sub(target, self.shoulder);
        let distance = length(delta);
        let [a, b] = self.lengths;
        if distance <= (a - b).abs() + 8 || distance >= a + b - 8 {
            return None;
        }
        let along = ((i128::from(a).pow(2) - i128::from(b).pow(2) + i128::from(distance).pow(2))
            / (2 * i128::from(distance))) as i64;
        // A horizontal rest thigh can be collinear with a raised ankle.
        // Projecting that thigh alone flips the knee from above to below the
        // ground on the first lift sample. Give its anatomical bend plane a
        // stable upward component through that singularity.
        let pole = sub(self.knee, self.shoulder);
        let pole = [pole[0], pole[1] + a, pole[2]];
        let projection = (dot(pole, delta) / i128::from(distance)) as i64;
        let parallel = scale(delta, projection, distance);
        let mut bend = sub(pole, parallel);
        if length(bend) < 8 {
            bend = [-delta[1], delta[0], 0];
        }
        let bend_length = length(bend);
        if bend_length == 0 {
            return None;
        }
        let height =
            ((i128::from(a).pow(2) - i128::from(along).pow(2)).max(0) as u128).isqrt() as i64;
        let forward = scale(delta, along, distance);
        let side = scale(bend, height, bend_length);
        let knee = [0, 1, 2].map(|k| self.shoulder[k] + forward[k] + side[k]);
        Some([knee, target])
    }

    /// Conservative occupied bounds of the same two capsules and rigid pad
    /// used by the smooth renderer. Bounds remain fixed-point body-local.
    pub fn occupied(&self, sole: Point) -> Option<[(PartId, Bounds); 3]> {
        let [upper, lower, foot] = self.skin(sole)?.occupied();
        Some([(self.upper, upper), (self.lower, lower), (self.foot, foot)])
    }
    pub fn skin(&self, sole: Point) -> Option<Skin> {
        let [knee, ankle] = self.solve(sole)?;
        let centre = [sole[0], sole[1] + self.half[1], sole[2]];
        let pad = [
            [0, 1, 2].map(|k| centre[k] - self.half[k]),
            [0, 1, 2].map(|k| centre[k] + self.half[k]),
        ];
        Some(Skin {
            upper: Capsule {
                a: self.shoulder,
                b: knee,
                radius: self.radius,
            },
            lower: Capsule {
                a: knee,
                b: ankle,
                radius: self.radius,
            },
            pad,
            connector: Capsule {
                a: ankle,
                b: centre,
                radius: self.radius * 3 / 5,
            },
        })
    }
}

#[cfg(test)]
mod tests;
