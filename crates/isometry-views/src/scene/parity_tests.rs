//! The parity gate: the DOM board and the scene board agree on which tile is
//! under a probe pixel.
//!
//! B2's done-condition is not byte parity — the renderers differ — it is that
//! the same tiles are selectable. So this drives one real scene frame over the
//! demo map, then walks a grid of pane pixels asking both paths what is under
//! each: the DOM's [`IsoGeometry::screen_to_tile`] inverse (the geometric
//! fallback B3 retired, arithmetic for arithmetic) and the scene's
//! `Scene::pick` terrain hit mapped back through B1's coordinate convention.
//!
//! B3 added a third receipt below, on the resolver the gestures actually call:
//! two `UiState`s over one frame, one with a `ScenePick` and one without.
//!
//! **Where the two are allowed to differ.** The DOM's inverse is flat-ground
//! picking, documented as such: a click on a raised tile's top face resolves to
//! the tile behind it. The scene's pick is a real ray against real bricks, so
//! over the demo map's hill the two *must* disagree, and that is the DOM path's
//! known limitation rather than a scene bug. The gate is therefore stated on
//! the probes whose scene hit is flat ground — where the two projections are
//! identical by construction (`world.rs`) and nothing excuses a mismatch — and
//! the elevated probes are counted and reported.
//!
//! **The ruled ratio.** The gate above only sees flat ground, so it cannot
//! catch a change to the subdivision itself. `the_ruled_elevation_step_projects_
//! the_doms_own_step` pins it directly, in pixels, and needs no adapter.
//!
//! Without an adapter the probe tests skip **loudly** and assert nothing.

use std::cell::RefCell;
use std::rc::Rc;

use isometer::FrameRequest;
use isometry_core::IsoGeometry;

use super::board::{BoardPick, BoardProducer, BoardSource};
use super::pick::ScenePick;
use super::terrain::{VOXELS_PER_STEP, VOXELS_PER_TILE};
use super::view::BoardView;
use super::world::{elevation_px, world_px};
use crate::demo::demo_map;
use crate::state::UiState;

/// The pane the probe grid is laid over, in logical px, and the texture the
/// scene draws into: render scale 1, so they are the same numbers.
const PANE: (f32, f32) = (640.0, 480.0);
/// Probes per axis. 16 by 16 is 256 pixels spread over the whole pane.
const PROBES: u32 = 16;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

/// The pan that puts the demo map's centre tile in the middle of the pane.
fn centred(geo: &IsoGeometry, map: &isometry_core::MapDocument) -> (f32, f32) {
    let centre = (
        map.ground.width() as i32 / 2,
        map.ground.height() as i32 / 2,
    );
    let (x, y) = geo.tile_to_screen(centre, 0);
    (PANE.0 / 2.0 - x, PANE.1 / 2.0 - y)
}

#[test]
fn the_scene_board_and_the_dom_board_name_the_same_tile_under_a_probe_grid() {
    let Some((device, queue)) = device() else {
        eprintln!(
            "SKIPPED: no wgpu adapter on this machine, so the scene board cannot draw a \
             frame and the parity gate asserts nothing. Run this receipt where a GPU \
             adapter exists."
        );
        return;
    };
    let map = demo_map();
    let geo = IsoGeometry::default();
    let camera = centred(&geo, &map);
    let mut view = BoardView::new(map.clone());
    view.geo = geo;
    view.camera = camera;
    view.pane = PANE;
    let mut source = BoardSource::new(view.into_handle());

    let size = [PANE.0 as u32, PANE.1 as u32];
    let request = FrameRequest {
        device: &device,
        queue: &queue,
        size,
        aspect: PANE.0 / PANE.1,
        color: None,
        needs_frame: true,
        render_scale: 1,
    };
    <BoardSource as isometer::SceneSource>::frame(&mut source, &request)
        .expect("the scene board draws one frame");
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the frame completes");

    let mut flat = 0;
    let mut raised = 0;
    let mut faces = 0;
    let mut empty = 0;
    let mut tokens = 0;
    let mut disagreed = Vec::new();
    for iy in 0..PROBES {
        for ix in 0..PROBES {
            // Probe centres, inset half a cell so no probe sits on an edge.
            let px = (ix as f32 + 0.5) * PANE.0 / PROBES as f32;
            let py = (iy as f32 + 0.5) * PANE.1 / PROBES as f32;
            let ndc = [2.0 * px / PANE.0 - 1.0, 1.0 - 2.0 * py / PANE.1];
            let dom = {
                let at = geo.screen_to_tile((px - camera.0, py - camera.1));
                map.ground.in_bounds(at.0, at.1).then_some(at)
            };
            match source.pick(ndc) {
                None => empty += 1,
                Some(BoardPick::Token(_)) => tokens += 1,
                Some(BoardPick::Tile { at, elevation, top }) => {
                    if !top {
                        // An exposed side face. The DOM board draws one only
                        // where an elevation step exposes it; the scene's
                        // ground is a solid slab, so it has one at the map's
                        // rim and around every void column too.
                        faces += 1;
                    } else if elevation == 0 {
                        flat += 1;
                        if dom != Some(at) {
                            disagreed.push((px, py, dom, at));
                        }
                    } else {
                        raised += 1;
                    }
                },
            }
        }
    }

    eprintln!(
        "parity grid {PROBES}x{PROBES} over {}x{} px: {flat} flat tops, {raised} raised \
         tops, {faces} side faces, {tokens} token, {empty} off-map",
        PANE.0, PANE.1
    );
    assert!(
        flat >= 80,
        "the probe grid must land mostly on the map's flat ground, got {flat}"
    );
    assert!(
        disagreed.is_empty(),
        "{} flat probes name different tiles: {:?}",
        disagreed.len(),
        &disagreed[..disagreed.len().min(8)]
    );
}

