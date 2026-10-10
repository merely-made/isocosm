// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One fixed motion step solved over conatus's character controller (ruling
//! 233): a disposable local query whose result is quantized before the sim
//! admits it.

use conatus::{
    BodyDesc, BodyWorld, CharacterConfig, ColliderDesc, ColliderId, ColliderShape, Transform,
};
use eponym_play::{
    MOTION_SCALE, MotionError, MotionInput, MotionOutcome, MotionPose, MotionRules, MotionSolver,
};
use isometer_core::ground::Ground;

const S: i64 = MOTION_SCALE;
const DT: f32 = 1.0 / 60.0;

/// The solver a host hands its state.
pub const SOLVER: MotionSolver = MotionSolver::new(advance);

pub fn advance(
    ground: &Ground,
    pose: MotionPose,
    input: MotionInput,
    rules: MotionRules,
) -> Result<MotionOutcome, MotionError> {
    match rules.revision {
        1 => rules.validate()?,
        eponym_play::MOVEMENT_PROFILE_REVISION => rules.validate_projection()?,
        _ => return Err(MotionError::InvalidRules),
    }
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
