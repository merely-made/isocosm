//! The pointer's gestures: what a press, a click, a drag and a hover resolve
//! to, and where on the board they land.
//!
//! Split out of `interaction.rs` on 2026-09-15, along the seam that file
//! already had: the gestures above, and below them the editor verbs a gesture
//! ends in ([`UiState::click_tile`], the turn list, undo). The boundary is
//! "pointer-facing" — everything here takes a pane-local point or reports one
//! — and B3 is what made it worth drawing, because resolving a point is now a
//! real piece of work rather than one call to the inverse projection.
//!
//! **How a point becomes a tile (B3).** There is one resolver,
//! [`UiState::board_at`], and two arms:
//!
//! - **Scene board.** [`ScenePick`](crate::scene::ScenePick) asks the frame
//!   that was actually drawn: a ray against real bricks and real bodies. A
//!   token names itself, a raised tile names the tile you can see, and a cliff
//!   face names the step it cuts through.
//! - **DOM board.** The flat-ground inverse §1 of the board plan calls a
//!   geometric fallback, plus a token lookup on the tile it lands on. It is
//!   wrong on raised ground by construction — it names the tile *behind* the
//!   one you clicked — and it is reached only with the flag off. B5 retires it
//!   with the DOM board.
//!
//! The two agreed on flat ground before the fallback stopped being reachable:
//! `scene::parity_tests` walks a probe grid over the demo map and makes both
//! arms answer, and the plan's B2 entry records the numbers.
//!
//! **What B3 did not change.** Not one rule about what a click *means*. The
//! nine editor modes, the session's local and remote branches, the reach gate
//! and the turn order are exactly what they were; only the line that finds the
//! tile moved.

use super::*;

impl UiState {
    /// What the board shows at a pane-local point.
    ///
    /// The one place a pointer position becomes board identity. See the module
    /// header for the two arms.
    pub fn board_at(&self, pane: (f32, f32)) -> Option<BoardPick> {
        if self.scene_board {
            return self.board_pick.as_ref()?.at(pane, self.viewport);
        }
        self.geometric_pick(pane)
    }

    /// The tile a pane-local point names, whichever path answered.
    ///
    /// A token pick answers with the tile the token stands on, because that is
    /// what the DOM board's inverse reports there and every caller below wants
    /// ground: a paint-drag paints under a token, and a token drag releases
    /// onto the tile it was dropped on.
    pub fn tile_at(&self, pane: (f32, f32)) -> Option<TileCoord> {
        self.tile_of(self.board_at(pane)?)
    }

    /// The DOM board's geometric fallback: the inverse projection on the plane
    /// flat ground presents, plus whatever token stands on the tile it lands
    /// on. Flat-ground picking, so a raised tile's top face resolves to the
    /// tile behind it.
    ///
    /// Private, and reached only with the scene flag off. The two public
    /// fallbacks it replaced — `tile_at_pane` and `token_drag_candidate` —
    /// were retired by B3: the mode gate the second carried was policy and
    /// moved to [`UiState::board_press`], and the arithmetic is the four lines
    /// below.
    fn geometric_pick(&self, pane: (f32, f32)) -> Option<BoardPick> {
        let x = pane.0 - self.camera.0;
        let y = pane.1 - self.camera.1;
        let at = self.geo.screen_to_tile((x, y));
        if !self.map.ground.in_bounds(at.0, at.1) {
            return None;
        }
        Some(match self.token_at(at) {
            Some(id) => BoardPick::Token(id),
            None => BoardPick::Tile {
                at,
                elevation: *self
                    .map
                    .elevation
                    .get(at.0 as u32, at.1 as u32)
                    .unwrap_or(&0) as i32,
                top: true,
            },
        })
    }

    /// A primary press in the board pane.
    ///
    /// **The DOM board.** The tile or token element under the pointer has
    /// already had its own click dispatched by the time this runs — the host
    /// routes the click first, then the pointer-down that begins the drag — so
    /// this is only the gesture's bookkeeping: dismiss an open menu, note what
    /// was grabbed, and mark the tile the press itself already applied to.
    ///
    /// **The scene board.** There is no element to dispatch that click, so the
    /// press does it, in the same order: the click first, then the same
    /// bookkeeping. That order is load-bearing — `drag_tile` records the tile
    /// the press already applied to, so a drag does not paint it twice.
    pub fn board_press(&mut self, pane: (f32, f32)) {
        // A press off the menu dismisses it. Since 2026-09-03 the menu's
        // `overlay_surface` takes that press first — its dismissal layer covers
        // the whole window, which is the only way a press on the *side panel*
        // can reach it — so this rarely runs. It stays because it costs a
        // comparison and it is still the right answer if a press ever reaches
        // the pane with a menu open.
        if self.context_menu.is_some() {
            self.close_context_menu();
        }
        let pick = self.board_at(pane);
        if self.scene_board {
            self.scene_click(pick);
        }
        // Free-move is Select mode's alone; Play movement stays the gated
        // click-a-reach-tile path. The gate lived inside the retired
        // `token_drag_candidate`; it is policy, so it lives with the gesture.
        self.drag_token = match pick {
            Some(BoardPick::Token(id)) if self.mode == EditMode::Select => Some(id),
            _ => None,
        };
        self.drag_tile = pick.and_then(|pick| self.tile_of(pick));
    }

    /// The click the scene board's leaf cannot dispatch for itself, in the two
    /// forms the DOM board's elements dispatch: a token element's handler (the
    /// target-pick branch included) and a tile element's.
    fn scene_click(&mut self, pick: Option<BoardPick>) {
        match pick {
            Some(BoardPick::Token(id)) => {
                // In target-pick mode a click on a token names the victim
                // rather than selecting it — `board/tokens.rs`'s own branch.
                if self.picking_target() {
                    self.pick_action_target(id);
                } else {
                    self.click_token(id);
                }
            },
            Some(BoardPick::Tile { at, .. }) => self.click_tile(at),
            None => {},
        }
    }