/// The token half of the same frame: every token the demo map carries is drawn
/// as a real body, none of them as the placeholder.
#[test]
fn every_demo_token_draws_from_its_own_recipe() {
    let Some((device, queue)) = device() else {
        eprintln!(
            "SKIPPED: no wgpu adapter, so no frame is drawn and the token receipt \
             asserts nothing."
        );
        return;
    };
    let map = demo_map();
    let geo = IsoGeometry::default();
    let camera = centred(&geo, &map);
    let mut view = BoardView::new(map.clone());
    view.geo = geo;
    view.camera = camera;
    view.pane = PANE;
    let mut source = BoardSource::new(view.into_handle());
    let request = FrameRequest {
        device: &device,
        queue: &queue,
        size: [PANE.0 as u32, PANE.1 as u32],
        aspect: PANE.0 / PANE.1,
        color: None,
        needs_frame: true,
        render_scale: 1,
    };
    <BoardSource as isometer::SceneSource>::frame(&mut source, &request)
        .expect("the scene board draws one frame");
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the frame completes");

    assert_eq!(map.tokens.len(), 4, "the demo map's two sides");
    assert_eq!(
        source.placeholders(),
        0,
        "every demo sprite is in the recipe table"
    );
}

/// The cliff-height ruling, in pixels rather than in voxels.
///
/// Mark ruled 5 voxels to a tile and 2 to an elevation step on 2026-09-15
/// because that ratio projects within a fifth of a pixel of the DOM board's
/// own `elev_step`. This is that claim, asserted: change either constant and
/// this fails before anything reaches a capture. A tile's own width is pinned
/// beside it, since the ratio is only right if the denominator is.
#[test]
fn the_ruled_elevation_step_projects_the_doms_own_step() {
    let geo = IsoGeometry::default();
    let step = elevation_px(&geo);
    let tile = world_px(&geo) * VOXELS_PER_TILE as f32 * std::f32::consts::SQRT_2;
    eprintln!(
        "ruled grid {VOXELS_PER_TILE} voxels/tile, {VOXELS_PER_STEP} voxels/step: a step \
         projects {step:.3} px against the DOM's {:.1}, a tile {tile:.3} px against {:.1}",
        geo.elev_step, geo.tile_w
    );
    assert!(
        (step - geo.elev_step).abs() < 1.0,
        "an elevation step projects {step} px, the DOM board's is {}",
        geo.elev_step
    );
    assert!(
        (tile - geo.tile_w).abs() < 0.01,
        "a tile projects {tile} px wide, the DOM board's is {}",
        geo.tile_w
    );
    // And the error must not accumulate past a pixel over a real board: the
    // demo map's crown stands four steps up.
    let ceiling = demo_map()
        .elevation
        .iter()
        .map(|(_, _, e)| *e)
        .max()
        .unwrap_or(0) as f32;
    let drift = (step - geo.elev_step).abs() * ceiling;
    eprintln!("over the demo map's {ceiling} steps the two paths drift {drift:.3} px");
    assert!(drift < 1.0, "the full height range drifts {drift} px");
}

