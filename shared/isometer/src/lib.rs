// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The games wing's shared scene: one orthographic slab, one world cut, one
//! depth attachment.
//!
//! What lives here is the geometry every vessel agrees on — where the camera
//! is, how far it reaches, what it culls against, and how a body document's
//! volumes are resolved. What does not live here is any product's world:
//! Mesocosm's `CameraMode` presets, Paredros's `CameraPolicy` and Isometry's
//! board all stay in their own hosts and hand this crate a forward vector.
//!
//! See `mesocosm/design_docs/2026-09-14_isometer_extraction_plan.md`.

mod anchors;
mod bodies;
mod camera;
mod capture;
mod glyphs;
mod producer;
mod query;
mod scene;
mod volumes;

pub use anchors::{GlyphAnchor, MAX_GLYPH_ANCHORS};
pub use bodies::{
    BodyFrameStats, BodyLayer, PartAddress, Pose, SceneBody, SceneVolumes, SubjectKey,
};
pub use camera::{Cutaway, SlabCamera, SlabWindow};
pub use glyphs::{GlyphOrientation, MAX_SPATIAL_GLYPHS, SpatialGlyph, Stroke};
pub use producer::{
    BodySignature, FrameRequest, SCENE_ALPHA, SCENE_ENCODING, SceneProducer, SceneSignature,
    SceneSource,
};
pub use query::{BodyPick, BodyPickError};
pub use scene::{
    CapsuleFrame, GroundTerrain, HostTerrain, Scene, SceneFrame, SceneHost, SceneStats,
    TerrainRefresh, TerrainSource,
};
pub use volumes::DeclaredExtentVolumes;

/// The volume seam, re-exported so a producer needs one crate in scope.
pub use isometer_core::VolumeRef;
pub use mesocosm_mesh::{Volume, VolumeMap, VolumeSource};
