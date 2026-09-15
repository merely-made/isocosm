//! The scene adapter: the board seen by the isometer scene rather than the DOM.
//!
//! B1 landed the terrain half: a [`isometry_core::MapDocument`] read as
//! isometer-core's `Terrain`, plus the material palette that binds its tile
//! kinds to the tileset's colours. B2 adds the rest of the board — where it
//! stands in the world and what camera frames it ([`world`]), its tokens as
//! live bodies from the DOM board's own recipes ([`tokens`]), and the
//! [`isometer::SceneSource`] that draws all three as one frame ([`board`]).

mod board;
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
pub use terrain::{MapTerrain, SEA_LEVEL, VOID_SURFACE, terrain_palette};
pub use tokens::{TokenBodies, owner_tint, yaw_of};
pub use world::{BoardWorld, ELEVATION_PX, WORLD_PX, world_px};
