//! What the board draws at, in pane pixels: the receipt that pins the scene
//! board's size against the DOM board's (B4 item 1), and the focus cutaway.
//!
//! B3 found `BoardWorld::camera` taking its centre from the pane and its half
//! height from the **texture**, which is the pane times the device scale times
//! the interface zoom over the render scale. On this laptop that is
//! `2 / (2 * 0.9171)` = 1.0905, and the scene board drew nine per cent large.
//! The fix is that the texture is not an input at all, and this is what says
//! so: every grid below computes the texture the producer would hand the
//! source and then asserts the camera draws a tile at exactly the DOM board's
//! own `tile_w`, in the pane's pixels, for every one of them.
//!
//! No adapter is needed. The claim is about a projection, and the projection
//! is [`SlabCamera::ndc_of`], which is `clip_from_world`'s own arithmetic —
//! the matrix the draw writes with.

use isometer::SlabCamera;
use isometry_core::{IsoGeometry, MapDocument, TileCoord};

use super::terrain::VOXELS_PER_STEP;
use super::world::{BoardWorld, elevation_px, focus_top};
use crate::demo::demo_map;

/// The pane the board is framed in, logical px.
const PANE: (f32, f32) = (971.0, 820.0);

/// `(device scale, interface zoom, render scale)` triples the board is drawn
/// under. The second is this laptop's own — device 2 at the design fit's
/// 0.9171 zoom, which rounds to a render scale of 2 and is exactly the grid
/// every capture under `testing/isometry/scene-board*` was taken at.
const GRIDS: [(f32, f32, u32); 4] = [
    (1.0, 1.0, 1),
    (2.0, 0.917_073_2, 2),
    (1.5, 1.0, 2),
    (1.25, 1.1, 1),
];

/// Where a world point lands in the pane, in the pane's own logical pixels
/// measured from its top left — which is where the presented texture lands,
/// whatever internal resolution it was drawn at.
fn pane_px(camera: &SlabCamera, point: [f32; 3]) -> (f32, f32) {
    let ndc = camera.ndc_of(point).expect("the camera frames the board");
    ((ndc[0] + 1.0) * PANE.0 / 2.0, (1.0 - ndc[1]) * PANE.1 / 2.0)
}

/// The texture the producer hands the source at one pixel grid: the leaf's
/// physical size floored by the render scale, exactly `FrameRequest::for_scene`.
fn texture(device: f32, zoom: f32, scale: u32) -> [u32; 2] {
    let physical = |logical: f32| (logical * device * zoom) as u32;
    [
        (physical(PANE.0) / scale).max(1),
        (physical(PANE.1) / scale).max(1),
    ]
}

fn board() -> (MapDocument, IsoGeometry, BoardWorld) {
    let map = demo_map();
    let geo = IsoGeometry::default();
    let world = BoardWorld::new(&map);
    (map, geo, world)
}

/// A tile drawn through the scene is the DOM board's own `tile_w` wide and
/// `tile_h` tall, in pane pixels, at every pixel grid — including the
/// fractional ones, which is the whole of what B3 found.
#[test]
fn the_scene_board_draws_the_doms_own_tile_at_every_pixel_grid() {
    let (_, geo, world) = board();
    let at: TileCoord = (12, 12);
    for (device, zoom, scale) in GRIDS {
        let size = texture(device, zoom, scale);
        let camera = world
            .camera(&geo, (420.0, 140.0), PANE, None)
            .expect("the board frames the pane");
        let centre = pane_px(&camera, world.stand(at, 0));
        let column = pane_px(&camera, world.stand((at.0 + 1, at.1), 0));
        let row = pane_px(&camera, world.stand((at.0, at.1 + 1), 0));
        let width = column.0 - row.0;
        let height = (column.1 - centre.1) + (row.1 - centre.1);
        eprintln!(
            "device {device} zoom {zoom} scale {scale}: the scene draws into a \
             {}x{} texture and a tile {width:.4} x {height:.4} px against the DOM's \
             {} x {}",
            size[0], size[1], geo.tile_w, geo.tile_h
        );
        assert!(
            (width - geo.tile_w).abs() < 0.01,
            "a tile drew {width} px wide against the DOM board's {}",
            geo.tile_w
        );
        assert!(
            (height - geo.tile_h).abs() < 0.01,
            "a tile drew {height} px tall against the DOM board's {}",
            geo.tile_h
        );
    }
}

