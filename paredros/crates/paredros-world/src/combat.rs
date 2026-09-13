// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Deterministic, anatomy-shaped volley adjudication.
//!
//! A recorded strike receipt is a bounded simulation input. This module
//! verifies its source against the current body and computes target contact
//! from admitted anatomy, current positions, and exact Ground visibility.

use std::collections::BTreeSet;

use mesocosm_core::places::Ground;
use mesocosm_core::{Aabb, BodyDocument, PartId};
use paredros_identity::{BodyRevisionId, SubjectId};
use serde::{Deserialize, Serialize};

use crate::part_bounds;
use crate::timed_action::{Direction, StrikeReceipt};

pub const COMBAT_RULES_REVISION: u32 = 1;
pub const MAX_VOLLEY_STRIKES: usize = 32;
pub const MAX_COMBAT_REACH: i32 = 64;

/// Product policy recorded with each accepted volley. It is deliberately
/// small and is not a universal combat-system profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatRules {
    pub revision: u32,
    pub max_reach: i32,
    pub charge_per_reach: u64,
    pub base_harm: u16,
    pub sever_threshold: u16,
}

impl Default for CombatRules {
    fn default() -> Self {
        Self {
            revision: COMBAT_RULES_REVISION,
            max_reach: 8,
            charge_per_reach: 2,
            base_harm: 8,
            sever_threshold: 24,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrikeOutcome {
    Miss,
    Hit {
        part: PartId,
        quality: u16,
        harm: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedStrike {
    pub source: PartId,
    pub outcome: StrikeOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VolleyResolution {
    pub strikes: Vec<ResolvedStrike>,
    pub harm: u16,
    pub severed_parts: Vec<PartId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatError {
    InvalidRules,
    SameSubject(SubjectId),
    EmptyVolley,
    TooManyStrikes,
    DuplicateSource(PartId),
    EmptyCharge(PartId),
    InvalidSourcePart(PartId),
    StaleSource {
        part: PartId,
        expected: BodyRevisionId,
        actual: BodyRevisionId,
    },
    GeometryOverflow,
}

impl CombatRules {
    pub fn validate(self) -> Result<(), CombatError> {
        if self.revision != COMBAT_RULES_REVISION
            || self.max_reach < 1
            || self.max_reach > MAX_COMBAT_REACH
            || self.charge_per_reach == 0
            || self.base_harm == 0
            || self.sever_threshold == 0
        {
            return Err(CombatError::InvalidRules);
        }
        Ok(())
    }
}

pub(crate) fn resolve(
    actor: SubjectId,
    actor_revision: BodyRevisionId,
    actor_body: &BodyDocument,
    actor_at: [i32; 3],
    target: SubjectId,
    target_body: &BodyDocument,
    target_at: [i32; 3],
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
        let bounds =
            world_bounds(target_body, part.id, target_at).ok_or(CombatError::GeometryOverflow)?;
        target_parts.push((part.id, bounds));
    }
    let mut strikes = Vec::with_capacity(receipts.len());
    let mut harm = 0u16;
    let mut severed_parts = Vec::new();
    let mut sources = BTreeSet::new();
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
        let source = world_bounds(actor_body, receipt.binding.part, actor_at)
            .ok_or(CombatError::GeometryOverflow)?;
        let sweep = swept(source, receipt.direction, reach(receipt.charge, rules)?)?;
        let contact = target_parts
            .iter()
            .filter_map(|(part, bounds)| {
                (intersects(sweep, *bounds) && ground.sees(center(source), center(*bounds)))
                    .then_some((*part, *bounds))
            })
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

fn world_bounds(document: &BodyDocument, id: PartId, at: [i32; 3]) -> Option<Aabb> {
    let local = part_bounds(document, id)?;
    let min = [
        local.min[0].checked_add(at[0])?,
        local.min[1].checked_add(at[1])?,
        local.min[2].checked_add(at[2])?,
    ];
    let max = [
        local.max[0].checked_add(at[0])?,
        local.max[1].checked_add(at[1])?,
        local.max[2].checked_add(at[2])?,
    ];
    Some(Aabb { min, max })
}

fn reach(charge: u64, rules: CombatRules) -> Result<i32, CombatError> {
    let extra = charge / rules.charge_per_reach;
    let extra = i32::try_from(extra).unwrap_or(i32::MAX);
    Ok(1_i32.saturating_add(extra).min(rules.max_reach))
}

fn swept(mut bounds: Aabb, direction: Direction, amount: i32) -> Result<Aabb, CombatError> {
    let axis_delta = match direction {
        Direction::Forward => (2, amount),
        Direction::Backward => (2, -amount),
        Direction::Left => (0, -amount),
        Direction::Right => (0, amount),
        Direction::Up => (1, amount),
        Direction::Down => (1, -amount),
    };
    let (axis, delta) = axis_delta;
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

fn intersects(a: Aabb, b: Aabb) -> bool {
    (0..3).all(|axis| a.min[axis] < b.max[axis] && a.max[axis] > b.min[axis])
}

fn overlap_quality(a: Aabb, b: Aabb) -> u16 {
    let overlap = (0..3)
        .map(|axis| a.max[axis].min(b.max[axis]) - a.min[axis].max(b.min[axis]))
        .min()
        .unwrap_or(0)
        .max(0);
    overlap.min(i32::from(u16::MAX)) as u16
}

fn entry_distance(source: Aabb, target: Aabb, direction: Direction) -> i32 {
    match direction {
        Direction::Forward => target.min[2].saturating_sub(source.max[2]),
        Direction::Backward => source.min[2].saturating_sub(target.max[2]),
        Direction::Left => source.min[0].saturating_sub(target.max[0]),
        Direction::Right => target.min[0].saturating_sub(source.max[0]),
        Direction::Up => target.min[1].saturating_sub(source.max[1]),
        Direction::Down => source.min[1].saturating_sub(target.max[1]),
    }
}

fn center(bounds: Aabb) -> [i32; 3] {
    std::array::from_fn(|axis| {
        let sum = i64::from(bounds.min[axis]) + i64::from(bounds.max[axis]);
        (sum / 2) as i32
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_reject_unknown_or_unbounded_policy() {
        let mut rules = CombatRules::default();
        rules.revision += 1;
        assert_eq!(rules.validate(), Err(CombatError::InvalidRules));
        rules = CombatRules::default();
        rules.max_reach = MAX_COMBAT_REACH + 1;
        assert_eq!(rules.validate(), Err(CombatError::InvalidRules));
    }

    #[test]
    fn sweep_expands_only_in_the_selected_direction() {
        let bounds = Aabb {
            min: [2, 3, 4],
            max: [4, 5, 6],
        };
        assert_eq!(
            swept(bounds, Direction::Left, 3),
            Ok(Aabb {
                min: [-1, 3, 4],
                max: [4, 5, 6]
            })
        );
        assert_eq!(
            swept(bounds, Direction::Up, 3),
            Ok(Aabb {
                min: [2, 3, 4],
                max: [4, 8, 6]
            })
        );
    }

    #[test]
    fn outer_surface_wins_over_a_nearer_internal_part_center() {
        use crate::timed_action::LimbBinding;
        use mesocosm_core::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};
        let actor = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 10, [1; 3]);
        let mut target = BodyDocument::new(SpeciesId(2), VolumeRef::from_tag(2), 100, [10, 1, 1]);
        target
            .attach(
                VolumeRef::from_tag(3),
                1,
                [1; 3],
                Attachment {
                    parent: target.root,
                    offset: [-5, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let world = crate::World::generate(7, crate::WorldConfig::default()).unwrap();
        // Place above terrain to isolate part ordering. The small internal
        // part has a closer centre, but the enclosing body's surface is first.
        let result = resolve(
            SubjectId(1),
            BodyRevisionId(0),
            &actor,
            [0, 1000, 0],
            SubjectId(2),
            &target,
            [30, 1000, 0],
            world.ground(),
            &[StrikeReceipt {
                binding: LimbBinding {
                    part: actor.root,
                    revision: BodyRevisionId(0),
                },
                direction: Direction::Right,
                charge: 64,
            }],
            CombatRules {
                max_reach: 64,
                charge_per_reach: 1,
                sever_threshold: 1,
                ..CombatRules::default()
            },
        )
        .unwrap();
        assert!(matches!(
            result.strikes[0].outcome,
            StrikeOutcome::Hit {
                part: PartId(0),
                ..
            }
        ));
        assert!(
            result.severed_parts.is_empty(),
            "a root hit cannot sever the root"
        );
    }
}
