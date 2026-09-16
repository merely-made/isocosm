//! The scene board's host half: the flag, the shared snapshot, and the one
//! producer registration (B2).
//!
//! `isometry-views` owns everything about how the board draws through the
//! shared scene. What is left for the host is what only a host can do: read
//! the environment flag once, push the board's current state into the snapshot
//! the producer reads, and choose the integer pixel grid the scene renders on.
//!
//! **Registration moved out in M4.** Hanging a producer on a leaf key is the
//! one thing every wing host did identically, so `isomere::host::Assembly`
//! does it now, from the product impl's `viewport_keys` and `producer`. This
//! module hands the producer over through [`SceneBoard::producer`] and keeps
//! the two things that are about the board rather than about hosting it.
//!
//! **The pixel grid.** Decision 2 of the plan moved integer scaling off the
//! DOM's `board_scale` and onto the producer's render scale. The scale is the
//! device scale times the interface zoom, rounded to a whole number and never
//! below one: the scene then draws at (about) the pane's *logical* size and
//! the producer presents each of those pixels as one square block, which is
//! the GBA crispness pillar expressed in the renderer rather than in CSS.
//!
//! Nothing here runs unless the flag is set. With it unset the producer is
//! never built, the leaf is never registered, and every existing receipt sees
//! the board it always saw.

use std::cell::RefCell;
use std::rc::Rc;

use isometry_views::{
    BoardHandle, BoardProducer, BoardSource, BoardView, GroundCost, ScenePick, UiState,
};

use crate::Ctx;

/// `ISOMETRY_SCENE_BOARD=1`: draw the board through the isometer scene.
pub(crate) fn enabled() -> bool {
    std::env::var_os("ISOMETRY_SCENE_BOARD").is_some()
}

/// The host's handle on the scene board: the snapshot the view layer's source
/// reads, and the producer the document leaf is served from.
pub(crate) struct SceneBoard {
    view: BoardHandle,
    producer: Rc<RefCell<BoardProducer>>,
    scale: u32,
    /// The last error printed, so a frame that keeps failing says so once
    /// rather than once per frame — and a frame that starts failing says so at
    /// all. A scene board that quietly draws nothing is the failure this
    /// guards against.
    reported: Option<String>,
    /// The last ground change reported under `ISOMETRY_PROFILE`, so a still
    /// board says nothing and an edit says what it cost (B4).
    costed: Option<GroundCost>,
}

impl SceneBoard {
    /// Builds the producer over a first snapshot of the board.
    pub(crate) fn new(ui: &UiState) -> Self {
        let mut board = BoardView::new(ui.map.clone());
        board.sync(ui);
        let view = board.into_handle();
        Self {
            view: view.clone(),
            producer: Rc::new(RefCell::new(BoardProducer::new(BoardSource::new(view)))),
            scale: 1,
            reported: None,
            costed: None,
        }
    }

    /// The producer the assembly registers against the board's leaf key.
    /// Cheap: the handle is an `Rc`, and the assembly asks only until the key
    /// is registered.
    pub(crate) fn producer(&self) -> Rc<RefCell<BoardProducer>> {
        self.producer.clone()
    }

    /// The board's gestures, handed the frame the producer last drew (B3).
    ///
    /// The view layer owns every rule about what a press means; what it cannot
    /// own is the producer, which lives in the host's registry. So the state
    /// carries this borrow of it and asks at the moment the gesture runs,
    /// rather than reading a snapshot that is a frame stale by construction.
    pub(crate) fn pick(&self) -> ScenePick {
        ScenePick::new(self.producer.clone())
    }

    /// What the last ground change cost, said once per change under
    /// `ISOMETRY_PROFILE`.
    ///
    /// Called *after* the frame rather than in [`Self::sync`], because the
    /// ground is brought up to date inside the producer's own draw: asked
    /// before it, a sync would always report the change before last, and a
    /// board that then parks would never say what the last edit cost.
    pub(crate) fn report_ground(&mut self) {
        if std::env::var_os("ISOMETRY_PROFILE").is_none() {
            return;
        }
        let cost = self.producer.borrow().source().ground_cost();
        if self.costed != cost {
            self.costed = cost;
            if let Some(cost) = cost {
                eprintln!("[isometry] scene board ground: {}", cost.line());
            }
        }
    }

    /// Per frame: take the board's state and set the pixel grid.
    pub(crate) fn sync(&mut self, ctx: &mut Ctx<'_>) {
        self.view.borrow_mut().sync(ctx.runner.state());
        let device = ctx
            .window
            .map_or(1.0, |window| window.scale_factor() as f32);
        let scale = (device * ctx.ui_zoom).round().max(1.0) as u32;
        if self.scale != scale {
            self.scale = scale;
            self.producer.borrow_mut().set_render_scale(scale);
            // Said out loud, because the grid is the one thing about this
            // board a capture cannot tell you from its pixels alone once the
            // upscale is doing its job.
            eprintln!(
                "[isometry] scene board render scale {scale} (device {device}, zoom {})",
                ctx.ui_zoom
            );
        }
        let error = self.producer.borrow().last_error().map(str::to_owned);
        if self.reported != error {
            if let Some(why) = &error {
                eprintln!("[isometry] scene board: {why}");
            }
            self.reported = error;
        }
    }
}
