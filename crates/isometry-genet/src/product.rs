//! Isometry's product impl over the shared host assembly (M4 of the isomere
//! plan).
//!
//! What used to be here was `hooks::hooks` — seven closures over one
//! `Rc<RefCell<App>>`, five of which did something two other wing hosts also
//! did by hand. `isomere::host::Assembly` fills those five once: registering
//! the viewport producer, ticking a scenario lane, arming the bounded capture,
//! publishing the error line, and printing the frame profile. What is left is
//! below, and it is all about *this* host.
//!
//! **The capture came from here.** `capture.rs` was Isometry's, and M4
//! promoted it to `isomere::host::Capture` with the two environment variable
//! names, the file and the log prefix lifted into [`CAPTURE`]. The policy is
//! unchanged — one final frame once the armed self-tests finish, or every
//! frame under `ISOMETRY_CAPTURE_EVERY_FRAME=1`, written to a `.tmp` and
//! renamed — so `ISOMETRY_CAPTURE_DIR` still means exactly what it meant, and
//! `png` left this crate's manifest because `mesquite::write_png` does the
//! encoding now.
//!
//! **The scene board is the one producer.** B2 registered it inside
//! `SceneBoard::sync`; the assembly does it instead, from
//! [`Product::viewport_keys`] and [`Product::producer`]. With
//! `ISOMETRY_SCENE_BOARD` unset there is no scene board, so the key is never
//! named and nothing is registered — which is what [`NoProducer`] would have
//! been for had B2 not landed first. The snapshot push and the pixel grid stay
//! in `SceneBoard`, because they are about the board and not about hosting it.
//!
//! **No error line.** Isometry's DOM has nowhere to put one: the scene board
//! reports a producer refusal as a stderr line and the side panel has no error
//! element. So [`Product::PUBLISHES_ERROR`] is `false` and the assembly skips
//! the comparison entirely, rather than calling `runner.update` with nothing to
//! write and rebuilding the retained tree on every frame an error stood.
//! Giving Isometry the shared error line is a view change and belongs to a
//! later milestone.
//!
//! **No scenario lane.** Isometry drives itself through the `ISOMETRY_*_SELFTEST`
//! drivers rather than a `mesquite::Scenario`, so it hands the assembly no
//! lane; `after_frame` is the drivers, exactly as before.

use std::cell::RefCell;
use std::rc::Rc;

use isomere::host::{
    CaptureNames, CloseDisposition, CloseRequest, FocusedTextSlot, KeyPress, Product,
};
use isometry_views::{BOARD_SCENE_LEAF_KEY, BoardProducer, UiChild, UiState};

use crate::scene_board::SceneBoard;
use crate::{App, Ctx, Logic, Runner};

/// What this host calls its capture policy. The two variables and the file name
/// are the ones every receipt under `testing/isometry/images` was taken with.
pub(crate) const CAPTURE: CaptureNames = CaptureNames {
    directory_var: "ISOMETRY_CAPTURE_DIR",
    every_frame_var: "ISOMETRY_CAPTURE_EVERY_FRAME",
    file: "isometry_capture.png",
    log_prefix: "isometry",
};

/// Isometry, as the shared assembly sees it: one handle on the application
/// state the hooks used to capture.
pub(crate) struct Isometry(pub(crate) Rc<RefCell<App>>);

impl Product for Isometry {
    type State = UiState;
    type Logic = Logic;
    type View = UiChild;
    type Producer = Rc<RefCell<BoardProducer>>;

    const LOG_PREFIX: &'static str = "isometry";
    /// See the module header: this DOM has no error element to publish into.
    const PUBLISHES_ERROR: bool = false;

    /// B2's scene leaf, and only while the flag built one.
    fn viewport_keys(&self, _ctx: &Ctx<'_>, keys: &mut Vec<u64>) {
        if self.0.borrow().scene_board.is_some() {
            keys.push(BOARD_SCENE_LEAF_KEY);
        }
    }

    fn producer(&self, _ctx: &Ctx<'_>, _key: u64) -> Option<Self::Producer> {
        self.0
            .borrow()
            .scene_board
            .as_ref()
            .map(SceneBoard::producer)
    }

    fn frame(&mut self, ctx: &mut Ctx<'_>) -> bool {
        self.0.borrow_mut().frame_tick(ctx)
    }

    /// A still board parks on `Wait`, so an armed self-test would never reach
    /// its own deadline if the capture landed first. Unchanged policy.
    fn capture_ready(&self, _ctx: &Ctx<'_>) -> bool {
        !self.0.borrow().selftests_pending()
    }

    fn after_dispatch(&mut self, ctx: &mut Ctx<'_>) {
        self.0.borrow_mut().after_dispatch(ctx);
    }

    fn after_frame(&mut self, ctx: &mut Ctx<'_>) {
        self.0.borrow_mut().drive_selftests(ctx);
    }

    /// The session actor woke us: drain what it sent, on the UI thread, in one
    /// turn. The pumps that used to ride a 10Hz idle tick ride this instead,
    /// because a wake is exactly the moment they have work.
    fn after_wake(&mut self, ctx: &mut Ctx<'_>) {
        let mut app = self.0.borrow_mut();
        if app.net.is_none() {
            return;
        }
        app.pump_net(ctx);
        app.pump_sheets(ctx);
        app.pump_generators(ctx);
        app.pump_storylets(ctx);
        app.refresh_source_history(ctx);
    }

    /// Nothing here outlives the window: the campaign checkpoint is written on
    /// an explicit save, and the session actor dies with the process.
    fn close_request(&mut self, _ctx: &mut Ctx<'_>, _request: CloseRequest) -> CloseDisposition {
        CloseDisposition::Exit
    }

    fn focused_text(runner: &Runner) -> Option<FocusedTextSlot<UiState>> {
        crate::hooks::focused_text(runner)
    }

    fn key(&mut self, runner: &mut Runner, press: &KeyPress) -> bool {
        crate::hooks::key_intercept(runner, press)
    }

    fn profiling(&self) -> bool {
        self.0.borrow().profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The policy itself is tested in `isomere`, which now owns it. What is
    /// still this crate's to guarantee is the *contract*: every headed receipt
    /// ever taken names these two variables and reads that file, and a rename
    /// during the promotion would have broken them all silently. These two are
    /// what `capture.rs`'s own tests were replaced by.
    #[test]
    fn the_capture_variables_are_the_ones_every_receipt_was_taken_with() {
        assert_eq!(CAPTURE.directory_var, "ISOMETRY_CAPTURE_DIR");
        assert_eq!(CAPTURE.every_frame_var, "ISOMETRY_CAPTURE_EVERY_FRAME");
    }

    #[test]
    fn the_capture_file_and_prefix_are_unchanged() {
        assert_eq!(CAPTURE.file, "isometry_capture.png");
        assert_eq!(CAPTURE.log_prefix, "isometry");
        assert_eq!(Isometry::LOG_PREFIX, CAPTURE.log_prefix);
    }
}
