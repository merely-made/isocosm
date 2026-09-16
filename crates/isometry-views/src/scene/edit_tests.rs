//! B4's edit and focus receipts, over real frames.
//!
//! An elevation edit must reach the next frame as **slots** rather than as a
//! whole map: `Ground::drain_dirty` is the seam the slot upload reads, and
//! `BoardGround` fills it by comparing the regrown bricks against the held
//! ones. What these assert is the tracer's own receipt —
//! `BrickDiagnostics::changed_slots_declared` and `brick_upload_bytes` — so
//! the claim is what the GPU was actually asked to do, not what this crate
//! believes it asked for.
//!
//! They also *show* the thing B2 could not have known was wrong, rather than
//! asserting it: `the_shared_ground_terrain_would_have_skipped_the_upload`
//! drives the path B2 took — a regrown `Ground` bound through
//! [`isometer::GroundTerrain`] — and reads `map_revision_unchanged` back off
//! the tracer. A grown ground's own revision is zero for its whole life
//! (growth is the world's starting fact, only `carve` bumps it), so the
//! residency recognised the map it already held and uploaded nothing. That is
//! the whole reason [`super::ground::BoardTerrain`] exists.
//!
//! Without an adapter every receipt here skips **loudly**.

//! The fixture the receipts run on moved to [`super::harness`] when B5's cost
//! receipts needed the same one; nothing about what they assert changed.

use isometer::lens::{BrickMap, Grade};
use isometer::{GroundTerrain, Scene, SceneFrame, SceneHost, SceneVolumes, VolumeMap};

use super::board::BoardPick;
use super::harness::{Board, PANE, board_or_skip, device, flat_tile};
use super::terrain::MapTerrain;
use super::world::BoardWorld;
use crate::demo::demo_map;
use crate::state::UiState;

/// One elevation edit uploads its own bricks and not the board's.
#[test]
fn an_elevation_edit_reaches_the_next_frame_as_slots() {
    let mut board = board_or_skip!("the incremental edit receipt");
    board.draw();
    let first = board.terrain();
    let whole = first.brick_upload_bytes;
    assert!(first.full_map_upload, "the first frame binds the whole map");
    eprintln!(
        "first frame: the whole map is {whole} bytes over {} bricks, {}",
        board.cost().bricks,
        board.cost().line()
    );

    let at = flat_tile(&board.ui);
    board.ui.map.elevation.set(at.0 as u32, at.1 as u32, 1);
    board.draw();
    let edit = board.terrain();
    let cost = board.cost();
    eprintln!(
        "one elevation edit at {at:?}: {}, tracer declared {} slots and uploaded {} bytes \
         against the whole map's {whole}",
        cost.line(),
        edit.changed_slots_declared,
        edit.brick_upload_bytes,
    );
    assert!(
        !edit.map_revision_unchanged,
        "the board's own revision moved, so the tracer did not skip the upload"
    );
    assert!(!edit.full_map_upload, "and it did not re-upload the board");
    assert_eq!(
        Some(edit.changed_slots_declared),
        cost.slots,
        "every brick the diff found is a slot the tracer was given"
    );
    assert!(edit.changed_slots_declared > 0, "an edit changes bricks");
    assert!(
        edit.brick_upload_bytes * 4 < whole,
        "the edit uploaded {} bytes where the whole map is {whole}",
        edit.brick_upload_bytes
    );

    // And the edit is on the board: the pick reads the drawn frame, so a tile
    // that stood at height 0 now stands at 1.
    let raised = board
        .ui
        .map
        .elevation
        .get(at.0 as u32, at.1 as u32)
        .copied();
    assert_eq!(raised, Some(1));
}

