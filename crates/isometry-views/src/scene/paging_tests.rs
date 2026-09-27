//! Paging's receipts over real frames (2026-09-26).
//!
//! The scene board holds only the bricks its frame shows. These draw boards
//! past the old cap through the shipping path and ask the frame what it holds:
//! that the generator's own 256 by 256 draws inside the atlas, that residency
//! follows the view, that one view holds the same bricks however it was
//! reached, and that an edit made while its bricks were away is on the board
//! when they come back.
//!
//! Headroom (2026-09-26, Mark: "Reserve headroom"): the pointer volume keeps
//! spare brick layers above the board's tallest tile, so raising a tile into
//! them retargets and only raising one past them rebuilds the map whole.
//!
//! `paging_receipts_at_256` is the cost receipt the lane's summary is built
//! from, and `paging_receipts_headroom` what each spare layer costs. Both are
//! ignored by default, because the first's baseline raises a 256 by 256 ground
//! the old way, which is a minute in the debug build; the script in
//! `testing/scene-board-paging/` runs them in both builds.

use std::collections::BTreeSet;
use std::time::Instant;

use isometer::core::ground::Ground;
use isometer::{Rebuild, ResidencySettings, TerrainSource};

use super::board::BoardPick;
use super::ground::BoardGround;
use super::harness::{Board, HEADED_PANE, PANE, relief_map};
use super::overlay::Overlays;
use super::terrain::MapTerrain;
use super::world::BoardWorld;
use crate::demo::{demo_map, synth_map};
use crate::state::UiState;

fn sized_or_skip(map: isometry_core::MapDocument, pane: (f32, f32), what: &str) -> Option<Board> {
    let board = Board::sized(map, pane);
    if board.is_none() {
        eprintln!("SKIPPED: no wgpu adapter on this machine, so {what} asserts nothing.");
    }
    board
}

/// A grid of pane pixels across the whole pane.
fn probes(pane: (f32, f32)) -> impl Iterator<Item = (f32, f32)> {
    (1..12).flat_map(move |row| {
        (1..12).map(move |col| (pane.0 * col as f32 / 12.0, pane.1 * row as f32 / 12.0))
    })
}

/// The scene holds exactly the bricks the board's rule frames for the camera
/// it drew, never more than the atlas, and the frame shows terrain.
fn assert_holds_its_frame(board: &Board, name: &str) -> BTreeSet<[i16; 3]> {
    let ground = board.ground();
    let bricks = ground.bricks();
    let framed = ground.framed(board.camera(), &bricks);
    let resident = ground.resident();
    assert_eq!(
        resident,
        framed.keys.iter().copied().collect(),
        "{name}: the scene holds what the frame shows"
    );
    assert!(
        resident.len() <= ground.capacity(),
        "{name}: inside the atlas"
    );
    let hits = probes(board.pane())
        .filter(|(x, y)| board.pick(*x, *y).is_some())
        .count();
    assert!(hits > 0, "{name}: the frame shows its ground");
    resident
}

#[test]
fn a_256_board_draws_holding_what_its_frame_shows() {
    for (name, map) in [
        ("256", synth_map(256, 256)),
        ("256 with relief", relief_map(256)),
    ] {
        let Some(mut board) = sized_or_skip(map, PANE, "the 256 receipt") else {
            return;
        };
        board.draw();
        let first = assert_holds_its_frame(&board, name);
        board.ui.camera.0 -= 20.0 * 32.0;
        board.draw();
        let moved = assert_holds_its_frame(&board, name);
        let kept = first.intersection(&moved).count();
        eprintln!(
            "{name}: {} bricks held, then {} after a twenty-tile pan with {kept} kept; {}",
            first.len(),
            moved.len(),
            board.cost().line()
        );
        assert_ne!(first, moved, "{name}: residency follows the view");
    }
}

