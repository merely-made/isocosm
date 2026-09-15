//! The parity gate: the DOM board and the scene board agree on which tile is
//! under a probe pixel.
//!
//! B2's done-condition is not byte parity — the renderers differ — it is that
//! the same tiles are selectable. So this drives one real scene frame over the
//! demo map, then walks a grid of pane pixels asking both paths what is under
//! each: the DOM's [`IsoGeometry::screen_to_tile`] inverse (through the same
//! `tile_at_pane` arithmetic the board uses) and the scene's `Scene::pick`
//! terrain hit mapped back through B1's coordinate convention.
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
//! Without an adapter the test skips **loudly** and asserts nothing.

use isometer::FrameRequest;
use isometry_core::IsoGeometry;

use super::board::{BoardPick, BoardSource, BoardView};
use crate::demo::demo_map;

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
