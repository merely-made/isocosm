// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Paredros anatomies as drawable voxel bodies.
//!
//! **Declared limit.** Paredros anatomies address volumes by
//! [`VolumeRef::from_tag`] and carry no voxel data at all. Every intact part is
//! therefore drawn as one [`Volume::solid`] of the part's own declared extent
//! (`half_extent * 2`) in a single neutral material, assembled by
//! [`mesocosm_mesh::mesh_body`] into one [`BodyMesh`] per body. Severed parts
//! are omitted, because the core's `living()` walk omits them. A part's shape
//! is therefore its declared box and nothing finer, and two parts of one body
//! are told apart geometrically, never by colour. Subject identity is the
//! per-body tint.
//!
//! **Second declared limit.** The live body renderer caches immutable geometry
//! by `VolumeRef` bytes. Two subjects whose anatomies reuse one tag for parts
//! of *different* declared extents therefore share the first cached box. That
//! is counted in [`BodyLayer::volume_conflicts`] rather than drawn wrong
//! silently; every fixture and generated body in the repository today gives one
//! extent per tag.

use std::collections::BTreeMap;

use mesocosm_core::{BodyDocument, PartId, VolumeRef};
use mesocosm_mesh::{BodyMesh, Volume, VolumeMap, mesh_body};
use mesocosm_render::live_body::LiveBody;
use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::{GameState, MOTION_SCALE, MotionPose};

/// The material every part is drawn in. 245 is the core's flat white tone, so
/// the colour a pixel carries is the body's tint times a face shade and
/// nothing else. Configurable, because the palette is presentation.
pub const BODY_MATERIAL: u8 = 245;

/// Body-document voxel units per world voxel. The anatomy's units are its own:
/// the fixture's root is four units tall inside a two-voxel motion envelope, so
/// a quarter puts a body at roughly the height the solver collides at.
pub const BODY_SCALE: f32 = 0.25;

/// How a subject is coloured. Identity, not decoration: the tests read it back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Appearance {
    /// The played subject.
    pub played: [f32; 3],
    /// Everyone else.
    pub other: [f32; 3],
    pub material: u8,
    pub scale: f32,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            played: [1.0, 0.42, 0.12],
            other: [0.20, 0.62, 1.0],
            material: BODY_MATERIAL,
            scale: BODY_SCALE,
        }
    }
}

/// One subject's drawable state for one frame.
pub struct DrawnBody {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub pose: MotionPose,
    pub origin: [f32; 3],
    pub scale: f32,
    pub tint: [f32; 3],
    mesh: BodyMesh,
}

impl DrawnBody {
    pub fn live(&self) -> LiveBody<'_> {
        let mut body = LiveBody::new(&self.mesh, self.origin);
        body.scale = self.scale;
        body.tint = self.tint;
        body
    }

    /// World bounds of every drawn part.
    pub fn bounds(&self) -> Option<([f32; 3], [f32; 3])> {
        let (min, max) = self.mesh.bounds()?;
        Some((
            [0, 1, 2].map(|i| self.origin[i] + min[i] as f32 * self.scale),
            [0, 1, 2].map(|i| self.origin[i] + max[i] as f32 * self.scale),
        ))
    }

    /// Intact parts, in the order they are drawn.
    pub fn parts(&self) -> impl Iterator<Item = PartId> + '_ {
        self.mesh.placements.iter().map(|placement| placement.part)
    }

    /// World bounds of one part, for framing and for region assertions.
    pub fn part_bounds(&self, part: PartId) -> Option<([f32; 3], [f32; 3])> {
        let placement = self.mesh.placements.iter().find(|p| p.part == part)?;
        let mesh = self.mesh.mesh_for(placement.volume)?;
        let mut min = [f32::MAX; 3];
        let mut max = [f32::MIN; 3];
        for quad in &mesh.quads {
            for corner in quad.corners() {
                let placed = mesocosm_mesh::place_point(
                    corner,
                    placement.yaw,
                    placement.pivot,
                    placement.pivot_at,
                );
                for axis in 0..3 {
                    let value = self.origin[axis] + placed[axis] as f32 * self.scale;
                    min[axis] = min[axis].min(value);
                    max[axis] = max[axis].max(value);
                }
            }
        }
        (min[0] <= max[0]).then_some((min, max))
    }
}

/// The per-frame body projection, plus the geometry cache that survives
/// suspension. Cached meshes are keyed by the pair a body's shape actually
/// depends on, so an unchanged body costs no rebuild and no upload.
#[derive(Default)]
pub struct BodyLayer {
    cache: BTreeMap<(SubjectId, BodyRevisionId), BodyMesh>,
    pub bodies: Vec<DrawnBody>,
    pub volume_conflicts: usize,
    pub missing_pose: usize,
    pub projection_failures: usize,
    pub last_error: Option<String>,
}

