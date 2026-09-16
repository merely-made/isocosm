//! B3's receipts: the board's gestures on the **scene** board, through the
//! real host and a real drawn frame.
//!
//! [`super::host_routing`] proves the routing on the DOM board, where identity
//! is closure-captured on one element per tile. Under `ISOMETRY_SCENE_BOARD`
//! there are no such elements: the pane carries one `custom_leaf` and nothing
//! else, so a press that selects a tile can only have got there through
//! `isometer`'s own pick. That is what these assert, and the tile-element count
//! is asserted beside every one of them so the claim cannot quietly become
//! "the DOM did it after all".
//!
//! **The instrument.** [`Harness`] is the shipping host with `window: None`,
//! and it has no swapchain, so it never drives a producer. The frame is
//! therefore drawn here, headlessly, at the pane's own size and render scale 1
//! — the same thing `scene::parity_tests` does in the view crate — and the
//! [`ScenePick`] over that producer is installed on the state exactly as
//! `hooks::init` installs it headed. Everything downstream of `event.local` is
//! the shipping path.
//!
//! Without a GPU adapter every receipt here skips **loudly** and asserts
//! nothing, rather than passing on an absence.

use std::cell::RefCell;
use std::rc::Rc;

use cambium_genet_winit_host::{Harness, Init, inert_hooks};
use isometer::{FrameRequest, SceneSource};
use isometry_core::{TileCoord, TokenId};
use isometry_views::{BoardPick, BoardProducer, BoardSource, BoardView, ScenePick};
use taproot::Selector;

use super::*;

type BoardHarness = Harness<UiState, Logic, UiChild>;

/// The window the receipts lay out in, matching `host_routing`'s.
const WINDOW: (f32, f32) = (1_100.0, 820.0);

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

/// A laid-out scene board with one real frame behind it, or `None` where no
/// adapter exists.
fn scene_board() -> Option<BoardHarness> {
    let (device, queue) = device()?;
    let mut ui = UiState::new(demo_map());
    // The same camera and pane `host_routing` uses, so a point that names a
    // tile there names one here.
    ui.camera = (420.0, 140.0);
    ui.viewport = (WINDOW.0 - PANEL_W, WINDOW.1);
    ui.scene_board = true;

    let mut board = BoardView::new(ui.map.clone());
    board.sync(&ui);
    let producer = Rc::new(RefCell::new(BoardProducer::new(BoardSource::new(
        board.into_handle(),
    ))));
    let size = [ui.viewport.0 as u32, ui.viewport.1 as u32];
    let request = FrameRequest {
        device: &device,
        queue: &queue,
        size,
        aspect: ui.viewport.0 / ui.viewport.1,
        color: None,
        needs_frame: true,
        render_scale: 1,
    };
    <BoardSource as SceneSource>::frame(producer.borrow_mut().source_mut(), &request)
        .expect("the scene board draws one frame");
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the frame completes");
    ui.board_pick = Some(ScenePick::new(producer.clone()));

    let mut hooks = inert_hooks();
    hooks.key_intercept = Box::new(hooks::key_intercept);
    hooks.focused_text = Box::new(hooks::focused_text);
    let mut harness = Harness::with_hooks(
        Init {
            state: ui,
            logic: board_root as Logic,
            sheet: board_css(),
            fonts: Vec::new(),
            images: Vec::new(),
        },
        hooks,
    );
    harness.layout_at(WINDOW.0, WINDOW.1);
    Some(harness)
}

macro_rules! adapter_or_skip {
    ($what:literal) => {
        match scene_board() {
            Some(harness) => harness,
            None => {
                eprintln!(
                    "SKIPPED: no wgpu adapter on this machine, so no scene frame exists and \
                     {} asserts nothing.",
                    $what
                );
                return;
            },
        }
    };
}

/// How many `.tile` elements the board is drawing. Zero is the whole premise:
/// under the flag nothing in the tree names a tile.
fn tile_elements(harness: &BoardHarness) -> usize {
    harness.with_dom(|dom| taproot::matching(dom, &Selector::class("tile")).len())
}

/// Pane-local coordinates of a window point. The `.board` container sits at
/// the pane's origin under the flag (the scene carries the pan in its camera),
/// so this is the panel strip and nothing else.
fn pane_local(x: f32, y: f32) -> (f32, f32) {
    (x - PANEL_W, y)
}