/// An overlay is an edit of the same kind: selecting a tile repaints one
/// tile's top voxel and uploads that tile's bricks.
#[test]
fn selecting_a_tile_uploads_only_the_bricks_it_tints() {
    let mut board = board_or_skip!("the overlay upload receipt");
    board.draw();
    let whole = board.terrain().brick_upload_bytes;

    board.ui.selected = Some(flat_tile(&board.ui));
    board.draw();
    let tinted = board.terrain();
    let cost = board.cost();
    eprintln!(
        "one selected tile: {}, {} slots and {} bytes against the whole map's {whole}",
        cost.line(),
        tinted.changed_slots_declared,
        tinted.brick_upload_bytes,
    );
    assert!(!tinted.full_map_upload && !tinted.map_revision_unchanged);
    assert!(tinted.changed_slots_declared > 0);
    assert!(tinted.brick_upload_bytes * 4 < whole);
}

/// A frame nothing moved on costs no upload at all: the skip above it is the
/// producer's, and this is the tracer's own.
#[test]
fn a_still_board_uploads_nothing() {
    let mut board = board_or_skip!("the still-frame receipt");
    board.draw();
    board.draw();
    let still = board.terrain();
    eprintln!(
        "a still frame: revision unchanged {}, {} bytes",
        still.map_revision_unchanged, still.brick_upload_bytes
    );
    assert!(
        still.map_revision_unchanged,
        "nothing moved, nothing uploads"
    );
    assert_eq!(still.brick_upload_bytes, 0);
}

/// A focus elevation hides the layers above it: the ground is refiltered, the
/// map is rebuilt whole, and no pixel of the drawn frame shows raised ground
/// any more.
#[test]
fn a_focus_elevation_hides_the_layers_above_it() {
    let mut board = board_or_skip!("the focus receipt");
    board.draw();

    // Probe the frame for pixels that show ground above height 0.
    let probes: Vec<(f32, f32)> = (0..48)
        .flat_map(|iy| {
            (0..48).map(move |ix| {
                (
                    (ix as f32 + 0.5) * PANE.0 / 48.0,
                    (iy as f32 + 0.5) * PANE.1 / 48.0,
                )
            })
        })
        .collect();
    let raised = |board: &Board| {
        probes
            .iter()
            .filter(|(px, py)| {
                matches!(
                    board.pick(*px, *py),
                    Some(BoardPick::Tile { elevation, .. }) if elevation > 0
                )
            })
            .count()
    };
    let pieces = |board: &Board| {
        let mut seen: Vec<u32> = probes
            .iter()
            .filter_map(|(px, py)| match board.pick(*px, *py) {
                Some(BoardPick::Token(id)) => Some(id.0),
                _ => None,
            })
            .collect();
        seen.sort_unstable();
        seen.dedup();
        seen
    };
    let before = raised(&board);
    let standing = pieces(&board);
    assert!(before > 0, "the demo map stands a hill in the frame");
    assert!(!standing.is_empty(), "and pieces on it");

    board.ui.focus_elevation = Some(0);
    board.draw();
    let cost = board.cost();
    let after = raised(&board);
    eprintln!(
        "focus to height 0: {}, tracer uploaded {} bytes; raised pixels {before} -> {after}",
        cost.line(),
        board.terrain().brick_upload_bytes,
    );
    assert_eq!(cost.slots, None, "a focus change rebuilds the map whole");
    assert!(board.terrain().full_map_upload);
    assert_eq!(after, 0, "nothing above the focus reaches a pixel");

    // The pieces are cut the same way, and *only* the ones above the focus:
    // a token standing on the ground a focus keeps is drawn whole, which a
    // plane laid on that ground would have sliced off at the ankles.
    let kept = pieces(&board);
    let above: Vec<u32> = board
        .ui
        .map
        .tokens
        .iter()
        .filter(|token| {
            *board
                .ui
                .map
                .elevation
                .get(token.at.0 as u32, token.at.1 as u32)
                .unwrap_or(&0)
                > 0
        })
        .map(|token| token.id.0)
        .collect();
    eprintln!("pieces on screen {standing:?} -> {kept:?}, above the focus {above:?}");
    assert!(
        !kept.is_empty(),
        "the pieces on the focus layer are still drawn"
    );
    for id in &above {
        assert!(!kept.contains(id), "token {id} stands above the focus");
    }

    // And stepping the focus back off restores the hill.
    board.ui.focus_elevation = None;
    board.draw();
    let restored = raised(&board);
    eprintln!(
        "focus off: {}; raised pixels back to {restored}",
        board.cost().line()
    );
    assert_eq!(restored, before, "the whole board is back");
}

