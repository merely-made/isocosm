// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The sprite projection: a voxel volume becomes a pixel sheet.
//!
//! The other half of this crate's two projections. [`greedy`](crate::greedy)
//! emits quads for a live renderer; this module bakes the same voxel truth at
//! the locked isometric angle and emits RGBA pixels, which is what a tileset
//! wants. One volume, two outputs, and the body document is the shared organ.
//!
//! A token is a recipe, not an image: an [`Appearance`] names layers and a
//! [`Palette`], and recolouring is a palette swap rather than a repaint, so a
//! character creator restyles without touching a silhouette.
//!
//! Nothing here touches a GPU either. The `.vox` importer sits behind the
//! default-off `vox` feature, so a consumer that only bakes in-crate models
//! pulls no file-format dependency.

pub mod demo;
pub mod watchtower;

mod body;
mod png;
mod recipe;
mod sheet;
#[cfg(feature = "vox")]
mod vox;

#[cfg(test)]
mod tests;

pub use recipe::{Appearance, Clip, Palette, compose};
pub use sheet::{BakeParams, Sheet, bake_facing, bake_strip};
#[cfg(feature = "vox")]
pub use vox::load_vox;