/// Where a tile's diamond centre lands in window coordinates, through the DOM
/// board's own projection. The scene is aligned to it to within a fifth of a
/// pixel per elevation step (`scene::world`), which is why a point computed
/// this way is a fair probe of the pick.
fn tile_point(harness: &BoardHarness, at: TileCoord) -> (f32, f32) {
    let ui = harness.state();
    let elevation = *ui
        .map
        .elevation
        .get(at.0.max(0) as u32, at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let (sx, sy) = ui.geo.tile_to_screen(at, elevation);
    (PANEL_W + sx + ui.camera.0, sy + ui.camera.1)
}

/// A press selects the tile the pick names, with no tile element anywhere in
/// the tree for the click to have come from.
#[test]
fn a_press_selects_the_tile_the_pick_names() {
    let mut harness = adapter_or_skip!("the scene selection receipt");
    assert_eq!(
        tile_elements(&harness),
        0,
        "the scene board emits no tile elements, so only the pick can select"
    );
    let (x, y) = tile_point(&harness, (12, 12));
    let named = harness.state().board_at(pane_local(x, y));
    let Some(BoardPick::Tile { at, .. }) = named else {
        panic!("the demo map's centre is ground under the pointer, got {named:?}");
    };
    assert!(harness.state().selected.is_none());

    harness.click_at(x, y);
    harness.relayout();

    assert_eq!(
        harness.state().selected,
        Some(at),
        "the press selected exactly the tile the pick named"
    );
    assert_eq!(tile_elements(&harness), 0, "and still no tile elements");
}

/// The improvement the scene brings, asserted: on raised ground the pick names
/// the tile you can *see*, where the DOM board's flat-ground inverse names the
/// tile behind it.
///
/// The probe is found rather than hard-coded — a pane pixel where the scene
/// reports a raised top face and the retired inverse would have answered
/// something else — so this keeps meaning what it means if the demo map moves.
#[test]
fn on_raised_ground_the_press_names_the_tile_you_see() {
    let mut harness = adapter_or_skip!("the raised-ground receipt");
    // The DOM board's arm of the same resolver, over the same map and camera.
    let flat = {
        let mut ui = UiState::new(harness.state().map.clone());
        ui.camera = harness.state().camera;
        ui.viewport = harness.state().viewport;
        ui
    };
    let pane = harness.state().viewport;
    let mut probe = None;
    for iy in 0..48 {
        for ix in 0..48 {
            let local = (
                (ix as f32 + 0.5) * pane.0 / 48.0,
                (iy as f32 + 0.5) * pane.1 / 48.0,
            );
            let Some(BoardPick::Tile {
                at,
                elevation,
                top: true,
            }) = harness.state().board_at(local)
            else {
                continue;
            };
            if elevation > 0 && flat.tile_at(local) != Some(at) {
                probe = Some((local, at, flat.tile_at(local), elevation));
                break;
            }
        }
        if probe.is_some() {
            break;
        }
    }
    let Some((local, seen, behind, elevation)) = probe else {
        panic!("the demo map stands a hill, so some pixel must show a raised top face");
    };
    eprintln!(
        "raised probe at {local:?}: the scene shows {seen:?} at height {elevation}, the \
         retired inverse would have said {behind:?}"
    );

    harness.click_at(PANEL_W + local.0, local.1);
    harness.relayout();
    assert_eq!(
        harness.state().selected,
        Some(seen),
        "the press selected the tile on screen, not the one behind it"
    );
    assert_ne!(
        harness.state().selected,
        behind,
        "and that really is the answer the flat-ground inverse could not give"
    );
}

/// A right press on a token opens its menu, through a body pick.
///
/// The menu itself is still DOM — B3 changed how a press finds its token, not
/// what the menu is — so the rows are asserted the way `host_routing` asserts
/// them, which also proves the overlay still stands over a leaf-only pane.
#[test]
fn a_right_press_on_a_token_opens_its_menu_through_the_pick() {
    let mut harness = adapter_or_skip!("the scene context-menu receipt");
    let knight = harness
        .state()
        .map
        .token(TokenId(1))
        .expect("the demo skirmish stands a knight")
        .at;
    let (x, y) = tile_point(&harness, knight);
    assert_eq!(
        harness.state().board_at(pane_local(x, y)),
        Some(BoardPick::Token(TokenId(1))),
        "the knight's own body is what the pixel shows"
    );

    harness.right_click_at(x, y);
    let (id, _) = harness
        .state()
        .context_menu
        .expect("a right press on a token opens its menu");
    assert_eq!(id, TokenId(1));
    assert_eq!(harness.state().selected_token, Some(TokenId(1)));
    assert!(harness.click_on(&Selector::class("command-item")));
    assert_eq!(harness.state().open_sheet, Some(TokenId(1)));
}

/// A paint drag applies the brush once per tile crossing, not once per pixel,
/// and the press's own application is not repeated by the first move.
///
/// Raise rather than a ground brush, because raising is *counted*: painting
/// grass over grass twice leaves no trace, while a second application of Raise
/// on one tile would show as a second step of elevation. So the receipt reads
/// the map, not a call count.
#[test]
fn a_drag_applies_once_per_tile_crossing() {
    let mut harness = adapter_or_skip!("the scene drag receipt");
    harness.update(|ui| ui.mode = isometry_views::EditMode::Raise);

    let (x, y) = tile_point(&harness, (12, 12));
    let first = harness
        .state()
        .tile_at(pane_local(x, y))
        .expect("the probe is on the map");
    let before = elevation_at(&harness, first);
    harness.press_at(x, y);
    assert_eq!(harness.state().drag_tile, Some(first));
    // Two moves inside the same tile: one tile, one application.
    harness.move_to(x + 1.0, y + 1.0);
    harness.move_to(x - 1.0, y + 1.0);
    assert_eq!(
        elevation_at(&harness, first),
        before + 1,
        "the press raised once and neither move inside the tile raised again"
    );

    // Onto the next tile down the row axis, then up.
    let (nx, ny) = tile_point(&harness, (12, 13));
    let second = harness
        .state()
        .tile_at(pane_local(nx, ny))
        .expect("the neighbour is on the map");
    let second_before = elevation_at(&harness, second);
    harness.move_to(nx, ny);
    harness.release_at(nx, ny);

    assert_ne!(first, second, "the two probes really are different tiles");
    assert_eq!(
        elevation_at(&harness, second),
        second_before + 1,
        "crossing into the neighbour raised it exactly once"
    );
    assert_eq!(
        elevation_at(&harness, first),
        before + 1,
        "and left the tile the press started on alone"
    );
    assert_eq!(harness.state().drag_tile, None, "the release ends the drag");
}

/// The height authored at a tile.
fn elevation_at(harness: &BoardHarness, at: TileCoord) -> u8 {
    *harness
        .state()
        .map
        .elevation
        .get(at.0 as u32, at.1 as u32)
        .unwrap_or(&0)
}

/// The scene arm's clip receipt: what is selectable is the visible diamond,
/// pixel by pixel.
///
/// `watchtower_tests::board_tile_clips_its_hit_area_to_the_visible_diamond`
/// states this for the DOM board, where a tile is a rectangle with a
/// `clip-path` and the corners of that rectangle must not select it. The scene
/// has no rectangle at all — the pick is a ray — so the same claim is made
/// directly: the diamond's centre names the tile, the four corners of the box
/// it would have occupied name something else, and a pixel the scene draws as
/// sky names nothing.
#[test]
fn the_scene_board_clips_its_hit_area_to_the_visible_diamond() {
    let harness = adapter_or_skip!("the scene clip receipt");
    let ui = harness.state();
    let at: TileCoord = (12, 12);
    let (cx, cy) = tile_point(&harness, at);
    let (local_x, local_y) = pane_local(cx, cy);
    assert!(
        matches!(ui.board_at((local_x, local_y)), Some(BoardPick::Tile { at: hit, .. }) if hit == at),
        "the diamond's own centre names it"
    );

    let (half_w, half_h) = (ui.geo.tile_w / 2.0, ui.geo.tile_h / 2.0);
    // The same four fractions the DOM receipt presses: 10% and 90% of the box,
    // which on a 2:1 diamond are all outside it.
    for (fx, fy) in [(-0.8, -0.8), (0.8, -0.8), (-0.8, 0.8), (0.8, 0.8)] {
        let corner = (local_x + fx * half_w, local_y + fy * half_h);
        let named = ui.board_at(corner);
        assert!(
            !matches!(named, Some(BoardPick::Tile { at: hit, .. }) if hit == at),
            "the invisible corner {corner:?} is not part of {at:?}, got {named:?}"
        );
    }

    // And a pixel above the board's own top corner is sky: nothing is there to
    // select, which is what the DOM board's empty pane says by having no
    // element under the pointer.
    let sky = (local_x, 1.0);
    assert_eq!(
        ui.board_at(sky),
        None,
        "a pixel the scene draws as sky names no tile"
    );
}
