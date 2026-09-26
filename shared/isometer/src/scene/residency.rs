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

/// What one camera frames of a terrain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FramedBricks {
    /// The framed bricks, sorted, without repeats, never past the capacity.
    pub keys: Vec<[i16; 3]>,
    /// Framed bricks dropped because the atlas could not hold them.
    pub overflow: usize,
    /// A pointer extent holding any framing by a camera of this size,
    /// wherever it stands, so a map built at it is retargeted rather than
    /// rebuilt until the camera's size or the terrain's height moves.
    pub extent: [u32; 3],
}

/// The bricks `camera` can show of `source`: those whose screen box overlaps
/// the frame grown by `margin` world units along the screen's right and up,
/// and whose box along the camera's forward is within the slab's reach. At
/// most `capacity` are kept.
pub fn framed_bricks(
    camera: SlabCamera,
    margin: f32,
    source: &dyn BrickSource,
    capacity: usize,
) -> FramedBricks {
    let Some([low, high]) = source.bounds() else {
        return FramedBricks {
            keys: Vec::new(),
            overflow: 0,
            extent: [1; 3],
        };
    };
    let edge = BRICK as f32;
    let slab = [f32::from(low[1]) * edge, (f32::from(high[1]) + 1.0) * edge];
    let window = grown(camera.window(), margin);
    let frame = Frame::of(window, margin);
    // A brick's screen box can reach the frame where no point of the brick is
    // inside it, so the columns are searched over the window grown by one more
    // box width: every point of such a brick lies within that.
    let search = SlabWindow {
        half: [0, 1, 2].map(|axis| window.half[axis] + 2.0 * frame.spread[axis]),
        ..window
    };
    let extent = extent_of(search, slab, low, high);
    let Some([near, far]) = region(search, slab) else {
        return FramedBricks {
            keys: Vec::new(),
            overflow: 0,
            extent,
        };
    };
    let key = |value: f32| (value / edge).floor() as i32;
    let columns = |axis: usize, horizontal: usize| {
        key(near[horizontal]).max(low[axis].into())..=key(far[horizontal]).min(high[axis].into())
    };
    let mut framed = Vec::new();
    for z in columns(2, 1) {
        for x in columns(0, 0) {
            let column = [x as i16, z as i16];
            let layers = source.layers(column);
            for y in layers.start.max(low[1])..layers.end.min(high[1] + 1) {
                let brick = [column[0], y, column[1]];
                if let Some(distance) = frame.distance(brick) {
                    framed.push((distance, brick));
                }
            }
        }
    }
    let overflow = framed.len().saturating_sub(capacity);
    if overflow > 0 {
        framed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        framed.truncate(capacity);
    }
    let mut keys: Vec<_> = framed.into_iter().map(|(_, brick)| brick).collect();
    keys.sort_unstable();
    FramedBricks {
        keys,
        overflow,
        extent,
    }
}

/// The camera's window with the margin added to its screen half extents.
fn grown(window: SlabWindow, margin: f32) -> SlabWindow {
    SlabWindow {
        half: [
            window.half[0] + margin,
            window.half[1] + margin,
            window.half[2],
        ],
        ..window
    }
}

/// The frame a brick is tested against, in the camera's own axes.
struct Frame {
    centre: [f32; 3],
    axes: [[f32; 3]; 3],
    /// The grown half extents: where a brick stops overlapping.
    reach: [f32; 3],
    /// The frame's own half extents, before the margin: the unit a brick's
    /// distance from the centre is counted in.
    unit: [f32; 2],
    /// Half a brick's box, measured along each axis.
    spread: [f32; 3],
}

impl Frame {
    fn of(window: SlabWindow, margin: f32) -> Self {
        let half = BRICK as f32 / 2.0;
        Self {
            centre: window.centre,
            axes: window.axes,
            reach: window.half,
            unit: [
                (window.half[0] - margin).max(f32::EPSILON),
                (window.half[1] - margin).max(f32::EPSILON),
            ],
            spread: window
                .axes
                .map(|axis| half * axis.iter().map(|v| v.abs()).sum::<f32>()),
        }
    }