/// The focus cycle is its own way out: it walks the map's heights and then
/// returns to the whole board, so the key that entered it leaves it.
#[test]
fn the_focus_cycle_returns_to_the_whole_board() {
    let mut ui = UiState::new(demo_map());
    let tallest = ui
        .map
        .elevation
        .iter()
        .map(|(_, _, e)| *e as i32)
        .max()
        .unwrap_or(0);
    assert!(tallest > 0, "the demo map stands a hill");
    for height in 0..=tallest {
        ui.cycle_focus_elevation();
        assert_eq!(ui.focus_elevation, Some(height));
    }
    ui.cycle_focus_elevation();
    assert_eq!(ui.focus_elevation, None, "past the top is the whole board");
}

/// The path B2 took, shown rather than argued: a regrown ground bound through
/// the shared [`GroundTerrain`] never reaches the tracer at all.
///
/// The positive control is in the same run: the first frame *does* upload, so
/// the instrument works, and the second — over a ground grown from an edited
/// map — is recognised as the map already resident. `Ground::revision()` is
/// zero on both, because growing a ground is not editing one.
#[test]
fn the_shared_ground_terrain_would_have_skipped_the_upload() {
    let Some((device, queue)) = device() else {
        eprintln!("SKIPPED: no wgpu adapter, so the GroundTerrain receipt asserts nothing.");
        return;
    };
    let mut map = demo_map();
    let world = BoardWorld::new(&map);
    let geo = isometry_core::IsoGeometry::default();
    let camera = world
        .camera(&geo, (0.0, 0.0), PANE, None)
        .expect("the board frames the pane");
    let mut scene =
        Scene::new(device.clone(), queue.clone(), PANE.0 as u32, PANE.1 as u32).expect("a scene");

    struct Plain;
    impl SceneHost for Plain {}
    let volumes = VolumeMap::new();
    let draw = |scene: &mut Scene, ground: &isometer::core::ground::Ground| {
        let terrain = GroundTerrain(ground);
        let mut encoder = device.create_command_encoder(&Default::default());
        scene
            .render(
                &mut encoder,
                SceneFrame {
                    camera,
                    bodies: &[],
                    volumes: SceneVolumes::Voxels(&volumes),
                    terrain: Some(&terrain as &dyn isometer::TerrainSource),
                    dirty: &[],
                    grade: Grade::clay(),
                    terrain_appearance: None,
                    body_budget: 16,
                    capsules: None,
                },
                &mut Plain,
            )
            .expect("the frame encodes");
        queue.submit(Some(encoder.finish()));
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("the frame completes");
        scene.terrain_diagnostics().expect("a terrain receipt")
    };

    let ground = MapTerrain::new(&map).grow();
    assert_eq!(ground.revision(), 0, "growing a ground is not editing one");
    let first = draw(&mut scene, &ground);
    assert!(
        first.brick_upload_bytes > 0 && !first.map_revision_unchanged,
        "the control: a first frame really does upload its map"
    );

    // B2's edit path: regrow the whole ground and bind the map it builds.
    let at = flat_tile(&UiState::new(map.clone()));
    map.elevation.set(at.0 as u32, at.1 as u32, 1);
    let edited = MapTerrain::new(&map).grow();
    assert_eq!(edited.revision(), 0, "and neither is regrowing one");
    scene.set_terrain_map(BrickMap::from_ground(&edited).expect("a brick map"));
    let second = draw(&mut scene, &edited);
    eprintln!(
        "GroundTerrain after an edit: revision unchanged {}, full upload {}, {} bytes",
        second.map_revision_unchanged, second.full_map_upload, second.brick_upload_bytes
    );
    assert!(
        second.map_revision_unchanged,
        "the tracer held the map it already had, so the edit never reached a pixel"
    );
    assert_eq!(second.brick_upload_bytes, 0);
}