    /// The tile a pick stands on, without borrowing `self` mutably.
    fn tile_of(&self, pick: BoardPick) -> Option<TileCoord> {
        match pick {
            BoardPick::Tile { at, .. } => Some(at),
            BoardPick::Token(id) => self.map.token(id).map(|token| token.at),
        }
    }

    /// A pointer move while the primary button is held on the board.
    ///
    /// Drag painting: in a paint-capable mode, entering a new tile applies the
    /// brush there. One application per tile crossing, not one per pixel —
    /// hence [`drag_tile`](Self::drag_tile), which the press already seeded
    /// with the tile its own click applied to.
    ///
    /// It also carries the hover on the scene board, because a captured move
    /// is the only pointer motion the shared host routes into the tree at all
    /// (see [`hover_tile_enter`](Self::hover_tile_enter)).
    ///
    /// The side panel cannot be drag-painted, by construction rather than by a
    /// coordinate test: this handler hangs off the board pane, so a drag that
    /// wanders onto the panel is still routed here (the press captured the
    /// pointer) and simply reports a tile off the map, if any.
    pub fn board_drag(&mut self, pane: (f32, f32)) {
        // With neither a hover to carry nor a brush to apply there is nothing
        // to resolve, and the DOM board's drag costs exactly what it did.
        if !self.scene_board && !self.mode.drags() {
            return;
        }
        let at = self.tile_at(pane);
        if self.scene_board {
            self.hover_tile_enter(at);
            if !self.mode.drags() {
                return;
            }
        }
        let Some(at) = at else {
            return;
        };
        if self.drag_tile == Some(at) {
            return;
        }
        self.drag_tile = Some(at);
        self.click_tile(at);
    }

    /// The primary button came up on the board: finish a token drag.
    pub fn board_release(&mut self, pane: (f32, f32)) {
        self.drag_tile = None;
        let Some(id) = self.drag_token.take() else {
            return;
        };
        let Some(from) = self.map.token(id).map(|t| t.at) else {
            return;
        };
        let Some(to) = self.tile_at(pane) else {
            return;
        };
        if to != from {
            self.drag_move_token(id, to);
        }
    }

    /// A secondary press in the board pane: a token under the pointer opens
    /// its context menu, anchored where the press landed.
    ///
    /// "Under the pointer" is both readings, so the verb is the DOM board's
    /// exactly: the token's own body, and the tile it stands on. The scene
    /// pick answers the first directly and the second through the tile, which
    /// is all the DOM's inverse could ever say.
    ///
    /// The host routes one `Down` marked
    /// [`PointerButton::Secondary`](cambium::PointerButton::Secondary) and
    /// nothing after it — a right press captures nothing and dispatches no
    /// click — so the press is the whole gesture.
    pub fn board_context_menu(&mut self, pane: (f32, f32)) {
        let Some(pick) = self.board_at(pane) else {
            return;
        };
        let id = match pick {
            BoardPick::Token(id) => Some(id),
            BoardPick::Tile { at, .. } => self.token_at(at),
        };
        let Some(id) = id else { return };
        self.open_context_menu(id, pane);
    }

    /// A wheel notch over the board pane snap-pans the board (wheel = pan,
    /// the tactics-canvas convention). Over the side panel it never arrives:
    /// the handler is on the pane, so the panel keeps the host's own scrolling
    /// default.
    ///
    /// `dx`/`dy` are logical pixels in the direction the content moves, which
    /// is what the host hands a wheel handler; a notch is
    /// [`WHEEL_NOTCH_PX`](crate::state::WHEEL_NOTCH_PX) of them.
    pub fn board_wheel(&mut self, dx: f32, dy: f32) {
        let per_px = WHEEL_BOARD_TILES / WHEEL_NOTCH_PX;
        self.pan_tiles(dx * per_px, dy * per_px);
    }

    /// The pointer entered the element standing on `at` (or left the board,
    /// for `None`): show authored-site hints and move the play-mode path
    /// preview or measure template. Hidden tiles are never surfaced.
    ///
    /// **The DOM board.** The host routes `on_hover` Enter and Leave as the
    /// *hit element* changes, and deliberately routes no Move, so the
    /// granularity comes from the tree: every board element that stands on a
    /// tile carries this, and a crossing is one Leave plus one Enter.
    ///
    /// **The scene board.** The tree cannot supply that granularity — the
    /// board is one leaf, so the hit element never changes inside it — and the
    /// shared host routes neither a hover Move nor a cursor position an
    /// application could read. So the two motions that do reach here are a
    /// captured drag ([`board_drag`](Self::board_drag)) and leaving the leaf,
    /// and free hover with no button held waits on the host. Recorded for the
    /// plan's §6.
    ///
    /// The gate is the same one the desktop host used to apply before paying
    /// for a state update — Play mode with a reach highlight showing, or
    /// Measure mode with an anchor set — so an ordinary hover still rebuilds
    /// nothing.
    pub fn hover_tile_enter(&mut self, at: Option<TileCoord>) {
        if self.hover_tile == at {
            return;
        }
        if let Some(at) = at {
            if self.fog_level(at) == FogLevel::Hidden {
                return;
            }
            if let Some((label, _, _)) = self.authored_site_labels_at(at) {
                if self.status != label {
                    self.status = label;
                }
            }
        }
        let play = self.mode == EditMode::Play && !self.reach.is_empty();
        let measure = self.mode == EditMode::Measure && self.measure_anchor.is_some();
        if play || measure {
            self.hover_tile = at;
        }
    }
}
