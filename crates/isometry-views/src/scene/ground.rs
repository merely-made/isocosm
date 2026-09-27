//! The board's ground: one column per tile, diffed per tile, and paged into the
//! scene by what the frame shows (2026-09-26).
//!
//! **What this replaced.** B4 raised a whole `Ground` from the map on every
//! change, diffed it against the held one and uploaded the bricks that
//! differed; B5 measured that regrow as the scene board's whole weight, and a
//! board past `modulus::MAX_BRICKS` (2,047) refused its brick map outright.
//! Now [`TileColumns`] reads the map as one record per tile, a change is the
//! tiles whose records moved, and the scene holds only the bricks the frame
//! shows, made from the records on demand.
//!
//! **Which bricks.** Mark's ruling of 2026-09-26: every brick whose screen box
//! overlaps the pane grown by one brick, a pure function of the view and the
//! map. That is [`isometer::framed_bricks`] with [`RESIDENCY_MARGIN`], over
//! the residency settings the host chooses: every atlas row the pinned modulus
//! allows (2,047 bricks until the family's atlas is sized to the card) and one
//! spare brick layer of headroom above the board's tallest tile, so an edit
//! that lifts it one layer retargets rather than rebuilding. The one layer is
//! provisional, with what each costs in `testing/scene-board-paging/`. Retargeting, refreshing and rebuilding are
//! isometer's [`Residency`].
//!
//! **The revision.** The tracer skips an upload whose revision and projection
//! both match what it holds, so a frame is stamped with the *view's* revision,
//! which moves on every edit and overlay. B4 found why: a grown `Ground`'s own
//! revision never moves off zero, so `GroundTerrain` skipped every edit. The
//! residency moves the projection revision itself when the framed bricks do.
//!
//! **A focus elevation** is a ceiling on the columns. A new ceiling touches
//! every brick at once, so the map is rebuilt whole rather than refreshed.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use isometer::core::ground::BRICK;
use isometer::{
    FramedBricks, PagedTerrain, Residency, ResidencySettings, ResidencyStats, SlabCamera,
    framed_bricks,
};
use isometry_core::MapDocument;

use super::columns::{BoardBricks, TileColumns};
use super::overlay::Overlays;
use super::terrain::surface_of;

/// The ruled margin around the pane: one brick edge in world units, which is
/// 36 px at the shipped 32 by 16 tile.
pub const RESIDENCY_MARGIN: f32 = BRICK as f32;

/// What the last change to the board's ground cost, for the profile line and
/// the receipts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GroundCost {
    /// Reading the map into one column per tile, where the regrow stood.
    pub columns: Duration,
    /// Comparing the new columns with the held ones.
    pub diff: Duration,
    /// Tiles the change touched; `None` for a new board.
    pub tiles: Option<usize>,
    /// Making the bricks the frame loaded or refreshed.
    pub fill: Duration,
    /// Slots the tracer was handed; `None` where the map was rebuilt whole.
    pub slots: Option<usize>,
    /// The bricks the scene holds after the change.
    pub residency: ResidencyStats,
}

impl GroundCost {
    /// One line for the profile, in the shape the other board timers print in.
    pub fn line(&self) -> String {
        let ms = |time: Duration| time.as_secs_f32() * 1e3;
        let tiles = match self.tiles {
            Some(tiles) => format!("{tiles} tiles"),
            None => "a new board".to_owned(),
        };
        let upload = match (self.slots, self.residency.rebuilt) {
            (Some(slots), _) => format!("{slots} slots"),
            (None, Some(why)) => format!("whole map ({})", format!("{why:?}").to_lowercase()),
            (None, None) => "whole map".to_owned(),
        };
        let held = self.residency;
        format!(
            "columns {:.2} ms, diff {:.2} ms, {tiles}; fill {:.2} ms, {upload}; resident {} of {} \
             (+{} -{}, {} refreshed, {} past the atlas)",
            ms(self.columns),
            ms(self.diff),
            ms(self.fill),
            held.resident,
            held.capacity,
            held.loaded,
            held.evicted,
            held.refreshed,
            held.overflow,
        )
    }
}

/// A change read before the frame, waiting for the frame that uploads it.
#[derive(Clone, Copy, Debug, Default)]
struct Pending {
    columns: Duration,
    diff: Duration,
    tiles: Option<usize>,
}

/// The board's ground: its columns, the bricks a change touched, and the
/// residency the scene's brick map follows.
pub struct BoardGround {
    columns: TileColumns,
    /// The board revision the columns were read at.
    revision: u64,
    /// The focus elevation the columns are cut at.
    focus: Option<i32>,
    /// Bricks changed since the last frame reached the screen, sorted.
    dirty: Vec<[i16; 3]>,
    residency: RefCell<Residency>,
    /// The residency's change count when the cost was last taken.
    seen: u64,
    pending: Option<Pending>,
    cost: GroundCost,
}