/// The retirement's own receipt (B3): the two arms of `UiState::board_at`
/// answer the same thing where the DOM board is not wrong by construction.
///
/// B3 retired `tile_at_pane` and `token_drag_candidate` as reachable paths on
/// the scene board. What made that safe is this: one `UiState` with a
/// [`ScenePick`] over a real frame and one without, over the same map, camera
/// and pane, asked the same pane-local points. On flat ground they agree tile
/// for tile. Where they differ — a raised top face — **the scene is right**:
/// it names the tile you can see, and the DOM's flat-ground inverse names the
/// tile behind it, as `tile_at_pane` documented of itself. So the mismatches
/// are counted and their direction is asserted, rather than excused.
#[test]
fn both_arms_of_the_resolver_name_the_same_tile_on_flat_ground() {
    let Some((device, queue)) = device() else {
        eprintln!(
            "SKIPPED: no wgpu adapter, so no frame is drawn and the resolver receipt \
             asserts nothing."
        );
        return;
    };
    let map = demo_map();
    let geo = IsoGeometry::default();
    let camera = centred(&geo, &map);

    let mut view = BoardView::new(map.clone());
    view.geo = geo;
    view.camera = camera;
    view.pane = PANE;
    let producer = Rc::new(RefCell::new(BoardProducer::new(BoardSource::new(
        view.into_handle(),
    ))));
    let request = FrameRequest {
        device: &device,
        queue: &queue,
        size: [PANE.0 as u32, PANE.1 as u32],
        aspect: PANE.0 / PANE.1,
        color: None,
        needs_frame: true,
        render_scale: 1,
    };
    <BoardSource as isometer::SceneSource>::frame(producer.borrow_mut().source_mut(), &request)
        .expect("the scene board draws one frame");
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the frame completes");

    let board = |scene: bool| {
        let mut ui = UiState::new(map.clone());
        ui.geo = geo;
        ui.camera = camera;
        ui.viewport = PANE;
        if scene {
            ui.scene_board = true;
            ui.board_pick = Some(ScenePick::new(producer.clone()));
        }
        ui
    };
    let (dom, scene) = (board(false), board(true));

    let (mut agreed, mut raised, mut tokens, mut faces) = (0, 0, 0, 0);
    let (mut differed, mut behind) = (0, 0);
    let mut disagreed = Vec::new();
    for iy in 0..PROBES {
        for ix in 0..PROBES {
            let px = (ix as f32 + 0.5) * PANE.0 / PROBES as f32;
            let py = (iy as f32 + 0.5) * PANE.1 / PROBES as f32;
            match scene.board_at((px, py)) {
                None => {},
                Some(BoardPick::Token(id)) => {
                    tokens += 1;
                    // A body pick is identity the DOM's inverse cannot reach
                    // at all; what must hold is that it names a live token.
                    assert!(map.token(id).is_some(), "the pick named a live token");
                },
                Some(BoardPick::Tile { at, elevation, top }) if top && elevation == 0 => {
                    if dom.tile_at((px, py)) == Some(at) {
                        agreed += 1;
                    } else {
                        disagreed.push((px, py, dom.tile_at((px, py)), at));
                    }
                },
                Some(BoardPick::Tile { at, top, .. }) => {
                    if !top {
                        faces += 1;
                        continue;
                    }
                    raised += 1;
                    let Some(dom_at) = dom.tile_at((px, py)) else {
                        continue;
                    };
                    if dom_at == at {
                        continue;
                    }
                    // The documented direction. A raised tile is drawn higher
                    // up the screen than its own flat position, and screen y
                    // grows with `col + row`, so a flat inverse of a pixel on
                    // its top face lands on a tile further back: a strictly
                    // smaller sum. Anything else is not the known limitation.
                    differed += 1;
                    behind += usize::from(dom_at.0 + dom_at.1 < at.0 + at.1);
                },
            }
        }
    }
    eprintln!(
        "resolver arms over {PROBES}x{PROBES} probes: {agreed} flat tiles agreed, \
         {raised} raised tops of which {differed} differ ({behind} with the DOM naming \
         a tile behind the visible one), {faces} side faces, {tokens} token picks"
    );
    assert!(
        agreed >= 80,
        "the probe grid must land on flat ground: {agreed}"
    );
    assert!(
        disagreed.is_empty(),
        "{} flat probes disagree: {:?}",
        disagreed.len(),
        &disagreed[..disagreed.len().min(8)]
    );
    assert_eq!(
        behind, differed,
        "every raised probe the two arms differ on must be the DOM naming a tile behind"
    );
}
