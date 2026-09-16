//! The board's ground: grown, diffed, filtered, and handed to the scene as a
//! terrain source (B4).
//!
//! B2 grew a whole `Ground` on every change and bound the brick map it built
//! from it with `set_terrain_map`, which is a full 258 KiB upload for a
//! one-tile edit. Two things were wrong with that beyond the cost.
//!
//! **The revision never moved.** [`isometer::GroundTerrain`] stamps a frame
//! with `Ground::revision()`, and a *grown* ground's revision is zero by
//! construction — growth is the world's starting fact, not an edit, so only
//! `carve` ever bumps it. The tracer's residency skips an upload whose
//! revision and projection both match what it holds, so a board that regrows
//! rather than carves cannot use `GroundTerrain` at all. [`BoardTerrain`] is
//! this crate's own [`TerrainSource`]: the same slot refresh, stamped with the
//! *view's* revision, which moves whenever the board does.
//!
//! **Nothing said what had changed.** [`Ground::drain_dirty`] is the seam the
//! slot upload reads, and growth clears it. So a regrow's changed bricks are
//! found by comparing the new ground's bricks against the held one's, which
//! costs one pass over 133 KiB and turns a full upload into a handful of
//! slots. Where the brick *set* moves — an edit that raises ground past a
//! brick boundary — there are no slots to refresh into, so the map is rebuilt
//! whole and says so.
//!
//! **The focus elevation** is §2's keep predicate over the grown ground,
//! rebuilt on focus change as Mesocosm's terrarium does: one
//! `BrickMap::from_ground_filtered` and one full upload per change. A filtered
//! map cannot take slot refreshes, because a slot would be refilled from the
//! unfiltered ground, so with a focus on every ground change is a rebuild.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use isometer::core::ground::Ground;
use isometer::lens::{BrickMap, BrickProjectionRevision};
use isometer::{TerrainRefresh, TerrainSource};
use isometry_core::MapDocument;

use super::overlay::Overlays;
use super::terrain::{MapTerrain, surface_of};

/// What the last ground change cost, for the profile line and the receipts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundCost {
    /// Raising the whole ground from the map. Paid on every change, because
    /// `Ground` exposes no way to write one voxel's material (see §6).
    pub grow: Duration,
    /// Comparing the new bricks against the held ones.
    pub diff: Duration,
    /// Rebuilding the brick map, when the brick set moved or a focus is on.
    pub rebuild: Duration,
    /// Bricks the change touched, or `None` where the map was rebuilt whole.
    pub slots: Option<usize>,
    /// Bricks the ground holds.
    pub bricks: usize,
}

impl GroundCost {
    /// One line for the profile, in the shape the other board timers print in.
    pub fn line(&self) -> String {
        let what = match self.slots {
            Some(slots) => format!("{slots} slots"),
            None => "whole map".to_owned(),
        };
        format!(
            "grow {:.2} ms, diff {:.2} ms, rebuild {:.2} ms, {what} of {} bricks",
            self.grow.as_secs_f32() * 1e3,
            self.diff.as_secs_f32() * 1e3,
            self.rebuild.as_secs_f32() * 1e3,
            self.bricks,
        )
    }
}

/// The grown ground, the change awaiting upload, and what it cost.
pub struct BoardGround {
    ground: Ground,
    /// The board revision the ground was grown from.
    revision: u64,
    /// The focus elevation the bound map was filtered at.
    focus: Option<i32>,
    /// Bricks changed since the last successful frame.
    dirty: Vec<[i16; 3]>,
    /// A map this crate built itself, awaiting the scene's next refresh.
    rebuilt: RefCell<Option<BrickMap>>,
    /// Bumped per wholesale rebuild, so the tracer never mistakes a rebuilt
    /// map for the one it holds.
    projection: u64,
    cost: GroundCost,
}

impl BoardGround {
    /// Raises the ground for a board and binds its first map.
    pub fn new(map: &MapDocument, overlays: &Overlays, revision: u64) -> Result<Self, String> {
        let started = Instant::now();
        let ground = MapTerrain::with_overlays(map, Some(overlays)).grow();
        let grow = started.elapsed();
        let mut board = Self {
            cost: GroundCost {
                grow,
                bricks: ground.brick_count(),
                ..GroundCost::default()
            },
            ground,
            revision,
            focus: overlays.focus,
            dirty: Vec::new(),
            rebuilt: RefCell::new(None),
            projection: 0,
        };
        board.rebuild()?;
        Ok(board)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn ground(&self) -> &Ground {
        &self.ground
    }

    pub fn cost(&self) -> GroundCost {
        self.cost
    }

    /// The bricks the next frame must upload.
    pub fn dirty(&self) -> &[[i16; 3]] {
        &self.dirty
    }

    /// The terrain source for one frame.
    pub fn terrain(&self) -> BoardTerrain<'_> {
        BoardTerrain {
            ground: &self.ground,
            revision: self.revision,
            rebuilt: &self.rebuilt,
        }
    }

