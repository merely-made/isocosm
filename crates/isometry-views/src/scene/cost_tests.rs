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
//! The ceiling is asked in the same place. B5 walked it as a wall: 70 by 70 fit
//! `modulus::MAX_BRICKS` of 2,047 and 72 by 72 refused its map. Paging turned
//! it from the board's limit into the atlas's: the walk now shows every board,
//! the generator's 256 included, holding only what its frame shows.

use std::time::{Duration, Instant};

use isometer::{BrickSource, TerrainSource};
use isometry_core::MapDocument;

use super::columns::{BoardBricks, TileColumns};
use super::ground::BoardGround;
use super::harness::{CARD, PANE, board_or_skip, relief_map};
use super::overlay::Overlays;
use super::terrain::MapTerrain;
use super::world::BoardWorld;
use crate::demo::{demo_map, synth_map};
use crate::state::UiState;

#[test]
fn current_omissions_reach_the_host_without_a_residency_change() {
    let ui = UiState::new(demo_map());
    let mut ground = BoardGround::new(&ui.map, &Overlays::of(&ui), 1, Default::default(), CARD);
    let camera = BoardWorld::new(&ui.map)
        .camera(&ui.geo, ui.camera, PANE, None)
        .expect("a camera");
    let mut map = {
        let bricks = ground.bricks();
        let mut framed = ground.framed(camera, &bricks);
        assert!(!framed.keys.is_empty());
        framed.overflow = 7;
        ground.terrain(&bricks, &framed).brick_map().unwrap()
    };
    ground.uploaded(Duration::ZERO);
    let original = ground.cost();
    for omitted in [12, 3, 0] {
        {
            let bricks = ground.bricks();
            let mut framed = ground.framed(camera, &bricks);
            framed.overflow = omitted;
            assert!(matches!(
                ground
                    .terrain(&bricks, &framed)
                    .refresh(&mut map, &[])
                    .unwrap(),
                isometer::TerrainRefresh::Current
            ));
        }
        ground.uploaded(Duration::ZERO);
        let current = ground.cost();
        assert_eq!(current.residency.overflow, omitted);
        assert!(
            current
                .line()
                .contains(&format!("{omitted} past the atlas"))
        );
        let mut expected = original;
        expected.residency.overflow = omitted;
        assert_eq!(current, expected, "only the current count moved");
    }
}

/// A pan moves the camera and nothing else: no column is read and no brick is
/// remade except the ones it brings into view. What it uploads is the pointer
/// volume plus those bricks, to the byte.
#[test]
fn a_pan_reads_nothing_and_uploads_only_what_it_frames() {
    let mut board = board_or_skip!("the pan receipt");
    board.draw();

    board.ui.camera = (board.ui.camera.0 - 64.0, board.ui.camera.1 - 24.0);
    let started = Instant::now();
    board.draw();
    let elapsed = started.elapsed();
    let panned = board.terrain();
    let cost = board.cost();
    eprintln!(
        "[isometry-b5] pan: {}, tracer uploaded {} bytes, frame {:.2} ms",
        cost.line(),
        panned.brick_upload_bytes,
        elapsed.as_secs_f32() * 1e3,
    );
    assert!(
        !panned.map_revision_unchanged,
        "the demo board is wider than the pane, so this pan frames new bricks"
    );
    assert_eq!(
        (cost.columns, cost.diff, cost.tiles),
        (Duration::ZERO, Duration::ZERO, Some(0)),
        "a pan reads no map"
    );
    let held = cost.residency;
    assert!(held.resident <= held.capacity);
    assert_eq!(held.rebuilt, None, "a pan retargets");
    let pointers: u64 = held.extent.iter().map(|axis| u64::from(*axis)).product();
    assert!(panned.projection_replaced && !panned.full_map_upload);
    assert_eq!(
        panned.brick_upload_bytes,
        pointers * 4 + held.loaded as u64 * 512,
        "the pointer volume and the bricks brought into view"
    );
}

/// A token step moves a body's pose. The terrain is untouched, so the step
/// costs the ground nothing — the price is one re-traced frame.
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
    assert_eq!(board.cost(), before, "and nothing about the ground moved");
}

