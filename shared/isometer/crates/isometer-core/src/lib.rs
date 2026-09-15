// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The isometer family's floor: bodies, ground, and the bytes between them.
//!
//! Everything here is **product-neutral**. There is no world, no organism, no
//! lineage, and no generation model — a body document is a graph of parts with
//! provenance, a ground is a sparse field of voxel bricks, and the codec seam
//! is postcard framing plus an equality witness. Each product keeps its own
//! simulation and hands these types across.
//!
//! # Determinism
//!
//! Integer-only, like the cores that fed it. Voxel coordinates, masses in
//! milligrams and quarter-turn rotations are exact on every platform, so a
//! document encoded on one machine is byte-identical on another and a replay
//! cannot diverge on floating-point behaviour.
//!
//! # Shape
//!
//! ```text
//! BodyDocument   parts, attachment frames, per-part provenance
//! BodyPlan       the heritable rules that decide where growth goes
//! Ground         brick truth: solid voxels, revisions, a dirty queue
//! snapshot       postcard encode/decode plus the FNV-1a state witness
//! wire           the shared framed header
//! ```

pub mod anatomy;
pub mod body;
pub mod ground;
pub mod plan;
pub mod snapshot;
pub mod wire;

pub use body::{
    Aabb, AttachError, Attachment, BodyDocument, Origin, Part, PartId, Provenance, SpeciesId,
    VolumeRef, Yaw,
};
pub use ground::{AIR, BRICK, Brick, Cavity, Ground, ROCK, SOIL, SURFACE_BAND, Terrain};
pub use plan::{BodyPlan, Facing, Role, Symmetry, classify};
pub use wire::{WireError, frame, unframe};

/// The flat wire form of [`Provenance`], shared with every other v0 reader.
pub use wing_formats::PartOrigin;
