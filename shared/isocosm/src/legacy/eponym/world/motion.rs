// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Canonical fixed-point motion over the generated world's exact occupancy.
//! One voxel is 65536 quanta. Position is the feet centre; cell genesis centres
//! x/z in the column. Velocity is quanta/second. Every accepted step is 1/60 s.
//! The step itself is solved on the game side (rulings 233 and 597): the sim
//! admits what the solver it is handed returns, and never links one.

use isometer_core::ground::Ground;
use serde::{Deserialize, Serialize};

pub const MOTION_SCALE: i64 = 65_536;
const S: i64 = MOTION_SCALE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionPose {
    pub position: [i64; 3],
    pub velocity: [i64; 3],
    pub grounded: bool,
    pub step: u64,
    pub fall_origin_y: Option<i64>,
}
impl MotionPose {
    pub fn at_cell(at: [i32; 3]) -> Self {
        Self {
            position: [
                i64::from(at[0]) * S + S / 2,
                i64::from(at[1]) * S,
                i64::from(at[2]) * S + S / 2,
            ],
            velocity: [0; 3],
            grounded: true,
            step: 0,
            fall_origin_y: None,
        }
    }
    pub fn validate(self) -> Result<(), MotionError> {
        self.cell()?;
        if self.grounded != self.fall_origin_y.is_none() || (self.grounded && self.velocity[1] != 0)
        {
            return Err(MotionError::InvalidPose);
        }
        if self
            .velocity
            .iter()
            .any(|v| v.unsigned_abs() > (32 * S) as u64)
            || self.fall_origin_y.is_some_and(|y| {
                y < self.position[1] || y.unsigned_abs() > (i64::from(i32::MAX) * S) as u64
            })
        {
            return Err(MotionError::InvalidPose);
        }
        Ok(())
    }
    pub fn cell(self) -> Result<[i32; 3], MotionError> {
        let mut cell = [0; 3];
        for (out, value) in cell.iter_mut().zip(self.position) {
            *out = i32::try_from(value.div_euclid(S)).map_err(|_| MotionError::Overflow)?;
        }
        Ok(cell)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionInput {
    pub move_x: i16,
    pub move_z: i16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionRules {
    pub revision: u32,
    pub speed: i64,
    pub gravity: i64,
    pub terminal_speed: i64,
    pub half_width: i64,
    pub height: i64,
}
impl Default for MotionRules {
    fn default() -> Self {
        Self {
            revision: 1,
            speed: 4 * S,
            gravity: 642253,
            terminal_speed: 20 * S,
            half_width: S / 4,
            height: 2 * S,
        }
    }
}
impl MotionRules {
    pub fn validate(self) -> Result<(), MotionError> {
        if !(1..=crate::legacy::eponym::world::MOVEMENT_PROFILE_REVISION).contains(&self.revision)
            || !(1..=16 * S).contains(&self.speed)
            || !(1..=32 * S).contains(&self.gravity)
            || !(1..=32 * S).contains(&self.terminal_speed)
            || !(S / 16..=S).contains(&self.half_width)
            || !(S / 4..=4 * S).contains(&self.height)
        {
            return Err(MotionError::InvalidRules);
        }
        Ok(())
    }
    pub fn validate_projection(self) -> Result<(), MotionError> {
        if self.revision != crate::legacy::eponym::world::MOVEMENT_PROFILE_REVISION
            || !(0..=16 * S).contains(&self.speed)
            || !(1..=32 * S).contains(&self.gravity)
            || !(1..=32 * S).contains(&self.terminal_speed)
            || !(S / 16..=S).contains(&self.half_width)
            || !(S / 4..=4 * S).contains(&self.height)
        {
            return Err(MotionError::InvalidRules);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionError {
    InvalidRules,
    InvalidInput,
    Overflow,
    InvalidPose,
    Collision,
    Physics,
    InvalidProfile,
    MissingEnvelopeAnchor(isometer_core::PartId),
    /// No solver was handed the state, so no step can be solved.
    NoSolver,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionOutcome {
    pub pose: MotionPose,
    pub landed_distance: Option<i32>,
}

/// One fixed step over the world's ground, as a game solves it.
pub type Solve =
    fn(&Ground, MotionPose, MotionInput, MotionRules) -> Result<MotionOutcome, MotionError>;

/// The game's motion solver, held beside the state and never saved, compared
/// or replayed as state. A host hands its own; replaying a save with motion
/// in it needs the same one.
#[derive(Clone, Copy)]
pub struct MotionSolver(Solve);

impl MotionSolver {
    pub const fn new(solve: Solve) -> Self {
        Self(solve)
    }
    /// No solver: every motion step is refused by name.
    pub const NONE: Self = Self(|_, _, _, _| Err(MotionError::NoSolver));
    /// A step admitted without moving, for runs that need motion recorded
    /// but not solved.
    pub const STILL: Self = Self(still);

    pub(crate) fn advance(
        self,
        ground: &Ground,
        pose: MotionPose,
        input: MotionInput,
        rules: MotionRules,
    ) -> Result<MotionOutcome, MotionError> {
        (self.0)(ground, pose, input, rules)
    }
}

fn still(
    _: &Ground,
    pose: MotionPose,
    _: MotionInput,
    _: MotionRules,
) -> Result<MotionOutcome, MotionError> {
    let step = pose.step.checked_add(1).ok_or(MotionError::Overflow)?;
    Ok(MotionOutcome {
        pose: MotionPose { step, ..pose },
        landed_distance: None,
    })
}

impl Default for MotionSolver {
    fn default() -> Self {
        Self::NONE
    }
}

/// A solver is not state: two states differ only in what they hold.
impl PartialEq for MotionSolver {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for MotionSolver {}

impl std::fmt::Debug for MotionSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MotionSolver")
    }
}
