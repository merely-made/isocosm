//! The scene adapter: the board seen by the isometer scene rather than the DOM.
//!
//! B1 landed the terrain half: a [`isometry_core::MapDocument`] read as
//! isometer-core's `Terrain`, plus the material palette that binds its tile
//! kinds to the tileset's colours. B2 adds the rest of the board — where it
//! stands in the world and what camera frames it ([`world`]), its tokens as
//! live bodies from the DOM board's own recipes ([`tokens`]), and the
//! [`isometer::SceneSource`] that draws all three as one frame ([`board`]).
//!
//! B3 adds the pointer's half: [`pick`] is the handle a gesture asks what is
//! under the cursor, so a press, a click, a drag and a hover resolve through
//! the frame the scene actually drew rather than through a flat-ground
//! inverse.
//!
//! The grid under all of it was subdivided on 2026-09-15 by Mark's cliff-height
//! ruling: [`VOXELS_PER_TILE`] voxels to a tile and [`VOXELS_PER_STEP`] to an
//! elevation step, so a cube grid can hold a step shallower than a tile.

mod board;
mod pick;
mod terrain;
mod tokens;
mod world;

#[cfg(test)]
mod parity_tests;
#[cfg(test)]
mod terrain_tests;

pub use board::{
    BOARD_SCENE_LEAF_KEY, BoardHandle, BoardPick, BoardProducer, BoardSource, BoardView,
};
pub use pick::ScenePick;
pub use terrain::{
    MapTerrain, SEA_LEVEL, VOID_SURFACE, VOXELS_PER_STEP, VOXELS_PER_TILE, surface_of,
    terrain_palette,
};
pub use tokens::{TokenBodies, owner_tint, yaw_of};
pub use world::{BoardWorld, ELEVATION_PX, WORLD_PX, elevation_px, world_px};