/// One view, reached directly and by a walk through five others, holds the
/// same bricks and answers every probe the same. Each retarget in the walk is
/// checked to the byte against the tracer.
#[test]
fn one_view_holds_the_same_bricks_however_it_was_reached() {
    let map = relief_map(256);
    let what = "the determinism receipt";
    let (Some(mut direct), Some(mut walked)) = (
        sized_or_skip(map.clone(), PANE, what),
        sized_or_skip(map, PANE, what),
    ) else {
        return;
    };
    let home = direct.ui.camera;
    let there = (home.0 - 150.0, home.1 + 90.0);
    direct.ui.camera = there;
    direct.draw();
    walked.draw();
    let mut retargets = 0;
    for step in [
        (-37.0, 11.0),
        (-90.0, 60.0),
        (20.0, 40.0),
        (-180.0, 30.0),
        (-150.0, 90.0),
    ] {
        walked.ui.camera = (home.0 + step.0, home.1 + step.1);
        walked.draw();
        let held = walked.cost().residency;
        if held.rebuilt.is_none() && held.loaded + held.evicted > 0 {
            retargets += 1;
            let pointers: u64 = held.extent.iter().map(|axis| u64::from(*axis)).product();
            assert_eq!(
                walked.terrain().brick_upload_bytes,
                pointers * 4 + held.loaded as u64 * 512,
                "a retarget uploads the pointer volume and the loaded bricks"
            );
        }
    }
    assert!(retargets > 0, "the walk retargets in a real frame");
    assert_eq!(walked.ui.camera, there);
    assert_eq!(
        direct.ground().resident(),
        walked.ground().resident(),
        "the same view holds the same bricks"
    );
    for (x, y) in probes(PANE) {
        assert_eq!(direct.pick(x, y), walked.pick(x, y), "probe ({x}, {y})");
    }
}

/// An edit made while its tile's bricks are away touches nothing the scene
/// holds, and is on the board when the view comes back to it.
#[test]
fn an_edit_made_away_is_on_the_board_when_its_bricks_return() {
    let Some(mut board) = sized_or_skip(synth_map(256, 256), PANE, "the return receipt") else {
        return;
    };
    board.draw();
    let Some(BoardPick::Tile { at, elevation, .. }) = board.pick(PANE.0 / 2.0, PANE.1 / 2.0) else {
        panic!("the pane's centre shows a tile");
    };
    let home = board.ui.camera;
    board.ui.camera.0 -= 1600.0;
    board.draw();
    let world = BoardWorld::new(&board.ui.map);
    let footing = world.stand(at, elevation).map(|v| (v / 8.0).floor() as i16);
    assert!(
        !board.ground().resident().contains(&footing),
        "fifty tiles away, the tile's bricks are not held"
    );

    // A new height inside the ground's lowest brick layer: an edit that lifts
    // the board's tallest point into a new layer moves the pointer extent,
    // and the map is rebuilt whole for it, which is not what this asks.
    let raised = if elevation < 3 { 3 } else { 0 };
    board
        .ui
        .map
        .elevation
        .set(at.0 as u32, at.1 as u32, raised as u8);
    board.draw();
    let away = board.cost();
    eprintln!("an edit away: {}", away.line());
    assert_eq!((away.tiles, away.slots), (Some(1), Some(0)));
    assert_eq!(
        board.terrain().brick_upload_bytes,
        0,
        "nothing held was touched"
    );

    board.ui.camera = home;
    board.draw();
    let top = world.stand(at, raised);
    let pixel = board
        .camera()
        .pixel_of(top, [PANE.0 as u32, PANE.1 as u32])
        .expect("the raised tile's top is on screen");
    match board.pick(pixel[0] as f32 + 0.5, pixel[1] as f32 + 0.5) {
        Some(BoardPick::Tile {
            at: hit,
            elevation: height,
            top: true,
        }) => {
            assert_eq!(
                (hit, height),
                (at, raised),
                "the edit came back with its bricks"
            );
        },
        other => panic!("the raised tile's top picks {other:?}"),
    }
}

