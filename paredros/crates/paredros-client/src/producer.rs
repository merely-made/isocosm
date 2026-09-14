// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1: one played `GameState` as a Cambium document leaf.
//!
//! [`SceneProducer`] implements [`cambium_rootstock::TextureProducer`] over a
//! shared [`SceneHandle`]. It draws, in this order into one colour target and
//! one depth attachment:
//!
//! 1. every living subject's current anatomy at its exact pose, through
//!    `mesocosm-render`'s `LiveBodyRenderer`;
//! 2. the game's own ground, through `mesocosm-lens`'s `BrickTracer` and its
//!    depth join, under an orthographic slab camera.
//!
//! Both consume one `clip_from_world` from [`SlabView`], so a body standing
//! behind rock is covered by it and a body standing in front of rock covers it,
//! per pixel. Nothing here is renderling: the tracer, the body renderer and the
//! camera are all renderling-free, and this module adds no third path.
//!
//! # Declared limits
//!
//! - **Anatomies have no voxel data.** Paredros bodies address volumes by
//!   [`mesocosm_core::VolumeRef::from_tag`] and store no voxels, so each intact
//!   part is drawn as one solid box of its own declared extent
//!   (`half_extent * 2`) in a single flat material. A part's silhouette is its
//!   declared box and nothing finer. Severed parts are omitted. See
//!   [`bodies`] for the second, narrower limit about two subjects sharing one
//!   volume tag at different extents.
//! - **Body scale is presentation.** Anatomy units are the document's own and
//!   the world's are voxels; nothing in `paredros-world` relates them, so
//!   [`Appearance::scale`] owns the conversion and defaults to
//!   [`bodies::BODY_SCALE`].
//! - **Picking ignores terrain.** See
//!   [`SceneProducer::pick_body_ignoring_terrain`].
//! - **Bodies do not cast or receive terrain shadows**, and no per-part
//!   material expression is projected: Paredros has no phenotype to project.

mod bodies;
mod camera;
mod handle;
mod scene;

pub use bodies::{Appearance, BODY_MATERIAL, BODY_SCALE, BodyLayer, DrawnBody};
pub use camera::{CameraPolicy, SlabView};
pub use handle::{SceneHandle, SceneModel};
pub use scene::SceneProducer;

#[cfg(test)]
mod fixture;
#[cfg(test)]
mod harness;
#[cfg(test)]
mod tests;
