// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! One explicit fixed-point trajectory for supported transfers, foot steps and turns.
use super::{Point, UNIT};
use crate::PartId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stage {
    pub foot: PartId,
    pub root: Point,
    /// Zero denotes a planted body transfer, permitted only in a turning stride.
    pub lift: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<Point>,
    /// Body orientation reached before this foot lifts, relative to initial yaw.
    #[serde(default, skip_serializing_if = "zero")]
    pub turn: i64,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stride {
    pub stages: Vec<Stage>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub clearance: bool,
    /// Signed quarter turn. Omission keeps the original ordinary stride format.
    #[serde(default, skip_serializing_if = "zero")]
    pub turn: i8,
}
fn zero<T: Default + PartialEq>(value: &T) -> bool {
    *value == T::default()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrideFrame {
    pub root: Point,
    pub feet: BTreeMap<PartId, Point>,
    pub bearing: BTreeSet<PartId>,
    pub turn: i64,
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
        if !(0..=UNIT).contains(&phase)
            || self.stages.len() > 64
            || a.keys().ne(b.keys())
            || ![-1, 0, 1].contains(&self.turn)
        {
            return None;
        }
        let mut seen = BTreeSet::new();
        let mut end = a.clone();
        let mut angle = 0;
        for s in &self.stages {
            let old = *end.get(&s.foot)?;
            let goal = s.target.unwrap_or(b[&s.foot]);
            if self.turn == 0 {
                if !seen.insert(s.foot) || s.lift <= 0 || s.target.is_some() || s.turn != 0 {
                    return None;
                }
            } else {
                let direction = i64::from(self.turn);
                if s.turn.unsigned_abs() > UNIT as u64
                    || s.target.is_none()
                    || s.lift < 0
                    || s.turn * direction < angle * direction
                    || s.turn.abs() > UNIT
                    || (s.turn != angle && s.lift != 0)
                    || (s.lift == 0 && goal != old)
                {
                    return None;
                }
            }
            end.insert(s.foot, goal);
            angle = s.turn;
        }
        if end != *b || (self.turn != 0 && angle != i64::from(self.turn) * UNIT) {
            return None;
        }
        let units = self
            .stages
            .iter()
            .map(|s| if s.lift == 0 { 1 } else { 4 })
            .sum::<i64>()
            + 1;
        let mut scaled = phase * units;
        let mut feet = a.clone();
        let mut root = from;
        let mut turn = 0;
        for s in &self.stages {
            let duration = if s.lift == 0 { UNIT } else { 4 * UNIT };
            let goal = s.target.unwrap_or(b[&s.foot]);
            if scaled >= duration {
                feet.insert(s.foot, goal);
                root = s.root;
                turn = s.turn;
                scaled -= duration;
                continue;
            }
            let mut bearing = feet.keys().copied().collect::<BTreeSet<_>>();
            if scaled <= UNIT {
                return Some(StrideFrame {
                    root: transfer(root, s.root, scaled, self.clearance),
                    feet,
                    bearing,
                    turn: turn + (s.turn - turn) * scaled / UNIT,
                });
            }
            let t = (scaled - UNIT) / 3;
            let start = feet[&s.foot];
            let height = start[1].max(goal[1]).checked_add(s.lift)?;
            let mut sole = lerp(start, goal, ((t - UNIT / 4) * 2).clamp(0, UNIT));
            sole[1] = if t < UNIT / 4 {
                (i128::from(start[1])
                    + (i128::from(height) - i128::from(start[1])) * i128::from(t) * 4
                        / i128::from(UNIT)) as i64
            } else if t > UNIT * 3 / 4 {
                (i128::from(goal[1])
                    + (i128::from(height) - i128::from(goal[1])) * i128::from(UNIT - t) * 4
                        / i128::from(UNIT)) as i64
            } else {
                height
            };
            feet.insert(s.foot, sole);
            if t > 0 && t < UNIT {
                bearing.remove(&s.foot);
            }
            return Some(StrideFrame {
                root: s.root,
                feet,
                bearing,
                turn: s.turn,
            });
        }
        Some(StrideFrame {
            root: transfer(root, to, scaled, self.clearance),
            bearing: feet.keys().copied().collect(),
            feet,
            turn,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn smaller_pivots_replant_between_rotations_without_dragging_bearing_soles() {
        let a = BTreeMap::from([(PartId(1), [-UNIT, 0, 0]), (PartId(2), [UNIT, 0, 0])]);
        let mut b = a.clone();
        b.get_mut(&PartId(1)).unwrap()[0] += UNIT;
        let root = [0, 2 * UNIT, 0];
        let stride = Stride {
            turn: 1,
            clearance: true,
            stages: vec![
                Stage {
                    foot: PartId(1),
                    root,
                    lift: 0,
                    turn: UNIT / 2,
                    target: Some(a[&PartId(1)]),
                },
                Stage {
                    foot: PartId(1),
                    root,
                    lift: UNIT,
                    turn: UNIT / 2,
                    target: Some(b[&PartId(1)]),
                },
                Stage {
                    foot: PartId(1),
                    root,
                    lift: 0,
                    turn: UNIT,
                    target: Some(b[&PartId(1)]),
                },
            ],
        };
        let mut last = stride.sample(root, root, &a, &b, 0).unwrap();
        let mut rotated = false;
        let mut lifted = false;
        for i in 1..=stride.samples() {
            let frame = stride
                .sample(root, root, &a, &b, i * UNIT / stride.samples())
                .unwrap();
            if frame.turn != last.turn {
                rotated = true;
                assert_eq!(frame.bearing.len(), 2);
                if last.bearing.len() == 2 {
                    assert_eq!(frame.feet, last.feet);
                }
                for (id, p) in &frame.feet {
                    assert_eq!([p[0], p[2]], [last.feet[id][0], last.feet[id][2]]);
                }
            }
            lifted |= frame.bearing.len() == 1;
            last = frame;
        }
        assert!(rotated && lifted);
        assert_eq!(last.feet, b);
        assert_eq!(last.turn, UNIT);
        let mut forged = stride;
        forged.stages[0].target = Some(b[&PartId(1)]);
        assert!(forged.sample(root, root, &a, &b, UNIT / 2).is_none());
    }
    #[test]
    fn body_transfers_before_takeoff_and_all_other_feet_stay_anchored() {
        let a = BTreeMap::from([(PartId(1), [-UNIT, 0, 0]), (PartId(2), [UNIT, 0, 0])]);
        let b = BTreeMap::from([(PartId(1), [0, 0, 0]), (PartId(2), [UNIT, 0, 0])]);
        let stride = Stride {
            turn: 0,
            clearance: false,
            stages: vec![Stage {
                foot: PartId(1),
                root: [UNIT, 2 * UNIT, 0],
                lift: UNIT,
                target: None,
                turn: 0,
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
            target: None,
            turn: 0,
        };
        assert!(
            Stride {
                turn: 0,
                clearance: false,
                stages: vec![s.clone(), s.clone()]
            }
            .sample([0; 3], [0; 3], &a, &b, 0)
            .is_none()
        );
        assert!(
            Stride {
                turn: 0,
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