/// What paging costs at the generator's edge, per click, per raise and per
/// pan, beside the regrow it replaced; one JSON line per measurement.
#[test]
#[ignore = "the receipts script runs this in both builds"]
fn paging_receipts_at_256() {
    let build = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let line = |value: serde_json::Value| println!("[paging-receipt] {value}");
    for (board_name, map) in [("flat", synth_map(256, 256)), ("relief", relief_map(256))] {
        // The old path, once: raise the whole ground and compare it brick by
        // brick, which is what every click paid before paging.
        let ui = UiState::new(map.clone());
        let overlays = Overlays::of(&ui);
        let started = Instant::now();
        let grown = MapTerrain::with_overlays(&ui.map, Some(&overlays)).grow();
        let grow = started.elapsed();
        // B4's `compare`, as it ran: both key sets, then both brick copies per
        // key. Against a clone, so every brick is read and none differs.
        let held = grown.clone();
        let started = Instant::now();
        let before: BTreeSet<_> = held.keys().collect();
        let after: BTreeSet<_> = grown.keys().collect();
        let raw = |ground: &Ground, key| {
            ground
                .brick_materials(key)
                .map(|(brick, _)| brick.raw().to_vec())
        };
        let moved = before
            .iter()
            .filter(|key| raw(&held, **key) != raw(&grown, **key))
            .count();
        let diff = started.elapsed();
        assert!(before == after && moved == 0);
        line(serde_json::json!({
            "build": build, "board": board_name, "kind": "baseline",
            "grow_ms": grow.as_secs_f64() * 1e3, "diff_ms": diff.as_secs_f64() * 1e3,
            "bricks": grown.brick_count(),
        }));
        drop((grown, held));
        for (pane_name, pane) in [("harness", PANE), ("headed", HEADED_PANE)] {
            let Some(mut board) = sized_or_skip(map.clone(), pane, "the cost receipt") else {
                return;
            };
            let measure = |board: &mut Board, kind: &str| {
                let started = Instant::now();
                board.draw();
                let draw = started.elapsed();
                let (cost, tracer) = (board.cost(), board.terrain());
                let held = cost.residency;
                line(serde_json::json!({
                    "build": build, "board": board_name, "pane": pane_name, "kind": kind,
                    "draw_ms": draw.as_secs_f64() * 1e3,
                    "columns_ms": cost.columns.as_secs_f64() * 1e3,
                    "diff_ms": cost.diff.as_secs_f64() * 1e3,
                    "fill_ms": cost.fill.as_secs_f64() * 1e3,
                    "tiles": cost.tiles, "slots": cost.slots,
                    "upload_bytes": tracer.brick_upload_bytes,
                    "resident": held.resident, "capacity": held.capacity,
                    "loaded": held.loaded, "evicted": held.evicted,
                    "refreshed": held.refreshed, "overflow": held.overflow,
                    "rebuilt": held.rebuilt.map(|why| format!("{why:?}").to_lowercase()),
                }));
            };
            measure(&mut board, "first");
            let targets: Vec<_> = probes(pane)
                .filter_map(|(x, y)| match board.pick(x, y) {
                    Some(BoardPick::Tile { at, .. }) => Some(at),
                    _ => None,
                })
                .take(12)
                .collect();
            for at in &targets {
                board.ui.selected = Some(*at);
                measure(&mut board, "select");
            }
            for at in targets.iter().take(6) {
                let height = board
                    .ui
                    .map
                    .elevation
                    .get(at.0 as u32, at.1 as u32)
                    .copied();
                let raised = height.unwrap_or(0).saturating_add(1);
                board.ui.map.elevation.set(at.0 as u32, at.1 as u32, raised);
                measure(&mut board, "raise");
            }
            for step in 0..12 {
                if step % 2 == 0 {
                    board.ui.camera.0 -= 32.0;
                } else {
                    board.ui.camera.1 -= 16.0;
                }
                measure(&mut board, "pan");
            }
        }
    }
}

