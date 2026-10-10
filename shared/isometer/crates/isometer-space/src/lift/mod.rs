// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The lift (the place-graph plan's SP1 and SP2, moved here by ruling 701):
//! corners shared by every site around them, edge profiles both sides
//! compute alike, a site's exact surface lattice, and chunks of columns at
//! power-of-two cell sizes. Integers throughout.

pub mod check;
mod chunk;
mod corners;
mod lattice;
mod profile;

pub use chunk::{CHUNK, Chunk, Exception, Materials};
pub(crate) use chunk::{chunk, chunks, soil_depth};
pub use corners::CornerKey;
pub(crate) use corners::{corner_class, corner_height};
pub use lattice::{Lattice, POINTS, fading};
pub(crate) use lattice::lattice_with;
pub use profile::{EdgeProfile, SPANS};
pub(crate) use profile::{border_height, canonical, edge_profile};

/// A draw spread over `-amplitude..=amplitude`.
fn signed(draw: u64, amplitude: i64) -> i64 {
    if amplitude <= 0 {
        return 0;
    }
    (draw % (2 * amplitude as u64 + 1)) as i64 - amplitude
}