    /// How far from the frame's centre the brick's nearest point lands, in
    /// frame half-widths (1 is the frame's own edge), or `None` when the brick
    /// misses the grown frame or the slab.
    fn distance(&self, key: [i16; 3]) -> Option<f32> {
        let half = BRICK as f32 / 2.0;
        let offset = [0, 1, 2].map(|i| f32::from(key[i]) * BRICK as f32 + half - self.centre[i]);
        let mut nearest = 0.0f32;
        for axis in 0..3 {
            let along: f32 = (0..3).map(|i| offset[i] * self.axes[axis][i]).sum();
            let gap = along.abs() - self.spread[axis];
            if gap > self.reach[axis] {
                return None;
            }
            if axis < 2 {
                nearest = nearest.max(gap.max(0.0) / self.unit[axis]);
            }
        }
        Some(nearest)
    }
}

/// The x and z bounds of the part of `window` between the heights in `slab`:
/// the box of the corners inside the slab and the edges crossing its faces.
fn region(window: SlabWindow, slab: [f32; 2]) -> Option<[[f32; 2]; 2]> {
    // Corner `bits` takes the positive half extent on each axis whose bit is
    // set; an edge joins two corners one bit apart.
    let corners: [[f32; 3]; 8] = std::array::from_fn(|bits| {
        let mut point = window.centre;
        for axis in 0..3 {
            let sign = if bits >> axis & 1 == 1 { 1.0 } else { -1.0 };
            for (i, value) in point.iter_mut().enumerate() {
                *value += sign * window.half[axis] * window.axes[axis][i];
            }
        }
        point
    });
    let mut low = [f32::INFINITY; 2];
    let mut high = [f32::NEG_INFINITY; 2];
    let mut take = |point: [f32; 3]| {
        for (slot, value) in [point[0], point[2]].into_iter().enumerate() {
            low[slot] = low[slot].min(value);
            high[slot] = high[slot].max(value);
        }
    };
    for point in corners {
        if (slab[0]..=slab[1]).contains(&point[1]) {
            take(point);
        }
    }
    for bits in 0..8 {
        for axis in 0..3 {
            if bits >> axis & 1 == 1 {
                continue;
            }
            let (p, q) = (corners[bits], corners[bits | 1 << axis]);
            for level in slab {
                if (p[1] - level) * (q[1] - level) < 0.0 {
                    let t = (level - p[1]) / (q[1] - p[1]);
                    take([0, 1, 2].map(|i| p[i] + t * (q[i] - p[i])));
                }
            }
        }
    }
    (low[0] <= high[0] && low[1] <= high[1]).then_some([low, high])
}

/// The pointer extent any framing by this window's size fits in: its region
/// measured with the window stood at the slab's mid-height, so the answer
/// does not move as the camera pans, one brick of slack for where a region's
/// edges fall on the grid, and never more than the terrain's own key box.
fn extent_of(window: SlabWindow, slab: [f32; 2], low: [i16; 3], high: [i16; 3]) -> [u32; 3] {
    let span = |axis: usize| (i32::from(high[axis]) - i32::from(low[axis]) + 1).max(1) as u32;
    let standing = SlabWindow {
        centre: [0.0, (slab[0] + slab[1]) / 2.0, 0.0],
        ..window
    };
    let region = region(standing, slab);
    let across = |horizontal: usize, axis: usize| {
        region
            .map(|[near, far]| far[horizontal] - near[horizontal])
            .filter(|width| width.is_finite())
            .map_or(span(axis), |width| {
                ((width / BRICK as f32).floor() as u32 + 3).clamp(1, span(axis))
            })
    };
    [across(0, 0), span(1), across(1, 2)]
}

/// Why a residency replaced its map whole rather than retargeting it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rebuild {
    /// The first map, or one the scene lost.
    First,
    /// The host asked: something about every brick changed at once.
    Requested,
    /// The camera's size, or the terrain's height, moved the pointer extent.
    Extent,
    /// The selection shrank, and the hold rebuilds rather than retarget.
    Shrink,
}

/// What the last change did to a residency.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResidencyStats {
    /// Bricks the map holds.
    pub resident: usize,
    /// Bricks the atlas can hold.
    pub capacity: usize,
    /// Bricks brought in, each one slot written.
    pub loaded: usize,
    /// Bricks let go.
    pub evicted: usize,
    /// Kept bricks rewritten because an edit touched them.
    pub refreshed: usize,
    /// Framed bricks the atlas could not hold.
    pub overflow: usize,
    /// Set when the map was replaced whole, and why.
    pub rebuilt: Option<Rebuild>,
    /// The pointer extent the map is built at.
    pub extent: [u32; 3],
}

