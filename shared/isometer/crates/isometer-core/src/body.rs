// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The body graph: parts, attachment frames, and per-part provenance.
//!
//! Coordinates are voxel units and masses are milligrams, both integers.
//! Rotations are quarter turns. Nothing here is a float, so a body's derived
//! quantities are bit-identical on every platform, and the float physics a
//! host runs sits outside this boundary.
//!
//! Coordinates are three-dimensional even when a host presents two: a 2.5D
//! projection constrains an axis rather than changing the document.

use serde::{Deserialize, Serialize};

use wing_formats::PartOrigin;

use crate::plan::BodyPlan;

/// Stable index into [`BodyDocument::parts`]. Never reused within a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartId(pub u32);

/// Identifies a lineage. Provenance records which one a part came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpeciesId(pub u32);

/// Content address of the voxel volume a projection should draw. The core
/// never reads volume contents; it only carries the reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VolumeRef(pub [u8; 32]);

impl VolumeRef {
    /// Test and fixture helper: a recognisable address from a small number.
    pub fn from_tag(tag: u8) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0] = tag;
        Self(bytes)
    }
}

/// Quarter turns about the vertical axis. Enough to prove attachment while
/// keeping every transform exact in integers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Yaw {
    #[default]
    Zero,
    Quarter,
    Half,
    ThreeQuarter,
}

impl Yaw {
    /// Turns a vector by this many quarter turns about the vertical axis.
    ///
    /// Public since P3: a harvested branch has to be able to work out the box
    /// it occupies before it is attached to anything, which is the same
    /// arithmetic `world_pivot` does and must not be a second copy of it.
    pub fn rotate(self, v: [i32; 3]) -> [i32; 3] {
        let [x, y, z] = v;
        match self {
            Yaw::Zero => [x, y, z],
            Yaw::Quarter => [z, y, -x],
            Yaw::Half => [-x, y, -z],
            Yaw::ThreeQuarter => [-z, y, x],
        }
    }

    /// This turn applied on top of an inner one.
    pub fn compose(self, inner: Yaw) -> Yaw {
        let steps = (self.steps() + inner.steps()) % 4;
        match steps {
            0 => Yaw::Zero,
            1 => Yaw::Quarter,
            2 => Yaw::Half,
            _ => Yaw::ThreeQuarter,
        }
    }

    fn steps(self) -> u8 {
        match self {
            Yaw::Zero => 0,
            Yaw::Quarter => 1,
            Yaw::Half => 2,
            Yaw::ThreeQuarter => 3,
        }
    }
}

/// Where a part used to be before it became part of this body. This is the
/// keystone record: every part carries the fact that it was once somebody.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// Present when the lineage was founded.
    Founding,
    /// Taken from another organism by incorporation.
    Incorporated {
        from_species: SpeciesId,
        /// The part's identity in the body it came from.
        from_part: PartId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: Origin,
    /// The epoch during which this part joined the body.
    pub epoch: u64,
}

impl Provenance {
    pub fn founding() -> Self {
        Self {
            origin: Origin::Founding,
            epoch: 0,
        }
    }
}

/// How a part is fixed to its parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub parent: PartId,
    /// Displacement from the parent's pivot to this part's pivot, in the
    /// parent's frame and before the parent's own rotation.
    ///
    /// Pivot-to-pivot, which is what makes flush placement symmetric: a part
    /// sits against its parent's `+x` face at `+(parent_half + own_half)` and
    /// against `-x` at the negation of the same number.
    pub offset: [i32; 3],
    pub yaw: Yaw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub id: PartId,
    pub volume: VolumeRef,
    pub mass_mg: u64,
    /// Half-extent in voxel units, so an extent can be derived without
    /// resolving the volume.
    pub half_extent: [i32; 3],
    /// The point, in this part's own voxel space, that an attachment offset is
    /// measured to and that a rotation turns about.
    ///
    /// Defaults to the part's centre. Before pivots existed a part's origin
    /// was its lowest corner, which caused four separate defects: limbs that
    /// floated instead of joining, flush placement that needed to know a
    /// part's size and was asymmetric between faces, a centre of mass that
    /// averaged corners, and an AABB that treated a corner as a centre.
    pub pivot: [i32; 3],
    /// `None` only for the root.
    pub attachment: Option<Attachment>,
    pub provenance: Provenance,
    /// Lost, along with everything that hung off it. Tombstoned rather than
    /// removed so `PartId` stays an index and the injury stays on the record.
    /// See [`crate::anatomy`].
    #[serde(default)]
    pub severed: bool,
}

