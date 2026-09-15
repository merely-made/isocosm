// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Revision-2 fixed-point geometry for the persistent combat receipt.

use std::collections::BTreeSet;

use isometer_core::ground::Ground;
use isometer_core::{BodyDocument, PartId};
use paredros_identity::{BodyRevisionId, SubjectId};

use super::*;
use crate::{MOTION_SCALE, MotionPose, part_bounds};

/// A single exact visibility walk and the whole volley are bounded before
/// traversing terrain. Exhaustion rejects the receipt; it never becomes a
/// quiet miss.
const MAX_RAY_CELLS: i128 = 4_096;
const MAX_VOLLEY_RAY_CELLS: i128 = 65_536;

struct RayBudget(i128);

#[derive(Clone, Copy)]
struct FixedAabb {
    min: [i128; 3],
    max: [i128; 3],
}

pub(crate) fn resolve(
    actor: SubjectId,
    actor_revision: BodyRevisionId,
    actor_body: &BodyDocument,
    actor_pose: MotionPose,
    target: SubjectId,
    target_body: &BodyDocument,
    target_pose: MotionPose,
    ground: &Ground,
    receipts: &[StrikeReceipt],
    rules: CombatRules,
) -> Result<VolleyResolution, CombatError> {
    rules.validate()?;
    if actor == target {
        return Err(CombatError::SameSubject(actor));
    }
    if receipts.is_empty() {
        return Err(CombatError::EmptyVolley);
    }
    if receipts.len() > MAX_VOLLEY_STRIKES {
        return Err(CombatError::TooManyStrikes);
    }

    let mut target_parts = Vec::new();
    for part in target_body.parts.iter().filter(|part| !part.severed) {
        let bounds = world_bounds(target_body, part.id, target_pose)?;
        target_parts.push((part.id, bounds));
    }
    let mut strikes = Vec::with_capacity(receipts.len());
    let mut harm = 0u16;
    let mut severed_parts = Vec::new();
    let mut sources = BTreeSet::new();
    let mut ray_budget = RayBudget(MAX_VOLLEY_RAY_CELLS);
    for receipt in receipts {
        if !sources.insert(receipt.binding.part) {
            return Err(CombatError::DuplicateSource(receipt.binding.part));
        }
        if receipt.charge == 0 {
            return Err(CombatError::EmptyCharge(receipt.binding.part));
        }
        if receipt.binding.revision != actor_revision {
            return Err(CombatError::StaleSource {
                part: receipt.binding.part,
                expected: receipt.binding.revision,
                actual: actor_revision,
            });
        }
        let Some(part) = actor_body.part(receipt.binding.part) else {
            return Err(CombatError::InvalidSourcePart(receipt.binding.part));
        };
        if part.severed {
            return Err(CombatError::InvalidSourcePart(receipt.binding.part));
        }
        let source = world_bounds(actor_body, receipt.binding.part, actor_pose)?;
        let sweep = swept(source, receipt.direction, reach(receipt.charge, rules)?)?;
        let mut visible_parts = Vec::new();
        for &(part, bounds) in &target_parts {
            if intersects(sweep, bounds)
                && sees(ground, center(source), center(bounds), &mut ray_budget)?
            {
                visible_parts.push((part, bounds));
            }
        }
        let contact = visible_parts
            .into_iter()
            .min_by_key(|(part, bounds)| {
                (entry_distance(source, *bounds, receipt.direction), *part)
            });
        let outcome = if let Some((part, bounds)) = contact {
            let quality = overlap_quality(sweep, bounds);
            let strike_harm = rules
                .base_harm
                .saturating_add(receipt.charge.min(u64::from(u16::MAX)) as u16)
                .saturating_add(quality);
            harm = harm.saturating_add(strike_harm);
            if part != target_body.root
                && strike_harm >= rules.sever_threshold
                && !severed_parts.contains(&part)
            {
                severed_parts.push(part);
            }
            StrikeOutcome::Hit {
                part,
                quality,
                harm: strike_harm,
            }
        } else {
            StrikeOutcome::Miss
        };
        strikes.push(ResolvedStrike {
            source: receipt.binding.part,
            outcome,
        });
    }
    severed_parts.sort();
    Ok(VolleyResolution {
        strikes,
        harm,
        severed_parts,
    })
}

