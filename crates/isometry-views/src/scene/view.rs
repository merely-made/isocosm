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

use std::collections::BTreeMap;

use isometry_core::{IsoGeometry, MapDocument, Token, TokenId};

use crate::appearance::{Library, LookInput};

use super::overlay::Overlays;
use super::world::tallest;
use crate::state::UiState;

/// The board as the producer sees it.
pub struct BoardView {
    pub(super) native: Option<Result<Rc<super::board::native::Picture>,String>>,
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
    /// Each token's look as the table holds it; a token missing here is its
    /// bare class.
    pub looks: BTreeMap<TokenId, LookInput>,
    /// The bindings looks resolve through.
    pub library: Rc<Library>,
}

/// The shared handle. Cheap to clone; every clone sees one snapshot.
pub type BoardHandle = Rc<RefCell<BoardView>>;

impl BoardView {
    pub fn new(map: MapDocument) -> Self {
        Self {
            native: None,
            tallest: tallest(&map),
            map,
            geo: IsoGeometry::default(),
            camera: (0.0, 0.0),
            pane: (0.0, 0.0),
            overlays: Overlays::default(),
            terrain_revision: 1,
            looks: BTreeMap::new(),
            library: Rc::new(Library::core()),
        }
    }

    /// Takes the look editor's draft as a one-tile board: the same source,
    /// camera and bodies as the board, so the preview is what the board will
    /// draw (ruling 1028: the preview follows the active board). `pane` is
    /// the preview leaf's box.
    pub fn sync_preview(&mut self, ui: &UiState, pane: (f32, f32)) {
        let draft = &ui.look;
        let id = TokenId(1);
        let mut map = MapDocument::new("look preview", 1, 1);
        let stone = map.intern_tile_kind("stone");
        map.ground.set(0, 0, stone);
        map.tokens.push(Token {
            id,
            at: (0, 0),
            facing: draft.facing,
            sprite: draft.sprite.clone(),
            owner: None,
        });
        if !draft.appearance.is_empty() {
            map.appearances.insert(id, draft.appearance.clone());
        }
        if self.map != map {
            self.map = map;
            self.tallest = 0;
            self.terrain_revision += 1;
        }
        let looks = BTreeMap::from([(id, draft.input())]);
        if self.looks != looks {
            self.looks = looks;
        }
        if !Rc::ptr_eq(&self.library, &ui.appearance) {
            self.library = ui.appearance.clone();
        }
        // Two and a half times the board's tile, the tile's centre low in the
        // box so the figure stands in it.
        let base = IsoGeometry::default();
        self.geo = IsoGeometry {
            tile_w: base.tile_w * 2.5,
            tile_h: base.tile_h * 2.5,
            elev_step: base.elev_step * 2.5,
        };
        self.pane = pane;
        self.camera = (pane.0 / 2.0, pane.1 * 0.82);
        self.native = None;
    }

    /// `token`'s look, falling back to its bare class.
    pub fn look(&self, token: &Token) -> LookInput {
        self.looks
            .get(&token.id)
            .cloned()
            .unwrap_or_else(|| LookInput::class(&token.sprite))
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
        self.native=ui.native.world.as_ref().map(|_|super::board::native::read(ui));
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
        let looks: BTreeMap<TokenId, LookInput> = ui
            .map
            .tokens
            .iter()
            .map(|token| (token.id, LookInput::of(ui, token)))
            .filter(|(_, look)| !look.is_bare())
            .collect();
        if self.looks != looks {
            self.looks = looks;
        }
        if !Rc::ptr_eq(&self.library, &ui.appearance) {
            self.library = ui.appearance.clone();
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