/// A paged terrain's standing state: which bricks its map holds, the
/// revisions it has handed out, and what the last change did. The map itself
/// is the scene's.
#[derive(Debug)]
pub struct Residency {
    rows: u32,
    resident: BTreeSet<[i16; 3]>,
    projection: u64,
    /// The framing extent the map was built for, and the extent it was built
    /// at, which also holds the keys that were framed then.
    built_for: Option<[u32; 3]>,
    held: [u32; 3],
    rebuild: bool,
    stats: ResidencyStats,
    changes: u64,
}

impl Residency {
    /// A residency over `rows` rows of atlas slots, clamped to what modulus
    /// allows at the pinned revision, [`MAX_ATLAS_SLOTS_Y`].
    pub fn new(rows: u32) -> Self {
        Self {
            rows: rows.clamp(1, MAX_ATLAS_SLOTS_Y),
            resident: BTreeSet::new(),
            projection: 0,
            built_for: None,
            held: [1; 3],
            rebuild: false,
            stats: ResidencyStats::default(),
            changes: 0,
        }
    }

    /// Bricks the atlas holds: every slot but the reserved air slot.
    pub fn capacity(&self) -> usize {
        (ATLAS_SLOTS_X * self.rows * ATLAS_SLOTS_Z - 1) as usize
    }

    /// The bricks the map holds.
    pub fn resident(&self) -> &BTreeSet<[i16; 3]> {
        &self.resident
    }

    /// What the last change did. A frame that changed nothing leaves it.
    pub fn stats(&self) -> ResidencyStats {
        self.stats
    }

    /// How many changes the residency has made: builds, retargets and
    /// refreshes. A host compares it across a frame to learn whether the
    /// frame changed anything.
    pub fn changes(&self) -> u64 {
        self.changes
    }

    /// Replaces the map whole on the next frame, for a change that touches
    /// every brick at once, such as a cut through the terrain.
    pub fn rebuild(&mut self) {
        self.rebuild = true;
    }

    fn next_projection(&mut self) -> BrickProjectionRevision {
        self.projection += 1;
        BrickProjectionRevision(self.projection)
    }

    /// A fresh map at a fresh extent, holding the framed bricks.
    fn build(
        &mut self,
        source: &dyn BrickSource,
        framed: &FramedBricks,
        why: Rebuild,
    ) -> Result<BrickMap, String> {
        let spanned = span_of(&framed.keys);
        let extent = [0, 1, 2].map(|axis| framed.extent[axis].max(spanned[axis]));
        let start = self.next_projection();
        let mut map = BrickMap::with_capacity(start, self.rows, extent)
            .map_err(|error| format!("paged brick map: {error}"))?;
        let bytes = fill(source, &framed.keys);
        let revision = self.next_projection();
        map.retarget_with(revision, framed.keys.iter().copied(), |key| {
            brick_in(&framed.keys, &bytes, key)
        })
        .map_err(|error| format!("paged brick map: {error}"))?;
        let resident: BTreeSet<_> = framed.keys.iter().copied().collect();
        self.stats = ResidencyStats {
            resident: resident.len(),
            capacity: self.capacity(),
            loaded: resident.len(),
            evicted: self.resident.difference(&resident).count(),
            refreshed: 0,
            overflow: framed.overflow,
            rebuilt: Some(why),
            extent,
        };
        self.resident = resident;
        self.built_for = Some(framed.extent);
        self.held = extent;
        self.rebuild = false;
        self.changes += 1;
        Ok(map)
    }

