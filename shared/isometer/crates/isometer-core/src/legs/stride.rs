// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Explicit fixed-point body transfers and single-foot swings. Terrain and
//! admission remain the caller's; sampling never chooses a different motion.
use super::{Point, UNIT};
use crate::PartId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub foot: PartId,
    /// Absolute fixed-point root, reached with every current sole planted.
    pub root: Point,
    /// Clearance above the higher of this foot's two sole heights.
    pub lift: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stride {
    pub stages: Vec<Stage>,
    /// Rise to the higher endpoint before crossing; lower after crossing.
    /// Omission preserves an earlier straight root transfer.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub clearance: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrideFrame {
    pub root: Point,
    /// Absolute sole centres; a body shift cannot drag a planted foot.
    pub feet: BTreeMap<PartId, Point>,
    pub bearing: BTreeSet<PartId>,
}

fn lerp(a: Point, b: Point, t: i64) -> Point {
    [0, 1, 2].map(|k| {
        (i128::from(a[k])
            + (i128::from(b[k]) - i128::from(a[k])) * i128::from(t) / i128::from(UNIT))
            as i64
    })
}

fn transfer(a: Point, b: Point, t: i64, clearance: bool) -> Point {
    if !clearance || a[1] == b[1] {
        return lerp(a, b, t);
    }
    let bend = if a[1] < b[1] {
        [a[0], b[1], a[2]]
    } else {
        [b[0], a[1], b[2]]
    };
    if bend == a || bend == b {
        return lerp(a, b, t);
    }
    if t <= UNIT / 2 {
        lerp(a, bend, t * 2)
    } else {
        lerp(bend, b, t * 2 - UNIT)
    }
}

impl Stride {
    pub fn samples(&self) -> i64 {
        128.max(self.stages.len() as i64 * 16)
    }
    pub fn sample(
        &self,
        from: Point,
        to: Point,
        a: &BTreeMap<PartId, Point>,
        b: &BTreeMap<PartId, Point>,
        phase: i64,
    ) -> Option<StrideFrame> {
        if !(0..=UNIT).contains(&phase) || self.stages.len() > 64 || a.keys().ne(b.keys()) {
            return None;
        }
        let mut seen = BTreeSet::new();
        for s in &self.stages {
            if !a.contains_key(&s.foot) || !seen.insert(s.foot) || s.lift <= 0 {
                return None;
            }
        }
        if a.iter().any(|(id, p)| *p != b[id] && !seen.contains(id)) {
            return None;
        }
        let units = self.stages.len() as i64 * 4 + 1;
        let scaled = phase * units;
        let index = (scaled / (4 * UNIT)) as usize;
        let mut feet = a.clone();
        let mut root = from;
        for stage in self.stages.iter().take(index) {
            feet.insert(stage.foot, b[&stage.foot]);
            root = stage.root;
        }
        let mut bearing = feet.keys().copied().collect::<BTreeSet<_>>();
        let root = if let Some(stage) = self.stages.get(index) {
            let t = scaled - index as i64 * 4 * UNIT;
            if t <= UNIT {
                transfer(root, stage.root, t, self.clearance)
            } else {
                let t = (t - UNIT) / 3;
                let start = feet[&stage.foot];
                let end = b[&stage.foot];
                let height = start[1].max(end[1]).checked_add(stage.lift)?;
                let travel = ((t - UNIT / 4) * 2).clamp(0, UNIT);
                let mut sole = lerp(start, end, travel);
                sole[1] = if t < UNIT / 4 {
                    (i128::from(start[1])
                        + (i128::from(height) - i128::from(start[1])) * i128::from(t) * 4
                            / i128::from(UNIT)) as i64
                } else if t > UNIT * 3 / 4 {
                    (i128::from(end[1])
                        + (i128::from(height) - i128::from(end[1])) * i128::from(UNIT - t) * 4
                            / i128::from(UNIT)) as i64
                } else {
                    height
                };
                feet.insert(stage.foot, sole);
                if t > 0 && t < UNIT {
                    bearing.remove(&stage.foot);
                }
                stage.root
            }
        } else {
            let t = scaled - self.stages.len() as i64 * 4 * UNIT;
            transfer(root, to, t, self.clearance)
        };
        Some(StrideFrame {
            root,
            feet,
            bearing,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn body_transfers_before_takeoff_and_all_other_feet_stay_anchored() {
        let a = BTreeMap::from([(PartId(1), [-UNIT, 0, 0]), (PartId(2), [UNIT, 0, 0])]);
        let b = BTreeMap::from([(PartId(1), [0, 0, 0]), (PartId(2), [UNIT, 0, 0])]);
        let stride = Stride {
            clearance: false,
            stages: vec![Stage {
                foot: PartId(1),
                root: [UNIT, 2 * UNIT, 0],
                lift: UNIT,
            }],
        };
        let shifted = stride
            .sample(
                [0, 2 * UNIT, 0],
                [UNIT, 2 * UNIT, 0],
                &a,
                &b,
                (UNIT + 4) / 5,
            )
            .unwrap();
        assert_eq!(shifted.root, [UNIT, 2 * UNIT, 0]);
        assert_eq!(shifted.feet, a);
        assert_eq!(shifted.bearing.len(), 2);
        let swing = stride
            .sample([0, 2 * UNIT, 0], [UNIT, 2 * UNIT, 0], &a, &b, UNIT / 2)
            .unwrap();
        assert_eq!(swing.root, shifted.root);
        assert_eq!(swing.feet[&PartId(2)], a[&PartId(2)]);
        assert!(swing.feet[&PartId(1)][1] > 0);
        assert_eq!(swing.bearing, BTreeSet::from([PartId(2)]));
        assert_eq!(
            stride
                .sample([0, 2 * UNIT, 0], [UNIT, 2 * UNIT, 0], &a, &b, 0)
                .unwrap()
                .feet,
            a
        );
        assert_eq!(
            stride
                .sample([0, 2 * UNIT, 0], [UNIT, 2 * UNIT, 0], &a, &b, UNIT)
                .unwrap()
                .feet,
            b
        );
    }
    #[test]
    fn missing_duplicate_and_out_of_range_motion_is_refused() {
        let a = BTreeMap::from([(PartId(1), [0; 3])]);
        let b = BTreeMap::from([(PartId(1), [UNIT, 0, 0])]);
        assert!(
            Stride::default()
                .sample([0; 3], [0; 3], &a, &b, 0)
                .is_none()
        );
        let s = Stage {
            foot: PartId(1),
            root: [0; 3],
            lift: UNIT,
        };
        assert!(
            Stride {
                clearance: false,
                stages: vec![s.clone(), s.clone()]
            }
            .sample([0; 3], [0; 3], &a, &b, 0)
            .is_none()
        );
        assert!(
            Stride {
                stages: vec![s],
                clearance: false
            }
            .sample([0; 3], [0; 3], &a, &b, UNIT + 1)
            .is_none()
        );
    }
    #[test]
    fn clearance_transfers_rise_before_crossing_and_lower_after_crossing() {
        let a = [0, 0, 0];
        let b = [UNIT, UNIT, 0];
        assert_eq!(transfer(a, b, UNIT / 2, true), [0, UNIT, 0]);
        assert_eq!(transfer(b, a, UNIT / 2, true), [0, UNIT, 0]);
        assert_eq!(transfer(a, b, UNIT / 2, false), [UNIT / 2, UNIT / 2, 0]);
        assert_eq!(transfer(a, b, 0, true), a);
        assert_eq!(transfer(a, b, UNIT, true), b);
    }
}