/// And a flat tile lands on the exact pane pixel the DOM board's projection
/// puts it on, pan included: one board over the other, not merely one scale.
#[test]
fn a_flat_tile_lands_where_the_dom_board_draws_it() {
    let (map, geo, world) = board();
    let pan = (420.0, 140.0);
    let camera = world
        .camera(&geo, pan, PANE, None)
        .expect("the board frames the pane");
    let mut worst: f32 = 0.0;
    for (col, row, kind) in map.ground.iter() {
        if kind.0 == 0 || *map.elevation.get(col, row).unwrap_or(&0) != 0 {
            continue;
        }
        let at = (col as i32, row as i32);
        let (sx, sy) = geo.tile_to_screen(at, 0);
        let drawn = pane_px(&camera, world.stand(at, 0));
        worst = worst
            .max((drawn.0 - (sx + pan.0)).abs())
            .max((drawn.1 - (sy + pan.1)).abs());
    }
    eprintln!("the two boards' flat tiles are at most {worst:.4} px apart");
    assert!(
        worst < 0.05,
        "a flat tile drew {worst} px off the DOM board"
    );
}

/// An elevation step rises the ruled `ELEVATION_PX` and nothing horizontal,
/// measured in the same pane pixels rather than in world units.
#[test]
fn an_elevation_step_rises_the_ruled_pixels_in_the_pane() {
    let (_, geo, world) = board();
    let camera = world
        .camera(&geo, (420.0, 140.0), PANE, None)
        .expect("the board frames the pane");
    let low = pane_px(&camera, world.stand((12, 12), 0));
    let high = pane_px(&camera, world.stand((12, 12), 1));
    assert!(
        (high.0 - low.0).abs() < 0.01,
        "a step moves nothing sideways"
    );
    let rise = low.1 - high.1;
    eprintln!(
        "one step rises {rise:.3} px, the ruled {:.3}, the DOM board's {}",
        elevation_px(&geo),
        geo.elev_step
    );
    assert!((rise - elevation_px(&geo)).abs() < 0.01);
    assert!((rise - geo.elev_step).abs() < 0.2, "within the ruling");
}

/// A focus elevation lowers the cut to the ground it keeps, clearing a body's
/// own height above it — so the pieces standing *on* the focus layer are whole
/// and anything far above cannot reach a pixel. What stands one step up is
/// kept out by not being drawn (`Overlays::cuts`), not by this box.
#[test]
fn a_focus_elevation_lowers_the_cut_to_the_ground_it_keeps() {
    let (_, geo, world) = board();
    let whole = world
        .camera(&geo, (0.0, 0.0), PANE, None)
        .expect("the board frames the pane");
    let focused = world
        .camera(&geo, (0.0, 0.0), PANE, Some(1))
        .expect("the board frames the pane");
    let ceiling = |camera: &SlabCamera| match camera.cutaway {
        Some(isometer::Cutaway::Bounds { max, .. }) => max[1],
        other => panic!("the board always cuts to its own box, got {other:?}"),
    };
    assert!(
        ceiling(&focused) > focus_top(1),
        "the cut clears the pieces standing on the ground it keeps"
    );
    assert!(
        ceiling(&focused) < ceiling(&whole),
        "and still cuts lower than the whole board's ceiling"
    );
    // And the ruled grid puts one step between two focus heights.
    assert_eq!(
        focus_top(2) - focus_top(1),
        VOXELS_PER_STEP as f32,
        "a focus step is an elevation step"
    );
}
