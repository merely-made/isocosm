// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Immutable, revision-addressed body snapshots admitted for Eponym.
//!
//! An admitted record is evidence about one body revision, not body authority.
//! Injury can make it stale; a later transition must admit or reconcile the
//! replacement revision. Admission deliberately has no UI, equipment, or body
//! mutation powers.

use std::collections::{BTreeMap, BTreeSet};

use crate::identity::{BodyRevisionId, SubjectId};
use isocosm::lineage::LineageBody as BodyDocument;
use isometer_core::{Aabb, PartId, Yaw};
use serde::{Deserialize, Serialize};

/// A deliberately small bound: the authored three-lives fixture has seven
/// parts, while 256 keeps validation and every parent walk bounded.
pub const MAX_ANATOMY_PARTS: usize = 256;
/// Per-field coordinate bound for an externally supplied body document.
pub const MAX_ANATOMY_COORDINATE: i32 = 1_000_000;
/// A root-to-leaf pivot may accumulate one bounded offset per admitted part.
pub const MAX_ANATOMY_WORLD_COORDINATE: i32 = 256_000_000;

/// Derive a part's body-space bounds through the core's pivot and attachment
/// arithmetic. Consumers may translate this into an authoritative world pose.
pub fn part_bounds(document: &isometer_core::BodyDocument, id: PartId) -> Option<Aabb> {
    let part = document.part(id)?;
    let extent = [
        part.half_extent[0].checked_mul(2)?,
        part.half_extent[1].checked_mul(2)?,
        part.half_extent[2].checked_mul(2)?,
    ];
    let mut corners = (0..8).map(|mask| {
        document.place(
            id,
            [
                if mask & 1 == 0 { 0 } else { extent[0] },
                if mask & 2 == 0 { 0 } else { extent[1] },
                if mask & 4 == 0 { 0 } else { extent[2] },
            ],
        )
    });
    let first = corners.next()??;
    let mut min = first;
    let mut max = first;
    for corner in corners {
        let corner = corner?;
        for axis in 0..3 {
            min[axis] = min[axis].min(corner[axis]);
            max[axis] = max[axis].max(corner[axis]);
        }
    }
    Some(Aabb { min, max })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnatomyRecord {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub document: BodyDocument,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anatomies {
    records: BTreeMap<SubjectId, AnatomyRecord>,
}

/// Rejections are intentionally coarse: callers can choose their presentation
/// without depending on a deserialization detail of Mesocosm's body types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnatomyError {
    EmptyDocument,
    TooManyParts,
    MissingRoot,
    RootHasParent,
    NonContiguousPartIds,
    MissingAttachment,
    MissingParent,
    CycleOrDisconnected,
    SeveredRoot,
    LivingChildOfSeveredParent,
    NonPositiveHalfExtent,
    PivotOutsidePart,
    CoordinateOutOfBounds,
    TransformOverflow,
    MassOverflow,
    DuplicateSubject,
    Missing(SubjectId),
    DuplicatePart(PartId),
    MissingPart(PartId),
    RootPart(PartId),
    SeveredPart(PartId),
    StaleRevision {
        known: BodyRevisionId,
        current: BodyRevisionId,
    },
    RevisionNotAdvanced {
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
    },
}

impl Anatomies {
    pub fn get(&self, subject: SubjectId) -> Option<&AnatomyRecord> {
        self.records.get(&subject)
    }

    /// Adds exactly one validated snapshot for a subject. Validation finishes
    /// before the map changes, so a rejected document cannot leave a partial
    /// record behind.
    pub(crate) fn admit(
        &mut self,
        subject: SubjectId,
        revision: BodyRevisionId,
        document: BodyDocument,
    ) -> Result<(), AnatomyError> {
        if self.records.contains_key(&subject) {
            return Err(AnatomyError::DuplicateSubject);
        }
        validate(&document)?;
        self.records.insert(
            subject,
            AnatomyRecord {
                subject,
                revision,
                document,
            },
        );
        Ok(())
    }

    /// Carries an admitted anatomy forward through a summary-body injury.
    ///
    /// The detailed record is a tombstoned copy of the preceding revision:
    /// this does not manufacture a location, volume, attachment, or origin
    /// for any injury the summary did not identify.
    pub(crate) fn reconcile(
        &mut self,
        subject: SubjectId,
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
        severed_parts: &[PartId],
    ) -> Result<(), AnatomyError> {
        let stored = self
            .records
            .get(&subject)
            .ok_or(AnatomyError::Missing(subject))?;
        if stored.revision != from_revision {
            return Err(AnatomyError::StaleRevision {
                known: from_revision,
                current: stored.revision,
            });
        }
        if revision <= from_revision {
            return Err(AnatomyError::RevisionNotAdvanced {
                from_revision,
                revision,
            });
        }
        if severed_parts.len() > MAX_ANATOMY_PARTS {
            return Err(AnatomyError::TooManyParts);
        }

        let mut requested = BTreeSet::new();
        for &part in severed_parts {
            if !requested.insert(part) {
                return Err(AnatomyError::DuplicatePart(part));
            }
            let found = stored
                .document
                .part(part)
                .ok_or(AnatomyError::MissingPart(part))?;
            if part == stored.document.root {
                return Err(AnatomyError::RootPart(part));
            }
            if found.severed {
                return Err(AnatomyError::SeveredPart(part));
            }
        }

        let mut replacement = stored.clone();
        for &part in severed_parts {
            replacement.document.sever(part);
        }
        validate(&replacement.document)?;
        replacement.revision = revision;
        self.records.insert(subject, replacement);
        Ok(())
    }
}

fn validate(document: &BodyDocument) -> Result<(), AnatomyError> {
    if document.parts.is_empty() {
        return Err(AnatomyError::EmptyDocument);
    }
    if document.parts.len() > MAX_ANATOMY_PARTS {
        return Err(AnatomyError::TooManyParts);
    }
    if document.part(document.root).is_none() {
        return Err(AnatomyError::MissingRoot);
    }
    for (index, part) in document.parts.iter().enumerate() {
        if part.id != PartId(index as u32) {
            return Err(AnatomyError::NonContiguousPartIds);
        }
    }
    let root = document.part(document.root).expect("root checked above");
    if root.attachment.is_some() {
        return Err(AnatomyError::RootHasParent);
    }
    if root.severed {
        return Err(AnatomyError::SeveredRoot);
    }
    for part in &document.parts {
        if part.id != document.root && part.attachment.is_none() {
            return Err(AnatomyError::MissingAttachment);
        }
        if let Some(attachment) = part.attachment
            && document.part(attachment.parent).is_none()
        {
            return Err(AnatomyError::MissingParent);
        }
    }
    for part in &document.parts {
        let mut cursor = part.id;
        for _ in 0..document.parts.len() {
            if cursor == document.root {
                break;
            }
            cursor = document
                .part(cursor)
                .and_then(|found| found.attachment)
                .map(|attachment| attachment.parent)
                .ok_or(AnatomyError::CycleOrDisconnected)?;
        }
        if cursor != document.root {
            return Err(AnatomyError::CycleOrDisconnected);
        }
    }

    let mut total_mass_mg = 0u64;
    for part in &document.parts {
        total_mass_mg = total_mass_mg
            .checked_add(document.mass_mg(part.id))
            .ok_or(AnatomyError::MassOverflow)?;
        if let Some(attachment) = part.attachment
            && !part.severed
            && document
                .part(attachment.parent)
                .is_some_and(|parent| parent.severed)
        {
            return Err(AnatomyError::LivingChildOfSeveredParent);
        }
        for axis in 0..3 {
            let half = part.half_extent[axis];
            if half < 1 {
                return Err(AnatomyError::NonPositiveHalfExtent);
            }
            if half > MAX_ANATOMY_COORDINATE {
                return Err(AnatomyError::CoordinateOutOfBounds);
            }
            let maximum_pivot = half.checked_mul(2).ok_or(AnatomyError::TransformOverflow)?;
            if !(0..=maximum_pivot).contains(&part.pivot[axis]) {
                return Err(AnatomyError::PivotOutsidePart);
            }
        }
        if let Some(attachment) = part.attachment {
            for offset in attachment.offset {
                if offset.unsigned_abs() > MAX_ANATOMY_COORDINATE as u32 {
                    return Err(AnatomyError::CoordinateOutOfBounds);
                }
            }
        }
        checked_world_pivot(document, part.id)?;
    }
    Ok(())
}

fn checked_world_pivot(
    document: &isometer_core::BodyDocument,
    id: PartId,
) -> Result<[i32; 3], AnatomyError> {
    let mut offset = [0; 3];
    let mut cursor = id;
    for _ in 0..document.parts.len() {
        let part = document.part(cursor).ok_or(AnatomyError::MissingParent)?;
        let Some(attachment) = part.attachment else {
            return Ok(offset);
        };
        let rotated = checked_rotate(attachment.yaw, offset)?;
        for axis in 0..3 {
            offset[axis] = rotated[axis]
                .checked_add(attachment.offset[axis])
                .ok_or(AnatomyError::TransformOverflow)?;
            if offset[axis].unsigned_abs() > MAX_ANATOMY_WORLD_COORDINATE as u32 {
                return Err(AnatomyError::CoordinateOutOfBounds);
            }
        }
        cursor = attachment.parent;
    }
    Err(AnatomyError::CycleOrDisconnected)
}

fn checked_rotate(yaw: Yaw, [x, y, z]: [i32; 3]) -> Result<[i32; 3], AnatomyError> {
    let neg = |value: i32| value.checked_neg().ok_or(AnatomyError::TransformOverflow);
    match yaw {
        Yaw::Zero => Ok([x, y, z]),
        Yaw::Quarter => Ok([z, y, neg(x)?]),
        Yaw::Half => Ok([neg(x)?, y, neg(z)?]),
        Yaw::ThreeQuarter => Ok([neg(z)?, y, x]),
    }
}
