// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One capacity-fixed brick map that follows the framed bricks.

use super::*;

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
