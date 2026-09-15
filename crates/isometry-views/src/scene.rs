//! The scene adapter: the board seen by the isometer scene rather than the DOM.
//!
//! B1 of the board-on-isometer plan lands the terrain half: a
//! [`isometry_core::MapDocument`] read as isometer-core's `Terrain`, plus the
//! material palette that binds its tile kinds to the tileset's colours.

mod terrain;

#[cfg(test)]
mod terrain_tests;

pub use terrain::{MapTerrain, SEA_LEVEL, VOID_SURFACE, terrain_palette};
