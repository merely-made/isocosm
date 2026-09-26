// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Paged terrain: the bricks a frame shows, and one capacity-fixed brick map
//! that follows them.
//!
//! A terrain larger than the atlas is not held whole. Its product makes any
//! brick on demand through a [`BrickSource`]; [`framed_bricks`] names the ones
//! a camera can show; and a [`Residency`] keeps a single capacity-fixed
//! [`BrickMap`] holding exactly those, retargeting it as the frame moves and
//! refreshing the bricks an edit touched. A kept brick keeps its slot, so a
//! pan uploads the pointer volume and the bricks it brought into view.
//!
//! **Which bricks.** Every brick whose projected box overlaps the frame grown
//! by a margin in world units: a pure function of the camera and the terrain,
//! so one view of one map holds the same bricks whichever way it was reached.
//! When the frame shows more than the atlas holds, the bricks farthest from
//! the frame's centre go first, the margin's before the frame's own, and are
//! counted as [`FramedBricks::overflow`].
//!
//! **Headroom.** The pointer volume holds the terrain's layers and
//! [`ResidencySettings::headroom`] spare ones above them, sized across as well
//! as up, so an edit that lifts the terrain into them retargets and only one
//! past them rebuilds the map ([`Rebuild::Headroom`]).
//!
//! **The hold.** modulus at the pinned revision bounds a slot by the number of
//! keys rather than by the atlas, so after a retarget that shrinks the
//! selection a kept brick can sit in a slot past that number: the GPU still
//! draws it, the CPU reads it as air, and refreshing it panics. Until the pin
//! moves past the fix, a selection that would shrink rebuilds the map from
//! empty instead, which keeps the slots packed ([`Rebuild::Shrink`]).

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::ops::Range;

use isometer_core::ground::BRICK;
use isometer_lens::BrickMap;
use isometer_lens::bricks::{
    ATLAS_SLOTS_X, ATLAS_SLOTS_Z, BrickProjectionRevision, MAX_ATLAS_SLOTS_Y,
};

use super::terrain::{TerrainRefresh, TerrainSource};
use crate::camera::{SlabCamera, SlabWindow};

mod framing;
mod paging;
#[cfg(test)]
mod tests;

pub use framing::{FramedBricks, framed_bricks};
pub use paging::{PagedTerrain, Rebuild, Residency, ResidencySettings, ResidencyStats};

/// Bytes in one brick: eight cubed.
pub const BRICK_BYTES: usize = (BRICK * BRICK * BRICK) as usize;

/// A terrain whose bricks are made on demand rather than held whole.
///
/// Keys and voxels are the ground's own: brick `key` spans `key * BRICK` up
/// to `(key + 1) * BRICK` on each axis, and its bytes run Y, then Z, then X
/// with X contiguous.
pub trait BrickSource {
    /// The inclusive key box every brick of this terrain lies in, or `None`
    /// when it has none.
    fn bounds(&self) -> Option<[[i16; 3]; 2]>;

    /// The y keys one brick column can hold solid voxels in, low to high.
    /// Every brick outside the range is air; one inside it may be too, at the
    /// cost of the atlas slot it takes.
    fn layers(&self, column: [i16; 2]) -> Range<i16>;

    /// Writes one brick's materials into `out`, [`BRICK_BYTES`] long.
    fn fill(&self, key: [i16; 3], out: &mut [u8]);
}