    /// Brings `map` to the framed bricks and rewrites the kept ones `dirty`
    /// names. A dirty brick that is not held is left for its return, when it
    /// is made again from the source as it then stands.
    fn update(
        &mut self,
        map: &mut BrickMap,
        source: &dyn BrickSource,
        framed: &FramedBricks,
        dirty: &[[i16; 3]],
    ) -> Result<TerrainRefresh, String> {
        let spanned = span_of(&framed.keys);
        let why = if self.rebuild {
            Some(Rebuild::Requested)
        } else if self.built_for != Some(framed.extent)
            || (0..3).any(|axis| spanned[axis] > self.held[axis])
        {
            Some(Rebuild::Extent)
        } else if framed.keys.len() < self.resident.len() {
            // The hold: see the module header. After the pin bump this arm
            // goes, and a shrinking selection retargets like any other.
            Some(Rebuild::Shrink)
        } else {
            None
        };
        if let Some(why) = why {
            *map = self.build(source, framed, why)?;
            return Ok(TerrainRefresh::Full);
        }

        let mut stats = ResidencyStats {
            capacity: self.capacity(),
            overflow: framed.overflow,
            extent: self.held,
            ..ResidencyStats::default()
        };
        let mut slots = Vec::new();
        let mut incoming = Vec::new();
        if !framed.keys.iter().eq(self.resident.iter()) {
            incoming = framed
                .keys
                .iter()
                .copied()
                .filter(|key| !self.resident.contains(key))
                .collect();
            let bytes = fill(source, &incoming);
            let revision = self.next_projection();
            let delta = map
                .retarget_with(revision, framed.keys.iter().copied(), |key| {
                    brick_in(&incoming, &bytes, key)
                })
                .map_err(|error| format!("paged brick map: {error}"))?;
            stats.loaded = delta.loaded_slots.len();
            stats.evicted = delta.evicted;
            slots.extend(delta.loaded_slots);
            self.resident = framed.keys.iter().copied().collect();
        }
        let stale: Vec<[i16; 3]> = dirty
            .iter()
            .copied()
            .filter(|key| self.resident.contains(key) && incoming.binary_search(key).is_err())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !stale.is_empty() {
            let bytes = fill(source, &stale);
            let refreshed = map
                .refresh_with(stale.iter().copied(), |key| brick_in(&stale, &bytes, key))
                .map_err(|error| format!("paged brick map: {error}"))?;
            stats.refreshed = refreshed.len();
            slots.extend(refreshed);
        }
        if incoming.is_empty() && stale.is_empty() && stats.evicted == 0 {
            return Ok(TerrainRefresh::Current);
        }
        stats.resident = self.resident.len();
        self.stats = stats;
        self.changes += 1;
        slots.sort_unstable();
        slots.dedup();
        Ok(TerrainRefresh::Slots(slots))
    }
}

/// The key box the bricks span, one along any axis they do not.
pub(super) fn span_of(keys: &[[i16; 3]]) -> [u32; 3] {
    let Some(first) = keys.first() else {
        return [1; 3];
    };
    let (mut low, mut high) = (*first, *first);
    for key in keys {
        for axis in 0..3 {
            low[axis] = low[axis].min(key[axis]);
            high[axis] = high[axis].max(key[axis]);
        }
    }
    [0, 1, 2].map(|axis| (i32::from(high[axis]) - i32::from(low[axis]) + 1) as u32)
}

/// Makes the bricks `keys` names, in order, [`BRICK_BYTES`] apiece.
fn fill(source: &dyn BrickSource, keys: &[[i16; 3]]) -> Vec<u8> {
    let mut bytes = vec![0; keys.len() * BRICK_BYTES];
    for (key, out) in keys.iter().zip(bytes.chunks_exact_mut(BRICK_BYTES)) {
        source.fill(*key, out);
    }
    bytes
}

/// One brick of a [`fill`], found by its key in the sorted `keys`.
fn brick_in<'a>(keys: &[[i16; 3]], bytes: &'a [u8], key: [i16; 3]) -> Option<&'a [u8]> {
    let index = keys.binary_search(&key).ok()?;
    bytes.get(index * BRICK_BYTES..(index + 1) * BRICK_BYTES)
}

/// One frame of a paged terrain: the bricks it frames, where they are made,
/// and the residency that holds them between frames.
pub struct PagedTerrain<'a> {
    residency: &'a RefCell<Residency>,
    source: &'a dyn BrickSource,
    framed: &'a FramedBricks,
    revision: u64,
}

impl<'a> PagedTerrain<'a> {
    /// `revision` is the host's own, moved by an edit; the residency moves
    /// the projection revision itself whenever the framed bricks do.
    pub fn new(
        residency: &'a RefCell<Residency>,
        source: &'a dyn BrickSource,
        framed: &'a FramedBricks,
        revision: u64,
    ) -> Self {
        Self {
            residency,
            source,
            framed,
            revision,
        }
    }
}

impl TerrainSource for PagedTerrain<'_> {
    fn revision(&self) -> u64 {
        self.revision
    }

    fn brick_map(&self) -> Result<BrickMap, String> {
        self.residency
            .borrow_mut()
            .build(self.source, self.framed, Rebuild::First)
    }

    fn refresh(&self, map: &mut BrickMap, dirty: &[[i16; 3]]) -> Result<TerrainRefresh, String> {
        self.residency
            .borrow_mut()
            .update(map, self.source, self.framed, dirty)
    }
}