impl BoardGround {
    /// Reads a board's columns. Its bricks are made when a frame asks, into a
    /// map sized by `settings`.
    pub fn new(
        map: &MapDocument,
        overlays: &Overlays,
        revision: u64,
        settings: ResidencySettings,
    ) -> Self {
        let started = Instant::now();
        let columns = TileColumns::of(map, overlays);
        Self {
            pending: Some(Pending {
                columns: started.elapsed(),
                ..Pending::default()
            }),
            columns,
            revision,
            focus: overlays.focus,
            dirty: Vec::new(),
            residency: RefCell::new(Residency::new(settings)),
            seen: 0,
            cost: GroundCost::default(),
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn cost(&self) -> GroundCost {
        self.cost
    }

    /// New residency settings, which rebuild the map at them on the next
    /// frame.
    pub fn set_residency(&mut self, settings: ResidencySettings) {
        self.residency.borrow_mut().set_settings(settings);
    }

    /// Bricks the atlas holds.
    pub fn capacity(&self) -> usize {
        self.residency.borrow().capacity()
    }

    /// The bricks the scene's map holds.
    pub fn resident(&self) -> BTreeSet<[i16; 3]> {
        self.residency.borrow().resident().clone()
    }

    /// The bricks the next frame must remake, where it holds them.
    pub fn dirty(&self) -> &[[i16; 3]] {
        &self.dirty
    }

    /// The columns as a brick source, cut at the focus.
    pub fn bricks(&self) -> BoardBricks<'_> {
        BoardBricks::new(&self.columns, self.focus.map(surface_of))
    }

    /// The board's rule: every brick whose screen box overlaps the pane grown
    /// by [`RESIDENCY_MARGIN`], as many as the atlas holds.
    pub fn framed(&self, camera: SlabCamera, bricks: &BoardBricks<'_>) -> FramedBricks {
        framed_bricks(camera, RESIDENCY_MARGIN, bricks, self.capacity())
    }

    /// The terrain source for one frame.
    pub fn terrain<'a>(
        &'a self,
        bricks: &'a BoardBricks<'a>,
        framed: &'a FramedBricks,
    ) -> PagedTerrain<'a> {
        PagedTerrain::new(&self.residency, bricks, framed, self.revision)
    }

    /// Brings the columns up to `revision`, reading and diffing only when the
    /// board has actually moved. A change of board, or of focus, asks the
    /// residency for a whole new map.
    pub fn sync(&mut self, map: &MapDocument, overlays: &Overlays, revision: u64) {
        let focus_moved = self.focus != overlays.focus;
        if self.revision == revision && !focus_moved {
            return;
        }
        let started = Instant::now();
        let columns = TileColumns::of(map, overlays);
        let read = started.elapsed();
        let started = Instant::now();
        let change = columns.changed(&self.columns);
        let diff = started.elapsed();
        let tiles = match change {
            Some(change) => {
                let mut dirty: BTreeSet<_> = self.dirty.iter().copied().collect();
                dirty.extend(change.bricks);
                self.dirty = dirty.into_iter().collect();
                Some(change.tiles)
            },
            None => {
                self.dirty.clear();
                self.residency.borrow_mut().rebuild();
                None
            },
        };
        if focus_moved {
            self.residency.borrow_mut().rebuild();
        }
        self.columns = columns;
        self.revision = revision;
        self.focus = overlays.focus;
        self.pending = Some(Pending {
            columns: read,
            diff,
            tiles,
        });
    }

    /// A frame reached the screen: the change it carried is uploaded, and
    /// what it cost, if it changed anything, is taken. `fill` is the time the
    /// frame spent making bricks.
    pub fn uploaded(&mut self, fill: Duration) {
        self.dirty.clear();
        let residency = self.residency.borrow();
        let moved = residency.changes() != self.seen;
        self.seen = residency.changes();
        if !moved && self.pending.is_none() {
            return;
        }
        // Without a pending read the change was the camera's: no tile moved.
        let pending = self.pending.take().unwrap_or(Pending {
            tiles: Some(0),
            ..Pending::default()
        });
        let last = residency.stats();
        let (slots, stats) = if moved {
            let slots = last
                .rebuilt
                .is_none()
                .then_some(last.loaded + last.refreshed);
            (slots, last)
        } else {
            // An edit the frame holds no brick of: nothing moved residency.
            let still = ResidencyStats {
                resident: last.resident,
                capacity: last.capacity,
                overflow: last.overflow,
                extent: last.extent,
                ..ResidencyStats::default()
            };
            (Some(0), still)
        };
        self.cost = GroundCost {
            columns: pending.columns,
            diff: pending.diff,
            tiles: pending.tiles,
            fill,
            slots,
            residency: stats,
        };
    }
}