/// Headroom over real frames: raising the tile at the pane's centre into the
/// spare layer above the demo board's hill retargets, raising it past that
/// layer rebuilds the map whole, and with no headroom the first raise already
/// rebuilds, which is the control.
#[test]
fn a_raise_within_the_headroom_retargets_and_one_past_it_rebuilds() {
    for headroom in [1, 0] {
        let Some(mut board) = sized_or_skip(demo_map(), PANE, "the headroom receipt") else {
            return;
        };
        board.set_residency(ResidencySettings {
            headroom,
            ..ResidencySettings::default()
        });
        board.draw();
        assert_eq!(
            board.cost().residency.reserved,
            [0, 1 + headroom as i16],
            "the demo's hill tops out in layer 1"
        );
        let Some(BoardPick::Tile { at, .. }) = board.pick(PANE.0 / 2.0, PANE.1 / 2.0) else {
            panic!("the pane's centre shows a tile");
        };

        // Elevation 8 lays its top voxel at 17, in layer 2.
        board.ui.map.elevation.set(at.0 as u32, at.1 as u32, 8);
        board.draw();
        let raised = board.cost();
        eprintln!(
            "headroom {headroom}, {at:?} raised into layer 2: {}",
            raised.line()
        );
        if headroom == 0 {
            assert_eq!(
                raised.residency.rebuilt,
                Some(Rebuild::Headroom),
                "the control"
            );
            assert!(board.terrain().full_map_upload);
            continue;
        }
        assert_eq!(raised.residency.rebuilt, None, "within the headroom");
        assert!(!board.terrain().full_map_upload);
        assert!(
            raised.residency.loaded > 0,
            "the new layer came in as a retarget"
        );
        let world = BoardWorld::new(&board.ui.map);
        let top = world.stand(at, 8);
        let pixel = board
            .camera()
            .pixel_of(top, [PANE.0 as u32, PANE.1 as u32])
            .expect("the raised top is on screen");
        assert!(
            matches!(
                board.pick(pixel[0] as f32 + 0.5, pixel[1] as f32 + 0.5),
                Some(BoardPick::Tile { at: hit, elevation: 8, top: true }) if hit == at
            ),
            "the raised tile is drawn at its new height"
        );

        // Elevation 12 lays its top voxel at 25, in layer 3: past the headroom.
        board.ui.map.elevation.set(at.0 as u32, at.1 as u32, 12);
        board.draw();
        let past = board.cost().residency;
        assert_eq!(past.rebuilt, Some(Rebuild::Headroom));
        assert_eq!(
            past.reserved,
            [0, 4],
            "rebuilt with the headroom above the new top"
        );
        assert!(board.terrain().full_map_upload);
    }
}

/// What each spare layer of headroom costs the pointer volume, at the demo
/// board and at 256 by 256, flat and with relief, in the harness pane and B5's
/// headed pane: one line per board, pane and headroom.
#[test]
#[ignore = "the receipts script runs this"]
fn paging_receipts_headroom() {
    let build = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    for (board_name, map) in [
        ("demo", demo_map()),
        ("flat", synth_map(256, 256)),
        ("relief", relief_map(256)),
    ] {
        let ui = UiState::new(map.clone());
        let overlays = Overlays::of(&ui);
        let centre = (
            map.ground.width() as i32 / 2,
            map.ground.height() as i32 / 2,
        );
        let (x, y) = ui.geo.tile_to_screen(centre, 0);
        for (pane_name, pane) in [("harness", PANE), ("headed", HEADED_PANE)] {
            let camera = BoardWorld::new(&map)
                .camera(&ui.geo, (pane.0 / 2.0 - x, pane.1 / 2.0 - y), pane, None)
                .expect("the board frames the pane");
            for headroom in 0..=3 {
                let settings = ResidencySettings {
                    headroom,
                    ..ResidencySettings::default()
                };
                let mut ground = BoardGround::new(&ui.map, &overlays, 1, settings);
                let (extent, atlas) = {
                    let bricks = ground.bricks();
                    let framed = ground.framed(camera, &bricks);
                    let built = ground
                        .terrain(&bricks, &framed)
                        .brick_map()
                        .expect("a paged map");
                    (built.pointer_extent(), built.atlas().len())
                };
                // Taken as a frame would take it, so the reserve is on the cost.
                ground.uploaded(std::time::Duration::ZERO);
                let pointers: u64 = extent.iter().map(|axis| u64::from(*axis)).product();
                println!(
                    "[paging-receipt] {}",
                    serde_json::json!({
                        "build": build, "board": board_name, "pane": pane_name,
                        "kind": "headroom", "headroom": headroom,
                        "reserved": ground.cost().residency.reserved,
                        "extent": extent, "pointer_bytes": pointers * 4,
                        "atlas_bytes": atlas,
                    })
                );
            }
        }
    }
}
