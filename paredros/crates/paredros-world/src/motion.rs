// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Canonical fixed-point motion over the generated world's exact occupancy.
//! One voxel is 65536 quanta. Position is the feet centre; cell genesis centres
//! x/z in the column. Velocity is quanta/second. Every accepted step is 1/60 s.
//! Conatus is a disposable local query; its result is quantized before reuse.

use conatus::{
    BodyDesc, BodyWorld, CharacterConfig, ColliderDesc, ColliderId, ColliderShape, Transform,
};
use mesocosm_core::places::Ground;
use serde::{Deserialize, Serialize};

pub const MOTION_SCALE: i64 = 65_536;
const S: i64 = MOTION_SCALE;
const DT: f32 = 1.0 / 60.0;

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
        if self.revision != 1
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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionError {
    InvalidRules,
    InvalidInput,
    Overflow,
    InvalidPose,
    Collision,
    Physics,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionOutcome {
    pub pose: MotionPose,
    pub landed_distance: Option<i32>,
}

pub(crate) fn advance(
    ground: &Ground,
    pose: MotionPose,
    input: MotionInput,
    rules: MotionRules,
) -> Result<MotionOutcome, MotionError> {
    rules.validate()?;
    if input.move_x == i16::MIN || input.move_z == i16::MIN {
        return Err(MotionError::InvalidInput);
    }
    let cell = pose.cell()?;
    validate_clear(ground, pose.position, rules)?;
    pose.validate()?;
    // Translate around the current cell, so distant worlds do not lose f32 precision.
    let base = cell.map(|v| i64::from(v) * S);
    let local = std::array::from_fn::<_, 3, _>(|i| (pose.position[i] - base[i]) as f32 / S as f32);
    let width = rules.half_width as f32 / S as f32;
    let height = rules.height as f32 / S as f32;
    let mut spatial = BodyWorld::new([0.; 3]);
    // Bounds cover maximum permitted displacement (32/60), shape and clearance.
    for x in -3..=3 {
        for y in -2..=6 {
            for z in -3..=3 {
                let at = [
                    cell[0].checked_add(x).ok_or(MotionError::Overflow)?,
                    cell[1].checked_add(y).ok_or(MotionError::Overflow)?,
                    cell[2].checked_add(z).ok_or(MotionError::Overflow)?,
                ];
                if ground.solid(at) {
                    spatial
                        .spawn(box_body(
                            [x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5],
                            [0.5; 3],
                            false,
                        ))
                        .map_err(|_| MotionError::Physics)?;
                }
            }
        }
    }
    let id = spatial
        .spawn(box_body(
            [local[0], local[1] + height / 2., local[2]],
            [width, height / 2., width],
            true,
        ))
        .map_err(|_| MotionError::Physics)?;
    spatial.step(DT).map_err(|_| MotionError::Physics)?;
    let mut direction = [input.move_x as f32 / 32767., input.move_z as f32 / 32767.];
    let length = (direction[0] * direction[0] + direction[1] * direction[1]).sqrt();
    if length > 1. {
        direction = direction.map(|v| v / length);
    }
    let vy = (pose.velocity[1] - rules.gravity / 60).max(-rules.terminal_speed);
    let velocity = [
        quantize(direction[0] * rules.speed as f32 / S as f32)?,
        vy,
        quantize(direction[1] * rules.speed as f32 / S as f32)?,
    ];
    let requested = velocity.map(|v| v as f32 / S as f32 * DT);
    let movement = spatial
        .move_character(
            ColliderId::new(id, 0),
            requested,
            DT,
            CharacterConfig {
                offset: 0.002,
                snap_to_ground: Some(0.02),
                ..CharacterConfig::default()
            },
        )
        .map_err(|_| MotionError::Physics)?;
    let mut position = pose.position;
    for i in 0..3 {
        position[i] = base[i]
            .checked_add(quantize(local[i] + movement.applied[i])?)
            .ok_or(MotionError::Overflow)?;
    }
    // Exact occupancy validation rejects an embedded start/result instead of
    // admitting an invalid physics result. Surface touching is allowed.
    validate_clear(ground, position, rules)?;
    let origin = pose
        .fall_origin_y
        .unwrap_or(pose.position[1])
        .max(pose.position[1]);
    let landed_distance = if movement.grounded && !pose.grounded {
        Some(
            i32::try_from(origin.saturating_sub(position[1]).max(0) / S)
                .map_err(|_| MotionError::Overflow)?,
        )
    } else {
        None
    };
    let mut next = MotionPose {
        position,
        velocity,
        grounded: movement.grounded,
        step: pose.step.checked_add(1).ok_or(MotionError::Overflow)?,
        fall_origin_y: if movement.grounded {
            None
        } else {
            Some(origin.max(position[1]))
        },
    };
    if next.grounded {
        next.velocity[1] = 0;
    }
    for axis in [0, 2] {
        next.velocity[axis] = (next.position[axis] - pose.position[axis]) * 60;
    }
    next.validate()?;
    Ok(MotionOutcome {
        pose: next,
        landed_distance,
    })
}
fn quantize(v: f32) -> Result<i64, MotionError> {
    if !v.is_finite() || v.abs() > 128. {
        return Err(MotionError::Overflow);
    }
    Ok((v * S as f32).round() as i64)
}
fn box_body(center: [f32; 3], half_extents: [f32; 3], moving: bool) -> BodyDesc {
    (if moving {
        BodyDesc::kinematic_position()
    } else {
        BodyDesc::fixed()
    })
    .at(Transform::from_translation(center))
    .with_collider(ColliderDesc::new(ColliderShape::Box { half_extents }))
}
fn validate_clear(ground: &Ground, at: [i64; 3], rules: MotionRules) -> Result<(), MotionError> {
    let lo = [at[0] - rules.half_width, at[1], at[2] - rules.half_width];
    let hi = [
        at[0] + rules.half_width,
        at[1] + rules.height,
        at[2] + rules.half_width,
    ];
    for x in lo[0].div_euclid(S)..=(hi[0] - 1).div_euclid(S) {
        for y in lo[1].div_euclid(S)..=(hi[1] - 1).div_euclid(S) {
            for z in lo[2].div_euclid(S)..=(hi[2] - 1).div_euclid(S) {
                let cell = [
                    i32::try_from(x).map_err(|_| MotionError::Overflow)?,
                    i32::try_from(y).map_err(|_| MotionError::Overflow)?,
                    i32::try_from(z).map_err(|_| MotionError::Overflow)?,
                ];
                if ground.solid(cell) {
                    return Err(MotionError::Collision);
                }
            }
        }
    }
    Ok(())
}