/// Every brick the columns say a board holds, counted.
fn board_bricks(bricks: &BoardBricks<'_>) -> usize {
    let Some([low, high]) = bricks.bounds() else {
        return 0;
    };
    (low[2]..=high[2])
        .flat_map(|z| (low[0]..=high[0]).map(move |x| [x, z]))
        .map(|column| bricks.layers(column).len())
        .sum()
}

/// Reading a board is the one cost every change pays before its frame. It
/// replaced raising the whole ground, which every change paid before paging;
/// the two are measured side by side, and they agree on the bricks.
#[test]
fn what_reading_the_ground_costs_beside_raising_it() {
    for (name, map) in [
        ("demo 24x24", demo_map()),
        ("synth 30x30", synth_map(30, 30)),
    ] {
        let ui = UiState::new(map);
        let overlays = Overlays::of(&ui);
        let started = Instant::now();
        let grown = MapTerrain::with_overlays(&ui.map, Some(&overlays)).grow();
        let raise = started.elapsed();
        let started = Instant::now();
        let columns = TileColumns::of(&ui.map, &overlays);
        let read = started.elapsed();
        let bricks = BoardBricks::new(&columns, None);
        eprintln!(
            "[isometry-b5] {name}: raising the ground {:.2} ms over {} bricks; reading its \
             columns {:.2} ms",
            raise.as_secs_f32() * 1e3,
            grown.brick_count(),
            read.as_secs_f32() * 1e3,
        );
        assert_eq!(grown.brick_count(), board_bricks(&bricks));
    }
}

/// The board's first map, built the way the scene builds it: framed from the
/// harness's pane on the map's centre tile.
fn first_map(map: &MapDocument) -> (BoardGround, usize, usize) {
    let ui = UiState::new(map.clone());
    let ground = BoardGround::new(&ui.map, &Overlays::of(&ui), 1, Default::default(), CARD);
    let centre = (
        map.ground.width() as i32 / 2,
        map.ground.height() as i32 / 2,
    );
    let (x, y) = ui.geo.tile_to_screen(centre, 0);
    let camera = BoardWorld::new(map)
        .camera(&ui.geo, (PANE.0 / 2.0 - x, PANE.1 / 2.0 - y), PANE, None)
        .expect("the board frames the pane");
    let bricks = ground.bricks();
    let framed = ground.framed(camera, &bricks);
    let built = ground
        .terrain(&bricks, &framed)
        .brick_map()
        .expect("a paged map is never refused");
    let held = ground.resident().len();
    assert_eq!(held, framed.keys.len());
    let (total, capacity) = (board_bricks(&bricks), ground.capacity());
    assert!(built.capacity() == capacity && held <= capacity);
    (ground, held, total)
}

/// The cap, walked again: B5 found 70 by 70 fit at 1,936 bricks and 72 by 72
/// refused at 2,116. Paged, the atlas bounds what one frame holds rather
/// than what a board may be, so every board builds its map, the generator's
/// own 256 by 256 included, holding the same frame's worth of bricks however
/// wide it grows.
#[test]
fn the_brick_cap_bounds_residency_not_the_board() {
    let mut held_at = Vec::new();
    for edge in [24, 30, 48, 64, 68, 70, 72, 80, 96, 256] {
        let (ground, held, total) = first_map(&synth_map(edge, edge));
        eprintln!(
            "[isometry-b5] {edge}x{edge}: {total} bricks on the board, {held} of {} held for the \
             frame",
            ground.capacity(),
        );
        held_at.push((edge, held, total));
    }
    let edge = isocosm::legacy::campaign::MAX_GENERATED_MAP_EDGE;
    let (_, held, total) = first_map(&relief_map(edge));
    eprintln!("[isometry-b5] {edge}x{edge} with relief: {total} bricks on the board, {held} held");
    let past_the_old_cap: Vec<_> = held_at
        .iter()
        .filter(|(_, _, total)| *total > 2_047)
        .collect();
    assert!(
        past_the_old_cap.len() >= 4,
        "72, 80, 96 and 256 are past what one map could hold"
    );
    let widest = held_at.last().expect("the walk ran");
    assert_eq!(widest.0, edge, "the walk reaches the generator's edge");
    assert!(
        held_at
            .iter()
            .filter(|(e, ..)| *e >= 64)
            .all(|(_, h, _)| *h == widest.1),
        "a board wider than the frame holds the frame's bricks, not its own"
    );
}
