//! The snapshot's cached tallest tile, against a scan of the grid after every
//! kind of edit: the camera must come out the same either way.

use super::view::BoardView;
use super::world::{BoardWorld, tallest};
use crate::demo::{demo_map, synth_map};
use crate::state::UiState;

/// The cache is the scan, and the world built on it is the scanned world.
fn assert_current(view: &BoardView, what: &str) {
    assert_eq!(
        view.tallest(),
        tallest(&view.map),
        "{what}: the cache is the scan"
    );
    assert_eq!(
        BoardWorld::with_tallest(&view.map, view.tallest()),
        BoardWorld::new(&view.map),
        "{what}: and so is the world"
    );
}

fn step(ui: &mut UiState, view: &mut BoardView, edits: &[(u32, u32, u8)], what: &str) -> u8 {
    for (col, row, height) in edits {
        ui.map.elevation.set(*col, *row, *height);
    }
    view.sync(ui);
    assert_current(view, what);
    view.tallest()
}

#[test]
fn the_cached_tallest_tile_follows_every_edit() {
    let mut ui = UiState::new(synth_map(40, 40));
    let mut view = BoardView::new(ui.map.clone());
    view.sync(&ui);
    assert_current(&view, "a new board");
    let first = view.tallest();
    assert_eq!(first, 3, "the stress board's hills stand three steps");

    assert_eq!(
        step(&mut ui, &mut view, &[(5, 5, 9)], "raised past the top"),
        9
    );
    assert_eq!(
        step(
            &mut ui,
            &mut view,
            &[(30, 30, 0), (31, 30, 1)],
            "lowered below it"
        ),
        9
    );
    assert_eq!(
        step(&mut ui, &mut view, &[(5, 5, 2)], "the tallest lowered"),
        first,
        "lowering the tallest tile finds the next one down"
    );
    assert_eq!(
        step(
            &mut ui,
            &mut view,
            &[(1, 1, 7), (2, 1, 7)],
            "two tied at the top"
        ),
        7
    );
    assert_eq!(
        step(&mut ui, &mut view, &[(1, 1, 0)], "one of the two lowered"),
        7
    );
    assert_eq!(
        step(&mut ui, &mut view, &[(2, 1, 0)], "the other lowered"),
        first
    );

    let every: Vec<(u32, u32, u8)> = ui
        .map
        .elevation
        .iter()
        .map(|(col, row, _)| (col, row, 0))
        .collect();
    assert_eq!(step(&mut ui, &mut view, &every, "flattened"), 0);

    // A paint moves no height: the cache is not rescanned and stays right.
    let stone = ui.map.intern_tile_kind("stone");
    ui.map.ground.set(4, 4, stone);
    view.sync(&ui);
    assert_current(&view, "painted");

    ui.map = demo_map();
    view.sync(&ui);
    assert_current(&view, "another board");
    assert!(view.tallest() > 0, "the demo board stands a hill");
}
