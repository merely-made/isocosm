// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shared R0/R1 body selection and resident local-pose images (case 1161).

mod sprites;
#[cfg(test)]
mod tests;

pub use crate::camera::projected_size;
pub use isometer_render::hybrid::{
    BodyChoice, BodyDetail, BodySelection, HybridPolicy, RefreshClass,
};
pub use sprites::{SpriteCache, SpriteInstance, SpritePlacement, SpriteRevisions, SpriteSource};