impl BodyLayer {
    /// Rebuilds the frame's drawable bodies from one `GameState`.
    ///
    /// Anatomy comes from `current_anatomy`, falling back to the last admitted
    /// record when injury has made it stale: an out-of-date body is still a
    /// truer picture than a missing one, and the fallback is counted.
    pub fn prepare(&mut self, game: &GameState, played: SubjectId, appearance: Appearance) {
        self.bodies.clear();
        self.volume_conflicts = 0;
        self.missing_pose = 0;
        self.projection_failures = 0;
        let subjects: Vec<_> = game
            .bodies()
            .all()
            .filter(|body| body.alive())
            .map(|body| body.subject)
            .collect();
        let mut documents = Vec::new();
        for subject in subjects {
            let Some(record) = game
                .current_anatomy(subject)
                .ok()
                .or_else(|| game.anatomies().get(subject))
            else {
                continue;
            };
            let Some(pose) = game
                .pose(subject)
                .or_else(|| game.movement().position(subject).map(MotionPose::at_cell))
            else {
                self.missing_pose += 1;
                continue;
            };
            documents.push((subject, record, pose));
        }
        let volumes = self.volumes(&documents, appearance.material);
        for (subject, record, pose) in documents {
            let mesh = match self.mesh_for(subject, record.revision, &record.document, &volumes) {
                Ok(mesh) => mesh,
                Err(error) => {
                    self.projection_failures += 1;
                    self.last_error = Some(format!("subject {}: {error:?}", subject.0));
                    continue;
                },
            };
            let Some((min, max)) = mesh.bounds() else {
                continue;
            };
            let feet = [0, 1, 2].map(|i| pose.position[i] as f32 / MOTION_SCALE as f32);
            // Stand the declared boxes on the pose's feet and centre them on
            // its column, so the picture sits where the solver put the body.
            let origin = [
                feet[0] - (min[0] + max[0]) as f32 * 0.5 * appearance.scale,
                feet[1] - min[1] as f32 * appearance.scale,
                feet[2] - (min[2] + max[2]) as f32 * 0.5 * appearance.scale,
            ];
            self.bodies.push(DrawnBody {
                subject,
                revision: record.revision,
                pose,
                origin,
                scale: appearance.scale,
                tint: if subject == played {
                    appearance.played
                } else {
                    appearance.other
                },
                mesh,
            });
        }
    }

    /// One solid box per addressed volume, at the extent its parts declare.
    fn volumes(
        &mut self,
        documents: &[(SubjectId, &paredros_world::AnatomyRecord, MotionPose)],
        material: u8,
    ) -> VolumeMap {
        let mut extents: BTreeMap<VolumeRef, [u32; 3]> = BTreeMap::new();
        for (_, record, _) in documents {
            for part in record.document.living() {
                let extent = part.half_extent.map(|half| (half.max(0) as u32) * 2);
                if extent.iter().any(|d| *d == 0) {
                    continue;
                }
                match extents.entry(part.volume) {
                    std::collections::btree_map::Entry::Vacant(slot) => {
                        slot.insert(extent);
                    },
                    std::collections::btree_map::Entry::Occupied(slot) => {
                        self.volume_conflicts += usize::from(*slot.get() != extent);
                    },
                }
            }
        }
        let mut volumes = VolumeMap::new();
        for (reference, extent) in extents {
            volumes.insert(reference, Volume::solid(extent, material));
        }
        volumes
    }

    fn mesh_for(
        &mut self,
        subject: SubjectId,
        revision: BodyRevisionId,
        document: &BodyDocument,
        volumes: &VolumeMap,
    ) -> Result<BodyMesh, mesocosm_mesh::MeshError> {
        if let Some(mesh) = self.cache.get(&(subject, revision)) {
            return Ok(mesh.clone());
        }
        let mesh = mesh_body(document, volumes)?;
        // A severed part retires its revision, so the map cannot grow past one
        // entry per body that is still being drawn.
        self.cache.retain(|(held, _), _| *held != subject);
        self.cache.insert((subject, revision), mesh.clone());
        Ok(mesh)
    }

    pub fn cached_bodies(&self) -> usize {
        self.cache.len()
    }

    pub fn body(&self, subject: SubjectId) -> Option<&DrawnBody> {
        self.bodies.iter().find(|body| body.subject == subject)
    }

    /// What the skip test compares: identity, revision and exact pose.
    pub fn signature(&self) -> Vec<(SubjectId, BodyRevisionId, MotionPose)> {
        self.bodies
            .iter()
            .map(|body| (body.subject, body.revision, body.pose))
            .collect()
    }
}
