// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One capacity-fixed brick map that follows the framed bricks.

use super::*;

/// How a residency sizes its pointer volume; the host's to choose. The
/// atlas is the card's, given to [`Residency::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResidencySettings {
    /// Spare brick layers the pointer volume keeps above the terrain's
    /// tallest point. An edit that lifts it no further than this retargets;
    /// one past it rebuilds the map whole. Each layer costs the volume one more
    /// layer of `x * z` pointers, four bytes apiece and re-uploaded with every
    /// retarget, and each traced ray up to a brick's height of empty steps.
    pub headroom: u32,
}

impl Default for ResidencySettings {
    /// One spare layer, provisionally: a first guess, taken back to Mark with
    /// what each layer costs.
    fn default() -> Self {
        Self { headroom: 1 }
    }
}

/// Why a residency replaced its map whole rather than retargeting it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rebuild {
    /// The first map, or one the scene lost.
    First,
    /// The host asked: something about every brick changed at once, or the
    /// settings moved.
    Requested,
    /// The camera's size moved the pointer extent, or the framed bricks
    /// outgrew it.
    Extent,
    /// The terrain rose past the layers the pointer volume reserved for it.
    Headroom,
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
    /// The lowest and highest brick layer that extent holds: the terrain's,
    /// and the headroom above it.
    pub reserved: [i16; 2],
}

/// What a map's pointer volume was built to hold.
#[derive(Clone, Copy, Debug)]
struct Reserve {
    /// The terrain's layers and the headroom above them.
    layers: [i16; 2],
    /// The extent those layers take under the camera the map was built for;
    /// a camera of another size gives another.
    sized: [u32; 3],
    /// The extent the map was built at: `sized`, widened to the framed keys.
    held: [u32; 3],
}

/// A paged terrain's standing state: which bricks its map holds, the
/// revisions it has handed out, and what the last change did. The map itself
/// is the scene's.
#[derive(Debug)]
pub struct Residency {
    settings: ResidencySettings,
    limits: AtlasLimits,
    resident: BTreeSet<[i16; 3]>,
    projection: u64,
    /// What the bound map was sized for; `None` when it was built over no
    /// terrain at all.
    reserve: Option<Reserve>,
    rebuild: bool,
    stats: ResidencyStats,
    changes: u64,
}

impl Residency {
    /// A residency whose atlas holds every brick `limits` allow, its pointer
    /// volume sized by `settings`. A tracer fills the limits from its device:
    /// [`Scene::atlas_limits`](crate::Scene::atlas_limits).
    pub fn new(settings: ResidencySettings, limits: AtlasLimits) -> Self {
        Self {
            settings,
            limits,
            resident: BTreeSet::new(),
            projection: 0,
            reserve: None,
            rebuild: false,
            stats: ResidencyStats::default(),
            changes: 0,
        }
    }

    pub fn settings(&self) -> ResidencySettings {
        self.settings
    }

    /// New settings rebuild the map at them on the next frame.
    pub fn set_settings(&mut self, settings: ResidencySettings) {
        if settings != self.settings {
            self.settings = settings;
            self.rebuild = true;
        }
    }

    /// The card the atlas is sized to.
    pub fn limits(&self) -> AtlasLimits {
        self.limits
    }

    /// Bricks the atlas holds: every slot the limits allow but the reserved
    /// air slot.
    pub fn capacity(&self) -> usize {
        self.limits.max_bricks()
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

    /// A fresh map at a fresh extent, holding the framed bricks, its pointer
    /// volume reserving the headroom above the terrain.
    fn build(
        &mut self,
        source: &dyn BrickSource,
        framed: &FramedBricks,
        why: Rebuild,
    ) -> Result<BrickMap, String> {
        let headroom = i16::try_from(self.settings.headroom).unwrap_or(i16::MAX);
        let reserve = framed.layers().map(|[low, top]| {
            let layers = [low, top.saturating_add(headroom)];
            let sized = framed.extent(layers);
            let spanned = span_of(&framed.keys);
            let held = [0, 1, 2].map(|axis| sized[axis].max(spanned[axis]));
            Reserve {
                layers,
                sized,
                held,
            }
        });
        let extent = reserve.map_or([1; 3], |reserve| reserve.held);
        let start = self.next_projection();
        let mut map = BrickMap::with_limits(start, self.capacity(), extent, self.limits)
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
            reserved: reserve.map_or([0; 2], |reserve| reserve.layers),
        };
        self.resident = resident;
        self.reserve = reserve;
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
        let why = if self.rebuild {
            Some(Rebuild::Requested)
        } else {
            self.outgrown(framed)
        };
        if let Some(why) = why {
            *map = self.build(source, framed, why)?;
            return Ok(TerrainRefresh::Full);
        }

        let (extent, reserved) = self
            .reserve
            .map_or(([1; 3], [0; 2]), |reserve| (reserve.held, reserve.layers));
        let mut stats = ResidencyStats {
            capacity: self.capacity(),
            overflow: framed.overflow,
            extent,
            reserved,
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

    /// Why the bound map can no longer hold this framing, if it cannot: the
    /// terrain rose past the reserved layers, the camera changed size, or the
    /// framed bricks spread wider than the volume. Lowering the terrain never
    /// rebuilds; the reserve holds more headroom until the next build.
    fn outgrown(&self, framed: &FramedBricks) -> Option<Rebuild> {
        let layers = framed.layers()?;
        let Some(reserve) = self.reserve else {
            return Some(Rebuild::Headroom);
        };
        if layers[0] < reserve.layers[0] || layers[1] > reserve.layers[1] {
            return Some(Rebuild::Headroom);
        }
        let spanned = span_of(&framed.keys);
        let camera_moved = framed.extent(reserve.layers) != reserve.sized;
        (camera_moved || (0..3).any(|axis| spanned[axis] > reserve.held[axis]))
            .then_some(Rebuild::Extent)
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
    for (key, out) in keys.iter().zip(bytes.as_chunks_mut::<BRICK_BYTES>().0) {
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
