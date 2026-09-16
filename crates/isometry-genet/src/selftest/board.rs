//! The board's own gestures, self-driven: `ISOMETRY_SELECT_SELFTEST`.
//!
//! B3's headed receipt. Every other self-test drives a panel control or a text
//! lane; this one drives the *board*, which before B3 had nothing to drive on
//! the scene arm at all — the pane was one leaf and a press selected nothing.
//!
//! It presses where the scene shows **raised** ground, because that is the
//! case the two paths disagree on: the DOM board's flat-ground inverse names
//! the tile behind the one you clicked, and the pick names the one you can see.
//! The panel's `selected:` line then carries the answer and its height into the
//! capture, and the same two tiles are printed to stderr so the log says which
//! is which. A right press on a token follows, so the frame also shows a body
//! pick reaching the context menu.
//!
//! With the scene flag off it drives the same two gestures through the DOM
//! board, which is the positive control: the capture proves the driver works,
//! not that the flag does.

use super::*;

impl App {
    /// `ISOMETRY_SELECT_SELFTEST=1`: select a tile and open a token menu
    /// through the board's own pointer path, then leave both standing.
    pub(crate) fn maybe_select_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.select_selftest || self.select_fired {
            return;
        }
        // Later than the other lanes: the pick reads the frame the producer
        // last drew, so the scene has to have drawn one.
        if !self
            .started
            .is_some_and(|started| started.elapsed() > Duration::from_secs(3))
        {
            return;
        }
        self.select_fired = true;

        let ui = ctx.runner.state();
        let Some((local, at, elevation)) = raised_probe(ui) else {
            eprintln!("[isometry] select selftest: no raised ground is on screen");
            return;
        };
        let flat = ui
            .geo
            .screen_to_tile((local.0 - ui.camera.0, local.1 - ui.camera.1));
        eprintln!(
            "[isometry] select selftest: pressing pane {local:?}; the board shows {at:?} at \
             height {elevation}, the flat-ground inverse would say {flat:?}"
        );
        let (x, y) = (PANEL_W + local.0, local.1);
        ctx.pointer.push(HostPointer::Moved(x, y));
        ctx.pointer.push(HostPointer::Press(x, y));
        ctx.pointer.push(HostPointer::Release(x, y));

        // And one body pick: the token menu, opened by a right press on a
        // pixel the board says a token is drawn at.
        //
        // Found through the pick rather than projected from the token's tile,
        // and that is not fussiness: under a fractional zoom the scene draws
        // about nine per cent larger than the DOM board's own projection (the
        // camera's half height comes off the texture where the pan comes off
        // the pane), so a point computed the DOM's way lands half a tile from
        // the body. The pick is self-consistent with what was drawn, which is
        // the whole point of B3; the scale gap is the drawing lane's and is
        // recorded for B5.
        let Some((token_local, id)) = token_probe(ui) else {
            eprintln!("[isometry] select selftest: no token is on screen");
            return;
        };
        eprintln!(
            "[isometry] select selftest: right press on token {} at pane {token_local:?}",
            id.0
        );
        let (tx, ty) = (PANEL_W + token_local.0, token_local.1);
        ctx.pointer.push(HostPointer::Moved(tx, ty));
        ctx.pointer.push(HostPointer::SecondaryPress(tx, ty));
    }
}

/// A pane pixel the board draws a token's body at.
fn token_probe(ui: &UiState) -> Option<((f32, f32), TokenId)> {
    probe(ui, |pick| match pick {
        isometry_views::BoardPick::Token(id) => Some(id),
        _ => None,
    })
}

/// A pane pixel the board shows raised ground at, with the tile and height it
/// names. Coarse on purpose: the point only has to be somewhere on a hill.
fn raised_probe(ui: &UiState) -> Option<((f32, f32), TileCoord, i32)> {
    probe(ui, |pick| match pick {
        isometry_views::BoardPick::Tile {
            at,
            elevation,
            top: true,
        } if elevation > 0 && open_ground(ui, at) => Some((at, elevation)),
        _ => None,
    })
    .map(|(local, (at, elevation))| (local, at, elevation))
}

/// Whether `at` is open ground: no token, and no prop anywhere near it.
///
/// The prop radius is for the DOM arm, which is the control this capture is
/// taken against. A prop element is `standing_on` but not `clickable`, and its
/// box stands a whole sprite tall above the diamond it belongs to, so a tree
/// two tiles further into the screen still covers the pixel a press would use
/// and swallows the click. The scene board draws no props at all yet (B2
/// counts them; B4 owns them), so the restriction costs it nothing.
fn open_ground(ui: &UiState, at: TileCoord) -> bool {
    if ui.map.tokens.iter().any(|token| token.at == at) {
        return false;
    }
    let propped = |(col, row): TileCoord| {
        ui.map
            .props
            .get(col.max(0) as u32, row.max(0) as u32)
            .is_some_and(|kind| kind.0 != 0)
    };
    !(-2..=2).any(|dc| (-2..=2).any(|dr| propped((at.0 + dc, at.1 + dr))))
}

/// The first pane pixel, on a coarse grid, whose pick `want` accepts.
fn probe<T>(
    ui: &UiState,
    want: impl Fn(isometry_views::BoardPick) -> Option<T>,
) -> Option<((f32, f32), T)> {
    let (w, h) = ui.viewport;
    if !(w > 0.0 && h > 0.0) {
        return None;
    }
    const STEPS: u32 = 96;
    for iy in 0..STEPS {
        for ix in 0..STEPS {
            let local = (
                (ix as f32 + 0.5) * w / STEPS as f32,
                (iy as f32 + 0.5) * h / STEPS as f32,
            );
            if let Some(found) = ui.board_at(local).and_then(&want) {
                return Some((local, found));
            }
        }
    }
    None
}