/// An axis-aligned box in body space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl Aabb {
    fn around(centre: [i32; 3], half: [i32; 3]) -> Self {
        Self {
            min: [
                centre[0] - half[0],
                centre[1] - half[1],
                centre[2] - half[2],
            ],
            max: [
                centre[0] + half[0],
                centre[1] + half[1],
                centre[2] + half[2],
            ],
        }
    }

    fn union(self, other: Aabb) -> Self {
        Self {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn extent(&self) -> [i32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
}

/// The portable description of one critter's body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyDocument {
    pub species: SpeciesId,
    pub root: PartId,
    /// The heritable rules that decide where growth goes. Parts fill in during
    /// an epoch; this changes between them.
    pub plan: BodyPlan,
    /// Ordered by `PartId`, so iteration is deterministic.
    pub parts: Vec<Part>,
}

/// Returned when an attachment names a part that does not exist or would
/// close a cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachError {
    UnknownParent(PartId),
    CycleDetected,
}

impl BodyDocument {
    /// A body with a single root part.
    pub fn new(species: SpeciesId, volume: VolumeRef, mass_mg: u64, half_extent: [i32; 3]) -> Self {
        let root = PartId(0);
        Self {
            species,
            root,
            plan: BodyPlan::default(),
            parts: vec![Part {
                id: root,
                volume,
                mass_mg,
                half_extent,
                pivot: half_extent,
                attachment: None,
                provenance: Provenance::founding(),
                severed: false,
            }],
        }
    }

