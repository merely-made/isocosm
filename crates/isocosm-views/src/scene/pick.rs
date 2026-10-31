//! The pointer's seam into the drawn frame (B3).
//!
//! B2 left [`BoardSource::pick`](super::BoardSource::pick) answering what a
//! pixel of the last drawn frame shows, and nothing routing a pointer through
//! it. This is the handle that closes that gap: the host holds the producer,
//! the view holds a [`ScenePick`], and a gesture asks it what is under the
//! pointer instead of inverting a projection.
//!
//! **Why a handle rather than a field of state.** The frame the pick reads is
//! the one the producer just drew, and the producer lives in the host's own
//! registry. A snapshot pushed into [`UiState`](crate::state::UiState) would
//! be a frame stale by construction — the pointer arrives between frames — so
//! what the view carries is a borrow of the live producer, asked at the moment
//! the gesture runs.
//!
//! **Coordinates.** A gesture is handed `local`, the pointer measured against
//! the board pane's own painted box, and the scene leaf is that same box: one
//! `custom_leaf` sized to `UiState::viewport` at the pane's origin. So the
//! pane's logical pixels map straight onto the presented image, whatever
//! render scale the producer drew it at — the scale changes how many texels
//! the image has, never how much of the pane it covers.

use std::cell::RefCell;
use std::rc::Rc;

use super::board::{BoardPick, BoardProducer};

/// The board's drawn frame, as a gesture sees it.
///
/// Cheap to clone; every clone reads the one producer the host registered.
#[derive(Clone)]
pub struct ScenePick(Rc<RefCell<BoardProducer>>);

impl ScenePick {
    pub fn new(producer: Rc<RefCell<BoardProducer>>) -> Self {
        Self(producer)
    }

    /// What the last drawn frame shows at a pane-local point.
    ///
    /// `None` off the pane, before the first frame, or where the ray leaves
    /// the map — the same three answers the DOM board's inverse gives for a
    /// point that names no tile.
    ///
    /// The borrow is fallible on purpose: the producer is held mutably while
    /// it renders, and a gesture that arrived mid-render should report nothing
    /// rather than panic the host.
    pub fn at(&self, pane: (f32, f32), viewport: (f32, f32)) -> Option<BoardPick> {
        let ndc = pane_ndc(pane, viewport)?;
        self.0.try_borrow().ok()?.source().pick(ndc)
    }
}

impl std::fmt::Debug for ScenePick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScenePick")
    }
}

/// A pane-local point as normalized clip coordinates over a `viewport`-sized
/// leaf, or `None` when the point is outside it.
///
/// Outside rather than clamped: a drag captures the pointer, so a gesture that
/// wanders onto the side panel keeps arriving here, and clamping would report
/// the pane's edge tile as if the pointer were on it.
pub fn pane_ndc(pane: (f32, f32), viewport: (f32, f32)) -> Option<[f32; 2]> {
    if !(viewport.0 > 0.0 && viewport.1 > 0.0) {
        return None;
    }
    if pane.0 < 0.0 || pane.1 < 0.0 || pane.0 > viewport.0 || pane.1 > viewport.1 {
        return None;
    }
    Some([
        2.0 * pane.0 / viewport.0 - 1.0,
        1.0 - 2.0 * pane.1 / viewport.1,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panes_corners_are_the_clip_cubes_corners() {
        let pane = (640.0, 480.0);
        assert_eq!(pane_ndc((0.0, 0.0), pane), Some([-1.0, 1.0]));
        assert_eq!(pane_ndc((640.0, 480.0), pane), Some([1.0, -1.0]));
        let [x, y] = pane_ndc((320.0, 240.0), pane).expect("the centre is on the pane");
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6, "the centre is the origin");
    }

    #[test]
    fn a_point_off_the_pane_names_no_pixel() {
        let pane = (640.0, 480.0);
        // A drag that wandered onto the side panel, and one past the bottom.
        assert_eq!(pane_ndc((-12.0, 100.0), pane), None);
        assert_eq!(pane_ndc((100.0, 481.0), pane), None);
        // And before the host has reported a pane at all.
        assert_eq!(pane_ndc((10.0, 10.0), (0.0, 0.0)), None);
    }
}