fn world_bounds(
    document: &BodyDocument,
    id: PartId,
    pose: MotionPose,
) -> Result<FixedAabb, CombatError> {
    let local = part_bounds(document, id).ok_or(CombatError::GeometryOverflow)?;
    let scale = i128::from(MOTION_SCALE);
    let anchor = [
        i128::from(pose.position[0]) - scale / 2,
        i128::from(pose.position[1]),
        i128::from(pose.position[2]) - scale / 2,
    ];
    let bounds = FixedAabb {
        min: std::array::from_fn(|axis| anchor[axis] + i128::from(local.min[axis]) * scale),
        max: std::array::from_fn(|axis| anchor[axis] + i128::from(local.max[axis]) * scale),
    };
    let minimum = i128::from(i32::MIN) * scale;
    let maximum = (i128::from(i32::MAX) + 1) * scale;
    if (0..3).any(|axis| bounds.min[axis] < minimum || bounds.max[axis] > maximum) {
        return Err(CombatError::GeometryOverflow);
    }
    Ok(bounds)
}

fn swept(
    mut bounds: FixedAabb,
    direction: Direction,
    amount: i32,
) -> Result<FixedAabb, CombatError> {
    let (axis, delta) = match direction {
        Direction::Forward => (2, amount),
        Direction::Backward => (2, -amount),
        Direction::Left => (0, -amount),
        Direction::Right => (0, amount),
        Direction::Up => (1, amount),
        Direction::Down => (1, -amount),
    };
    let delta = i128::from(delta) * i128::from(MOTION_SCALE);
    if delta < 0 {
        bounds.min[axis] = bounds.min[axis]
            .checked_add(delta)
            .ok_or(CombatError::GeometryOverflow)?;
    } else {
        bounds.max[axis] = bounds.max[axis]
            .checked_add(delta)
            .ok_or(CombatError::GeometryOverflow)?;
    }
    Ok(bounds)
}

fn intersects(a: FixedAabb, b: FixedAabb) -> bool {
    (0..3).all(|axis| a.min[axis] < b.max[axis] && a.max[axis] > b.min[axis])
}

fn overlap_quality(a: FixedAabb, b: FixedAabb) -> u16 {
    let overlap = (0..3)
        .map(|axis| a.max[axis].min(b.max[axis]) - a.min[axis].max(b.min[axis]))
        .min()
        .unwrap_or(0)
        .max(0);
    (overlap / i128::from(MOTION_SCALE)).min(i128::from(u16::MAX)) as u16
}

fn entry_distance(source: FixedAabb, target: FixedAabb, direction: Direction) -> i128 {
    match direction {
        Direction::Forward => target.min[2].saturating_sub(source.max[2]),
        Direction::Backward => source.min[2].saturating_sub(target.max[2]),
        Direction::Left => source.min[0].saturating_sub(target.max[0]),
        Direction::Right => target.min[0].saturating_sub(source.max[0]),
        Direction::Up => target.min[1].saturating_sub(source.max[1]),
        Direction::Down => source.min[1].saturating_sub(target.max[1]),
    }
}

fn center(bounds: FixedAabb) -> [i128; 3] {
    std::array::from_fn(|axis| (bounds.min[axis] + bounds.max[axis]) / 2)
}

/// Exact fixed-point voxel walk. Endpoint cells are deliberately excluded,
/// matching Ground::sees while retaining fractional endpoint information. A
/// tied boundary crossing advances all tied axes, selecting the diagonal cell
/// entered by the ray's half-open voxel ownership rule.
fn sees(
    ground: &Ground,
    from: [i128; 3],
    to: [i128; 3],
    budget: &mut RayBudget,
) -> Result<bool, CombatError> {
    traverse(from, to, budget, |at| ground.solid(at))
}