    /// A frame reached the screen: the change it carried is uploaded.
    pub fn uploaded(&mut self) {
        self.dirty.clear();
    }

    /// Brings the ground up to `revision`, regrowing and diffing only when the
    /// board has actually moved. A focus change alone re-filters the map
    /// without touching the ground.
    pub fn sync(
        &mut self,
        map: &MapDocument,
        overlays: &Overlays,
        revision: u64,
    ) -> Result<(), String> {
        let focus_moved = self.focus != overlays.focus;
        if self.revision == revision && !focus_moved {
            return Ok(());
        }
        self.focus = overlays.focus;
        if self.revision == revision {
            // The ground stands; only what is kept of it changed.
            self.cost = GroundCost {
                bricks: self.ground.brick_count(),
                ..GroundCost::default()
            };
            return self.rebuild();
        }
        self.revision = revision;
        let started = Instant::now();
        let grown = MapTerrain::with_overlays(map, Some(overlays)).grow();
        let grow = started.elapsed();

        let started = Instant::now();
        let change = compare(&self.ground, &grown);
        let diff = started.elapsed();
        self.ground = grown;
        self.cost = GroundCost {
            grow,
            diff,
            bricks: self.ground.brick_count(),
            ..GroundCost::default()
        };

        match change {
            // A filtered map holds no slot a ground brick could be refreshed
            // into, so a focus turns every change into a rebuild — and so does
            // *leaving* one, whose bound map is still the filtered one however
            // little the ground itself moved.
            Change::Slots(keys) if self.focus.is_none() && !focus_moved => {
                self.cost.slots = Some(keys.len());
                self.dirty = keys;
                Ok(())
            },
            _ => self.rebuild(),
        }
    }

    /// Builds the map the next frame binds: the whole ground, or what a focus
    /// elevation keeps of it.
    fn rebuild(&mut self) -> Result<(), String> {
        let started = Instant::now();
        self.projection += 1;
        let revision = BrickProjectionRevision(self.projection);
        let map = match self.focus {
            None => BrickMap::from_ground_keys(&self.ground, revision, self.ground.keys()),
            Some(focus) => {
                let ceiling = surface_of(focus);
                BrickMap::from_ground_filtered(&self.ground, revision, |at, _| at[1] <= ceiling)
            },
        }
        .map_err(|error| format!("brick map: {error}"))?;
        self.cost.rebuild = started.elapsed();
        self.dirty.clear();
        *self.rebuilt.borrow_mut() = Some(map);
        Ok(())
    }
}

/// What one regrow did to the brick set.
enum Change {
    /// These bricks hold different bytes; every one of them already has a slot.
    Slots(Vec<[i16; 3]>),
    /// The brick set itself moved, so the map has no slot to refresh into.
    Structure,
}

/// The bricks that differ between two grounds.
///
/// `Ground::drain_dirty` would say this for an *edited* ground; a regrown one
/// clears it, so the comparison stands in. Whole-brick byte equality, which is
/// 512 bytes per brick and 133 KiB for the demo board.
fn compare(before: &Ground, after: &Ground) -> Change {
    let held: BTreeSet<[i16; 3]> = before.keys().collect();
    let grown: BTreeSet<[i16; 3]> = after.keys().collect();
    if held != grown {
        return Change::Structure;
    }
    let raw = |ground: &Ground, key| {
        ground
            .brick_materials(key)
            .map(|(brick, _)| brick.raw().to_vec())
    };
    Change::Slots(
        held.into_iter()
            .filter(|key| raw(before, *key) != raw(after, *key))
            .collect(),
    )
}

/// The board's ground as a scene terrain: slot refreshes stamped with the
/// board's own revision, plus the wholesale map a focus change hands over.
pub struct BoardTerrain<'a> {
    ground: &'a Ground,
    revision: u64,
    rebuilt: &'a RefCell<Option<BrickMap>>,
}

impl TerrainSource for BoardTerrain<'_> {
    /// The **view's** revision, not the ground's. See the module header: a
    /// grown ground's own revision never moves, and the tracer would skip
    /// every upload after the first.
    fn revision(&self) -> u64 {
        self.revision
    }

    fn brick_map(&self) -> Result<BrickMap, String> {
        self.rebuilt
            .borrow_mut()
            .take()
            .ok_or_else(|| "the board built no brick map".to_string())
    }

    fn refresh(&self, map: &mut BrickMap, dirty: &[[i16; 3]]) -> Result<TerrainRefresh, String> {
        if let Some(rebuilt) = self.rebuilt.borrow_mut().take() {
            *map = rebuilt;
            return Ok(TerrainRefresh::Full);
        }
        if dirty.is_empty() {
            return Ok(TerrainRefresh::Current);
        }
        map.refresh(self.ground, dirty.iter().copied())
            .map(TerrainRefresh::Slots)
            .map_err(|error| error.to_string())
    }
}
