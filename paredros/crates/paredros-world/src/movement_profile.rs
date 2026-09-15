// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit anatomy roles used only by revision-two locomotion.

use std::collections::BTreeSet;

use isometer_core::PartId;
use paredros_identity::BodyRevisionId;
use serde::{Deserialize, Serialize};

use crate::{AnatomyRecord, MOTION_SCALE, MotionError, MotionRules, part_bounds};

pub const MOVEMENT_PROFILE_REVISION: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MotionEnvelope {
    /// The authored anatomy address that owns this explicit envelope.
    pub anchor: PartId,
    pub half_width: i64,
    pub height: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportBand {
    /// Body-document coordinates. A support contributes on positive overlap.
    pub min_y: i32,
    pub max_y: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovementProfile {
    pub revision: u32,
    pub source_revision: BodyRevisionId,
    pub envelope: MotionEnvelope,
    pub supports: Vec<PartId>,
    pub support_band: SupportBand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementProjection {
    pub source_revision: BodyRevisionId,
    pub current_revision: BodyRevisionId,
    pub envelope: MotionEnvelope,
    pub declared_supports: usize,
    pub active_supports: Vec<PartId>,
    pub speed: i64,
}

impl MovementProfile {
    pub fn validate_at(&self, record: &AnatomyRecord) -> Result<(), MotionError> {
        if self.revision != MOVEMENT_PROFILE_REVISION
            || self.source_revision != record.revision
            || self.supports.is_empty()
            || self.supports.len() > crate::MAX_ANATOMY_PARTS
            || self.support_band.min_y >= self.support_band.max_y
        {
            return Err(MotionError::InvalidProfile);
        }
        self.validate_envelope()?;
        self.validate_addresses(record, true)
    }

    pub(crate) fn project(
        &self,
        record: &AnatomyRecord,
        requested_speed: i64,
    ) -> Result<MovementProjection, MotionError> {
        if self.revision != MOVEMENT_PROFILE_REVISION || record.revision < self.source_revision {
            return Err(MotionError::InvalidProfile);
        }
        self.validate_envelope()?;
        self.validate_addresses(record, false)?;
        let active_supports: Vec<_> = self
            .supports
            .iter()
            .copied()
            .filter(|part| support_intersects(record, *part, self.support_band))
            .collect();
        let active = i64::try_from(active_supports.len()).map_err(|_| MotionError::Overflow)?;
        let declared = i64::try_from(self.supports.len()).map_err(|_| MotionError::Overflow)?;
        Ok(MovementProjection {
            source_revision: self.source_revision,
            current_revision: record.revision,
            envelope: self.envelope,
            declared_supports: self.supports.len(),
            active_supports,
            speed: if active == 0 {
                0
            } else {
                requested_speed
                    .checked_mul(active)
                    .ok_or(MotionError::Overflow)?
                    .checked_div(declared)
                    .ok_or(MotionError::Overflow)?
                    .max(1)
            },
        })
    }

    fn validate_envelope(&self) -> Result<(), MotionError> {
        MotionRules {
            revision: MOVEMENT_PROFILE_REVISION,
            speed: MOTION_SCALE,
            gravity: MOTION_SCALE,
            terminal_speed: MOTION_SCALE,
            half_width: self.envelope.half_width,
            height: self.envelope.height,
        }
        .validate()
    }

    fn validate_addresses(
        &self,
        record: &AnatomyRecord,
        require_intact: bool,
    ) -> Result<(), MotionError> {
        let mut supports = BTreeSet::new();
        for part in &self.supports {
            if !supports.insert(*part) {
                return Err(MotionError::InvalidProfile);
            }
            let found = record
                .document
                .part(*part)
                .ok_or(MotionError::InvalidProfile)?;
            if require_intact && found.severed {
                return Err(MotionError::InvalidProfile);
            }
        }
        let anchor = record
            .document
            .part(self.envelope.anchor)
            .ok_or(MotionError::MissingEnvelopeAnchor(self.envelope.anchor))?;
        if anchor.severed {
            return Err(MotionError::MissingEnvelopeAnchor(self.envelope.anchor));
        }
        Ok(())
    }
}

impl MovementProjection {
    pub(crate) fn rules_for(&self, mut rules: MotionRules) -> Result<MotionRules, MotionError> {
        if rules.revision != MOVEMENT_PROFILE_REVISION {
            return Err(MotionError::InvalidRules);
        }
        rules.half_width = self.envelope.half_width;
        rules.height = self.envelope.height;
        rules.speed = self.speed;
        rules.validate_projection()?;
        Ok(rules)
    }
}

fn support_intersects(record: &AnatomyRecord, part: PartId, band: SupportBand) -> bool {
    let Some(found) = record.document.part(part) else {
        return false;
    };
    !found.severed
        && part_bounds(&record.document, part)
            .is_some_and(|bounds| bounds.min[1] < band.max_y && band.min_y < bounds.max[1])
}
