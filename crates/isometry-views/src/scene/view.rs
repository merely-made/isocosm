//! The snapshot the host pushes and the producer reads.
//!
//! Split out of `board.rs` when B4 gave the board overlays: the source grew a
//! ground lifecycle of its own ([`super::ground`]) and the snapshot grew the
//! board's whole state ([`Overlays`]), and the two have nothing to say to each
//! other beyond a revision number.
//!
//! **Why a snapshot at all.** The producer is called from the host's own loop
//! and cannot borrow the runner's state while it is doing so, so the board's
//! current state is copied across once a frame. The copy is guarded: the map
//! is cloned only when one of the layers the ground is grown from has actually
//! moved, so an ordinary frame copies nothing.
//!
//! **The tallest tile** is kept here too (2026-09-26). The world's camera
//! needs it on every signature check, and scanning the grid for it cost 2.7 ms
//! a frame at 256 by 256 in the debug build. It is rescanned only when the
//! elevation layer moves, so lowering the tallest tile is found like any rise.

use std::cell::RefCell;
use std::rc::Rc;

use isometry_core::{IsoGeometry, MapDocument};

use super::overlay::Overlays;
use super::world::tallest;
use crate::state::UiState;

/// The board as the producer sees it.
pub struct BoardView {
    pub map: MapDocument,
    pub geo: IsoGeometry,
    /// Board-origin offset inside the pane, logical px.
    pub camera: (f32, f32),
    /// The board pane's logical size, logical px.
    pub pane: (f32, f32),
    /// What the board is saying about its tiles: the state tints, the fog and
    /// the focus elevation.
    pub overlays: Overlays,
    /// Bumped whenever anything the ground is grown from moves — the three
    /// terrain layers or any overlay — so the ground is regrown exactly then.
    pub(super) terrain_revision: u64,
    /// The elevation of the map's tallest tile, rescanned when the elevation
    /// layer moves and at no other time.
    tallest: u8,
}

/// The shared handle. Cheap to clone; every clone sees one snapshot.
pub type BoardHandle = Rc<RefCell<BoardView>>;

impl BoardView {
    pub fn new(map: MapDocument) -> Self {
        Self {
            tallest: tallest(&map),
            map,
            geo: IsoGeometry::default(),
            camera: (0.0, 0.0),
            pane: (0.0, 0.0),
            overlays: Overlays::default(),
            terrain_revision: 1,
        }
    }

    /// The elevation of the map's tallest tile, as of the last sync.
    pub fn tallest(&self) -> u8 {
        self.tallest
    }

    pub fn into_handle(self) -> BoardHandle {
        Rc::new(RefCell::new(self))
    }

    /// The revision the ground is grown at.
    pub fn terrain_revision(&self) -> u64 {
        self.terrain_revision
    }

    /// Takes the board's current state.
    ///
    /// The revision moves for the three layers the ground is grown from and
    /// for the overlays laid on top of it — a token step or a pan is not a
    /// regrow — and the map is cloned only when one of them differs.
    pub fn sync(&mut self, ui: &UiState) {
        self.geo = ui.geo;
        self.camera = ui.camera;
        self.pane = ui.viewport;
        let overlays = Overlays::of(ui);
        let heights_moved = self.map.elevation != ui.map.elevation;
        let terrain_moved = heights_moved
            || self.map.ground != ui.map.ground
            || self.map.tile_kinds != ui.map.tile_kinds
            || self.overlays != overlays;
        if terrain_moved || self.map.tokens != ui.map.tokens || self.map.props != ui.map.props {
            self.map = ui.map.clone();
        }
        if heights_moved {
            self.tallest = tallest(&self.map);
        }
        if terrain_moved {
            self.overlays = overlays;
            self.terrain_revision += 1;
        }
    }
}
