// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1: one played `GameState` as a Cambium document leaf.
//!
//! The scene itself is the wing's shared crate, `isometer`: bodies rastered
//! into a depth attachment, terrain traced against that same depth, both under
//! one `clip_from_world`, plus the unchanged-input skip and the part queries
//! over the frame that was actually drawn. This module is the translation from
//! a Paredros session to that crate's product-neutral inputs, and nothing else.
//!
//! - [`SceneModelSource`] is the [`isometer::SceneSource`]: which subjects are
//!   drawn, where their anatomies stand, which way the section looks, where the
//!   terrain comes from.
//! - [`SceneProducer`] is [`isometer::SceneProducer`] over it, and is what
//!   rootstock registers.
//!
//! # Declared limits
//!
//! - **Anatomies have no voxel data.** Paredros bodies address volumes by
//!   [`isometer_core::VolumeRef::from_tag`] and store no voxels, so each intact
//!   part is drawn as one solid box of its own declared extent
//!   (`half_extent * 2`) in a single flat material, through
//!   [`isometer::DeclaredExtentVolumes`]. A part's silhouette is its declared
//!   box and nothing finer. Severed parts are omitted.
//! - **Body scale is presentation.** Anatomy units are the document's own and
//!   the world's are voxels; nothing in `paredros-world` relates them, so
//!   [`Appearance::scale`] owns the conversion and defaults to [`BODY_SCALE`].
//! - **No per-part material expression is projected**: Paredros has no
//!   phenotype to project, and bodies neither cast nor receive terrain shadows.

mod handle;
mod policy;
mod source;

pub use handle::{Held, SceneHandle, SceneModel};
pub use policy::{Appearance, BODY_MATERIAL, BODY_SCALE, CameraPolicy};
pub use source::SceneModelSource;

/// The producer rootstock registers: the shared skip and output contract over
/// this host's own scene source.
pub type SceneProducer = isometer::SceneProducer<SceneModelSource>;

#[cfg(test)]
mod fixture;
#[cfg(test)]
mod harness;
#[cfg(test)]
mod tests;
