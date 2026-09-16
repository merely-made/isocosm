//! B5's cost model, measured: what the scene board costs when nothing moves,
//! when the camera moves, when a piece moves, and on the biggest board this
//! repository can produce.
//!
//! B4 measured the two *edits* — an elevation change at a handful of slots, a
//! focus change at one filtered rebuild — and [`super::edit_tests`] still owns
//! those. What was missing is everything that is **not** an edit, which is most
//! of what a session does: a still board, a pan, a token step. All three go
//! through the same seam, so all three are asked the same two questions here —
//! what the ground cost this crate, and what the tracer uploaded — rather than
//! being argued from the code.
//!
//! The ceiling is asked in the same place, because it is the one number the
//! plan's open decision turns on: `modulus::MAX_BRICKS` is 2,047, and a board
//! past it has no scene arm at all.

use std::time::Instant;

use super::ground::BoardGround;
use super::harness::board_or_skip;
use super::overlay::Overlays;
use super::terrain::MapTerrain;
use crate::demo::{demo_map, synth_map};
use crate::state::UiState;

/// A pan is the camera and nothing else: no regrow, no diff, no upload. It is
/// the cheapest thing the board does that still redraws the whole scene.
#[test]
fn a_pan_costs_the_ground_nothing() {
    let mut board = board_or_skip!("the pan receipt");
    board.draw();
    let grown = board.cost();

    board.ui.camera = (board.ui.camera.0 - 64.0, board.ui.camera.1 - 24.0);
    let started = Instant::now();
    board.draw();
    let elapsed = started.elapsed();
    let panned = board.terrain();
    eprintln!(
        "[isometry-b5] pan: {}, tracer uploaded {} bytes, frame {:.2} ms",
        board.cost().line(),
        panned.brick_upload_bytes,
        elapsed.as_secs_f32() * 1e3,
    );
    assert!(
        panned.map_revision_unchanged,
        "a pan does not move the board's revision"
    );
    assert_eq!(panned.brick_upload_bytes, 0, "and uploads no terrain");
    assert_eq!(
        board.cost().bricks,
        grown.bricks,
        "the same ground, seen from somewhere else"
    );
    assert_eq!(board.cost().slots, None);
}

/// A token step moves a body's pose. The terrain is untouched, so the step
/// costs the ground nothing either — the price is one re-traced frame.
#[test]
fn a_token_move_costs_the_ground_nothing() {
    let mut board = board_or_skip!("the token-move receipt");
    board.draw();
    let before = board.cost();

    let token = board.ui.map.tokens[0].id;
    let at = board.ui.map.token(token).expect("a token").at;
    let step = (at.0 + 1, at.1);
    board
        .ui
        .map
        .tokens
        .iter_mut()
        .find(|t| t.id == token)
        .expect("a token")
        .at = step;
    let started = Instant::now();
    board.draw();
    let elapsed = started.elapsed();
    let moved = board.terrain();
    eprintln!(
        "[isometry-b5] token {} {at:?} -> {step:?}: {}, tracer uploaded {} bytes, frame {:.2} ms",
        token.0,
        board.cost().line(),
        moved.brick_upload_bytes,
        elapsed.as_secs_f32() * 1e3,
    );
    assert!(
        moved.map_revision_unchanged,
        "a token is a body, not terrain"
    );
    assert_eq!(moved.brick_upload_bytes, 0);
    assert_eq!(board.cost().bricks, before.bricks);
}

/// The first grow is the board's one unavoidable large cost, and it is paid
/// again on every terrain or overlay change because `Ground` has no material
/// write (§6). Measured here rather than asserted, with the bare terrain
/// sampler beside the overlay one: the board never runs without overlays, but
/// the pair says how much of the price is the tint lookup per voxel and how
/// much is raising 33,000 cubes.
#[test]
fn what_the_ground_costs_to_raise() {
    for (name, map) in [
        ("demo 24x24", demo_map()),
        ("synth 30x30", synth_map(30, 30)),
    ] {
        let ui = UiState::new(map);
        let overlays = Overlays::of(&ui);
        let started = Instant::now();
        let bare = MapTerrain::new(&ui.map).grow();
        let plain = started.elapsed();
        let started = Instant::now();
        let ground = BoardGround::new(&ui.map, &overlays, 1).expect("the ground grows");
        let whole = started.elapsed();
        eprintln!(
            "[isometry-b5] {name}: no overlays grow {:.2} ms over {} bricks; \
             with overlays {} (grow + rebuild {:.2} ms)",
            plain.as_secs_f32() * 1e3,
            bare.brick_count(),
            ground.cost().line(),
            whole.as_secs_f32() * 1e3,
        );
        assert_eq!(
            bare.brick_count(),
            ground.cost().bricks,
            "a tint changes a voxel's material, never whether it is there"
        );
    }
}

/// What one brick costs to upload, from the demo board's own receipt:
/// `an_elevation_edit_reaches_the_next_frame_as_slots` reads 264,192 bytes for
/// 260 bricks off the tracer. Used to price a board this crate can grow but
/// never draws a frame of here.
const BRICK_BYTES: usize = 264_192 / 260;

/// The ceiling, found rather than quoted: the largest square board whose brick
/// map builds, and the first that does not.
///
/// §6 records the cap as "roughly 70 tiles square". This walks up to it, so the
/// number in the plan is a measurement and moves if the family's cap does.
/// `MAX_GENERATED_MAP_EDGE` is 256, so the substrate's own map generator will
/// happily author a board several times past whatever this finds.
#[test]
fn the_brick_cap_is_where_the_scene_board_stops() {
    let mut last_ok: Option<(u32, usize)> = None;
    let mut first_fail: Option<(u32, String)> = None;
    for edge in [24, 30, 48, 64, 68, 70, 72, 80, 96] {
        let ui = UiState::new(synth_map(edge, edge));
        let overlays = Overlays::of(&ui);
        match BoardGround::new(&ui.map, &overlays, 1) {
            Ok(ground) => {
                let bricks = ground.cost().bricks;
                eprintln!(
                    "[isometry-b5] {edge}x{edge}: {} ({} KiB of brick to upload)",
                    ground.cost().line(),
                    bricks * BRICK_BYTES / 1024,
                );
                last_ok = Some((edge, bricks));
            },
            Err(why) => {
                eprintln!("[isometry-b5] {edge}x{edge}: refused — {why}");
                first_fail = Some((edge, why));
                break;
            },
        }
    }
    let (edge, bricks) = last_ok.expect("the demo-sized board fits");
    let (failed, why) = first_fail.expect("some square board is past the cap");
    eprintln!(
        "[isometry-b5] brick cap: {edge}x{edge} fits at {bricks} bricks, {failed}x{failed} \
         refuses ({why}); the map generator's own edge cap is {}",
        isometry_campaign::MAX_GENERATED_MAP_EDGE
    );
    assert!(
        edge >= 30,
        "the repo's own 30x30 stress board must still have a scene arm"
    );
    assert!(
        failed < isometry_campaign::MAX_GENERATED_MAP_EDGE,
        "a board the substrate will generate is past the scene board's cap, which is \
         the fact §5 decision 4 is held open on"
    );
}