fn traverse(
    from: [i128; 3],
    to: [i128; 3],
    budget: &mut RayBudget,
    mut solid: impl FnMut([i32; 3]) -> bool,
) -> Result<bool, CombatError> {
    let scale = i128::from(MOTION_SCALE);
    let mut cell: [i128; 3] = std::array::from_fn(|axis| from[axis].div_euclid(scale));
    let end: [i128; 3] = std::array::from_fn(|axis| to[axis].div_euclid(scale));
    let delta: [i128; 3] = std::array::from_fn(|axis| to[axis] - from[axis]);
    let crossings = (0..3)
        .map(|axis| (end[axis] - cell[axis]).abs())
        .sum::<i128>();
    if crossings > MAX_RAY_CELLS || crossings > budget.0 {
        return Err(CombatError::GeometryOverflow);
    }
    budget.0 -= crossings;
    let mut step = [0i128; 3];
    let mut numerator = [0i128; 3];
    let mut denominator = [0i128; 3];
    for axis in 0..3 {
        if delta[axis] == 0 {
            continue;
        }
        step[axis] = delta[axis].signum();
        denominator[axis] = delta[axis].abs();
        let boundary = if step[axis] > 0 {
            (cell[axis] + 1) * scale
        } else {
            cell[axis] * scale
        };
        numerator[axis] = (boundary - from[axis]).abs();
    }
    for _ in 0..crossings {
        let axis = (0..3)
            .filter(|axis| denominator[*axis] != 0)
            .min_by(|left, right| {
                (numerator[*left] * denominator[*right])
                    .cmp(&(numerator[*right] * denominator[*left]))
            })
            .ok_or(CombatError::GeometryOverflow)?;
        let crosses = (0..3)
            .filter(|other| {
                denominator[*other] != 0
                    && numerator[*other] * denominator[axis]
                        == numerator[axis] * denominator[*other]
            })
            .collect::<Vec<_>>();
        let at_end = numerator[axis] >= denominator[axis];
        for axis in crosses {
            cell[axis] += step[axis];
            numerator[axis] += scale;
        }
        if at_end || cell == end {
            return Ok(true);
        }
        let at = [
            i32::try_from(cell[0]).map_err(|_| CombatError::GeometryOverflow)?,
            i32::try_from(cell[1]).map_err(|_| CombatError::GeometryOverflow)?,
            i32::try_from(cell[2]).map_err(|_| CombatError::GeometryOverflow)?,
        ];
        if solid(at) {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> crate::World {
        crate::World::generate(7, crate::WorldConfig::default()).unwrap()
    }

    fn q(cell: [i32; 3], offset: [i128; 3]) -> [i128; 3] {
        let scale = i128::from(MOTION_SCALE);
        std::array::from_fn(|axis| i128::from(cell[axis]) * scale + offset[axis])
    }

    #[test]
    fn exact_ray_handles_negative_direction_zero_length_and_boundary_ties() {
        let world = world();
        let high = 100_000;
        let centre = [i128::from(MOTION_SCALE) / 2; 3];
        let mut budget = RayBudget(MAX_VOLLEY_RAY_CELLS);
        assert!(sees(
            world.ground(),
            q([3, high, 3], centre),
            q([-3, high, 3], centre),
            &mut budget,
        )
        .unwrap());
        assert!(sees(
            world.ground(),
            q([0, high, 0], centre),
            q([0, high, 0], centre),
            &mut budget,
        )
        .unwrap());
        assert!(sees(
            world.ground(),
            q([0, high, 0], [0, 0, 0]),
            q([2, high, 2], [0, 0, 0]),
            &mut budget,
        )
        .unwrap());
    }

    #[test]
    fn fractional_endpoints_do_not_truncate_and_budget_rejects_excess() {
        let world = world();
        let ground = world.ground();
        let y = ground.surface(0, 0).unwrap_or(1);
        let centre = [i128::from(MOTION_SCALE) / 2; 3];
        let mut budget = RayBudget(MAX_VOLLEY_RAY_CELLS);
        assert!(!sees(
            ground,
            q([-1, y, 0], centre),
            q([1, y, 0], centre),
            &mut budget,
        )
        .unwrap());

        let high = 100_000;
        let mut budget = RayBudget(MAX_VOLLEY_RAY_CELLS);
        assert!(sees(
            ground,
            q([0, high, 0], [1, centre[1], centre[2]]),
            q([0, high, 0], [i128::from(MOTION_SCALE) - 1, centre[1], centre[2]]),
            &mut budget,
        )
        .unwrap());
        assert_eq!(
            sees(
                ground,
                q([0, high, 0], centre),
                q([MAX_RAY_CELLS as i32 + 1, high, 0], centre),
                &mut RayBudget(MAX_VOLLEY_RAY_CELLS),
            ),
            Err(CombatError::GeometryOverflow),
        );
    }

    #[test]
    fn fractional_ray_can_hit_a_voxel_the_legacy_integer_ray_skips() {
        let scale = i128::from(MOTION_SCALE);
        let half = scale / 2;
        let blocked = [0, 0, 1];
        let from = q([-1, 0, 0], [half, half, scale - 1]);
        let to = q([1, 0, 1], [half, half, 1]);
        let mut budget = RayBudget(MAX_VOLLEY_RAY_CELLS);
        assert!(!traverse(from, to, &mut budget, |at| at == blocked).unwrap());
        // Ground::sees receives only these old cell centres. Its fixed
        // half-stride samples never enter [0, 0, 1].
        let legacy_samples = [[-1, 0, 0], [0, 0, 0], [0, 0, 0]];
        assert!(!legacy_samples.contains(&blocked));
    }

    #[test]
    fn boundary_departure_tests_the_neighbor_entered_at_zero() {
        let scale = i128::from(MOTION_SCALE);
        let from = [0, scale / 2, scale / 2];
        let to = [-scale - scale / 2, scale / 2, scale / 2];
        assert!(!traverse(from, to, &mut RayBudget(MAX_VOLLEY_RAY_CELLS), |at| {
            at == [-1, 0, 0]
        })
        .unwrap());
    }
}