    pub fn part(&self, id: PartId) -> Option<&Part> {
        self.parts.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Adds a part fixed to `attachment.parent`, returning its new id.
    pub fn attach(
        &mut self,
        volume: VolumeRef,
        mass_mg: u64,
        half_extent: [i32; 3],
        attachment: Attachment,
        provenance: Provenance,
    ) -> Result<PartId, AttachError> {
        if self.part(attachment.parent).is_none() {
            return Err(AttachError::UnknownParent(attachment.parent));
        }
        let id = PartId(self.parts.len() as u32);
        self.parts.push(Part {
            id,
            volume,
            mass_mg,
            half_extent,
            // Centre by default. A part authored with a socket elsewhere can
            // override it; nothing generated needs to.
            pivot: half_extent,
            attachment: Some(attachment),
            provenance,
            severed: false,
        });
        Ok(id)
    }

    /// Where a part's **pivot** sits in body space, walking up the chain.
    ///
    /// This is the authoritative position. Everything else derives from it.
    ///
    /// Returns `None` if the chain is malformed, which the constructors above
    /// prevent but a deserialized document could carry.
    pub fn world_pivot(&self, id: PartId) -> Option<[i32; 3]> {
        let mut offset = [0i32; 3];
        let mut cursor = id;
        // Bounded by part count, so a cycle terminates rather than hanging.
        for _ in 0..=self.parts.len() {
            let part = self.part(cursor)?;
            match part.attachment {
                None => {
                    return Some(offset);
                },
                Some(a) => {
                    let rotated = a.yaw.rotate(offset);
                    offset = [
                        rotated[0] + a.offset[0],
                        rotated[1] + a.offset[1],
                        rotated[2] + a.offset[2],
                    ];
                    cursor = a.parent;
                },
            }
        }
        None
    }

    /// Where a part's lowest corner sits in body space.
    ///
    /// Derived from the pivot rather than accumulated, so a rotated part stays
    /// joined: the pivot holds still and the body swings around it.
    pub fn world_offset(&self, id: PartId) -> Option<[i32; 3]> {
        let pivot_at = self.world_pivot(id)?;
        let yaw = self.world_yaw(id)?;
        let part = self.part(id)?;
        let swung = yaw.rotate(part.pivot);
        Some([
            pivot_at[0] - swung[0],
            pivot_at[1] - swung[1],
            pivot_at[2] - swung[2],
        ])
    }

    /// Maps a point in a part's own voxel space into body space.
    ///
    /// The one transform a projection needs: rotate about the pivot, then put
    /// the pivot where it belongs.
    pub fn place(&self, id: PartId, local: [i32; 3]) -> Option<[i32; 3]> {
        let pivot_at = self.world_pivot(id)?;
        let yaw = self.world_yaw(id)?;
        let part = self.part(id)?;
        let relative = [
            local[0] - part.pivot[0],
            local[1] - part.pivot[1],
            local[2] - part.pivot[2],
        ];
        let swung = yaw.rotate(relative);
        Some([
            pivot_at[0] + swung[0],
            pivot_at[1] + swung[1],
            pivot_at[2] + swung[2],
        ])
    }

    /// Orientation of a part in body space, composing every joint up the
    /// chain. A projection needs this alongside [`Self::world_offset`] to
    /// place a part's volume; position alone would draw every part unrotated.
    pub fn world_yaw(&self, id: PartId) -> Option<Yaw> {
        let mut yaw = Yaw::Zero;
        let mut cursor = id;
        for _ in 0..=self.parts.len() {
            let part = self.part(cursor)?;
            match part.attachment {
                None => return Some(yaw),
                Some(a) => {
                    yaw = a.yaw.compose(yaw);
                    cursor = a.parent;
                },
            }
        }
        None
    }

    /// Mass of what is still attached. A creature does not carry the arm it
    /// lost, so severed parts weigh nothing here.
    pub fn total_mass_mg(&self) -> u64 {
        self.living().map(|p| p.mass_mg).sum()
    }

    /// Mass-weighted centre in voxel units, rounded toward zero.
    ///
    /// Uses each part's **centre**, not its origin. A part's origin is its
    /// lowest corner, so averaging origins biases the result by every part's
    /// size and reports a balance the body does not have. This is the same
    /// corner-versus-pivot confusion recorded in the body pipeline plan,
    /// surfacing a third time.
    ///
    /// Accumulated in `i128` so a large body cannot overflow into a different
    /// answer on a different platform.
    pub fn centre_of_mass(&self) -> [i32; 3] {
        let total = self.total_mass_mg();
        if total == 0 {
            return [0; 3];
        }
        let mut acc = [0i128; 3];
        for part in self.living() {
            // A pivot is the part's centre, so this is a true mass-weighted
            // centre rather than an average of corners.
            let Some(centre) = self.world_pivot(part.id) else {
                continue;
            };
            for axis in 0..3 {
                acc[axis] += centre[axis] as i128 * part.mass_mg as i128;
            }
        }
        let total = total as i128;
        [
            (acc[0] / total) as i32,
            (acc[1] / total) as i32,
            (acc[2] / total) as i32,
        ]
    }

    /// The body's collision extent: the union of every part's box.
    pub fn aabb(&self) -> Aabb {
        let mut result: Option<Aabb> = None;
        for part in self.living() {
            // `around` wants a centre. Passing the corner made every part's box
            // straddle its own edge; a pivot is the centre it always wanted.
            let Some(centre) = self.world_pivot(part.id) else {
                continue;
            };
            let box_ = Aabb::around(centre, part.half_extent);
            result = Some(match result {
                None => box_,
                Some(acc) => acc.union(box_),
            });
        }
        result.unwrap_or(Aabb {
            min: [0; 3],
            max: [0; 3],
        })
    }

    /// Every part that was taken from another organism, in id order.
    pub fn incorporated(&self) -> impl Iterator<Item = &Part> {
        self.parts
            .iter()
            .filter(|p| matches!(p.provenance.origin, Origin::Incorporated { .. }))
    }
}

/// The wire form of [`Provenance`]. Flat on purpose: `None` for both fields
/// means the part was there at founding, and a foreign reader needs no enum
/// from this crate to tell that from a part that was taken off somebody.
impl From<&Provenance> for PartOrigin {
    fn from(provenance: &Provenance) -> Self {
        match provenance.origin {
            Origin::Founding => PartOrigin {
                from_species: None,
                from_part: None,
                epoch: provenance.epoch,
            },
            Origin::Incorporated {
                from_species,
                from_part,
            } => PartOrigin {
                from_species: Some(from_species.0),
                from_part: Some(from_part.0),
                epoch: provenance.epoch,
            },
        }
    }
}

impl From<&PartOrigin> for Provenance {
    fn from(origin: &PartOrigin) -> Self {
        match (origin.from_species, origin.from_part) {
            (Some(species), Some(part)) => Provenance {
                origin: Origin::Incorporated {
                    from_species: SpeciesId(species),
                    from_part: PartId(part),
                },
                epoch: origin.epoch,
            },
            _ => Provenance {
                origin: Origin::Founding,
                epoch: origin.epoch,
            },
        }
    }
}

// Split at the 600-line ceiling: same module, a separate file, per the
// `places/bricks/tests.rs` precedent.
#[cfg(test)]
mod tests;
