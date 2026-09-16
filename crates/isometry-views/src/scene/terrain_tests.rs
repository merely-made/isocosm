//! B1's receipts at the subdivided scale: the grown ground agrees with the map
//! over every cell's whole `VOXELS_PER_TILE` footprint, an elevation step is
//! `VOXELS_PER_STEP` voxels tall, and the material palette agrees with the
//! stylesheet it was built from.

use super::terrain::{
    MapTerrain, SEA_LEVEL, VOID_SURFACE, VOXELS_PER_STEP, VOXELS_PER_TILE, material_of, surface_of,
    terrain_palette,
};
use isometer_core::ground::{BRICK, Ground};
use isometry_core::{MapDocument, TileKindId};

use crate::demo::{demo_map, synth_map};
use crate::theme::{board_css, tile_kind_colour};

fn material_at(ground: &Ground, at: [i32; 3]) -> u8 {
    let key = [
        at[0].div_euclid(BRICK) as i16,
        at[1].div_euclid(BRICK) as i16,
        at[2].div_euclid(BRICK) as i16,
    ];
    ground.brick_materials(key).map_or(0, |(brick, origin)| {
        brick.get([at[0] - origin[0], at[1] - origin[1], at[2] - origin[2]])
    })
}

/// Every voxel of every painted cell's footprint stands at that cell's surface
/// and carries that cell's kind. The whole map is inside the grown extent.
fn check_every_cell(map: &MapDocument) {
    let terrain = MapTerrain::new(map);
    let ground = terrain.grow();
    assert_eq!(ground.extent(), terrain.extent());
    for (col, row, kind) in map.ground.iter() {
        let (x0, z0) = terrain.column(col, row);
        let elevation = *map.elevation.get(col, row).unwrap() as i32;
        let top = surface_of(elevation);
        let expected = material_of(*kind);
        // The whole footprint, not just its low corner: the subdivision's own
        // claim is that a cell owns 5 by 5 columns of one material.
        for dz in 0..VOXELS_PER_TILE {
            for dx in 0..VOXELS_PER_TILE {
                let (x, z) = (x0 + dx, z0 + dz);
                assert!(
                    x.abs() <= terrain.extent() && z.abs() <= terrain.extent(),
                    "cell ({col}, {row}) +({dx}, {dz}) fell outside the extent"
                );
                assert_eq!(terrain.cell(x, z), Some((col, row)), "round trip");
                if kind.0 == 0 {
                    assert_eq!(ground.surface(x, z), None, "empty cell ({col}, {row})");
                    continue;
                }
                assert_eq!(
                    ground.surface(x, z),
                    Some(top),
                    "surface at ({col}, {row}) +({dx}, {dz})"
                );
                for y in 0..=top {
                    assert_eq!(
                        material_at(&ground, [x, y, z]),
                        expected,
                        "material at ({col}, {row}) +({dx}, {dz}) y {y}"
                    );
                }
                assert_eq!(material_at(&ground, [x, top + 1, z]), 0, "air above");
            }
        }
    }
}

/// The ruled grid itself: a tile is five voxels across, a step two tall, and
/// every other number here is derived from the pair.
#[test]
fn the_ruled_grid_is_five_voxels_to_a_tile_and_two_to_a_step() {
    assert_eq!((VOXELS_PER_TILE, VOXELS_PER_STEP), (5, 2));
    assert_eq!(surface_of(0), 1, "a flat cell is two voxels deep");
    assert_eq!(surface_of(3) - surface_of(2), VOXELS_PER_STEP);
    // A step of ground is shorter than a tile is wide; that is the point.
    assert!(VOXELS_PER_STEP < VOXELS_PER_TILE);
}

/// A neighbouring cell one elevation step up stands exactly two voxels higher,
/// so the cliff between them is a two-voxel face rather than a five-voxel one.
#[test]
fn one_elevation_step_is_two_voxels_of_cliff() {
    let mut map = MapDocument::new("step", 2, 1);
    let grass = map.intern_tile_kind("grass");
    map.ground.set(0, 0, grass);
    map.ground.set(1, 0, grass);
    map.elevation.set(1, 0, 1);
    let terrain = MapTerrain::new(&map);
    let ground = terrain.grow();
    let (low, z) = terrain.column(0, 0);
    let (high, _) = terrain.column(1, 0);
    assert_eq!(high - low, VOXELS_PER_TILE, "tiles are five columns apart");
    assert_eq!(ground.surface(low, z), Some(surface_of(0)));
    assert_eq!(ground.surface(high, z), Some(surface_of(1)));
    assert_eq!(
        ground.surface(high, z).unwrap() - ground.surface(low, z).unwrap(),
        VOXELS_PER_STEP
    );
}

#[test]
fn the_demo_map_grows_a_ground_that_matches_it() {
    let map = demo_map();
    check_every_cell(&map);
    // 24 tiles centred on 12 reach voxel columns -60 ..= 59, so the square
    // bound is 60: five times the old extent of 12.
    assert_eq!(MapTerrain::new(&map).extent(), 60, "a 24x24 map centres");
}

#[test]
fn a_synthetic_board_grows_the_same_way() {
    check_every_cell(&synth_map(30, 30));
}

/// The stylesheet is the authority: the palette entry a kind's material reads
/// is the colour the generated `.tile-<kind>` rule carries.
#[test]
fn the_palette_carries_the_stylesheets_colours() {
    let map = demo_map();
    let palette = terrain_palette(&map);
    let css = board_css();
    for (index, kind) in map.tile_kinds.iter().enumerate() {
        let material = material_of(TileKindId(index as u16));
        let entry = palette.colour(material).expect("a non-empty palette");
        match tile_kind_colour(kind) {
            Some(colour) => {
                assert_eq!(entry, colour, "palette entry for {kind}");
                let rule = format!(".tile-{kind} {{ background-color: ");
                let at = css
                    .find(&rule)
                    .unwrap_or_else(|| panic!("no rule for {kind}"));
                let hex = &css[at + rule.len()..at + rule.len() + 7];
                assert_eq!(
                    colour,
                    [
                        u8::from_str_radix(&hex[1..3], 16).unwrap() as f32 / 255.0,
                        u8::from_str_radix(&hex[3..5], 16).unwrap() as f32 / 255.0,
                        u8::from_str_radix(&hex[5..7], 16).unwrap() as f32 / 255.0,
                    ],
                    "{kind} disagrees with its stylesheet rule {hex}"
                );
            },
            // `tree` is a prop kind: the stylesheet gives it no tile colour, so
            // it reads as the unknown entry rather than as a plausible one.
            None => assert_eq!(entry, [0.0, 0.0, 0.0], "unstyled kind {kind}"),
        }
    }
    assert_eq!(
        palette.colour(0),
        Some([0.0, 0.0, 0.0]),
        "entry 0 is unknown"
    );
}

/// Off the map and empty on it are one thing: a column below the water line
/// with nothing laid in it.
#[test]
fn empty_and_off_map_columns_sit_below_sea_level() {
    let mut map = MapDocument::new("edge", 4, 4);
    let grass = map.intern_tile_kind("grass");
    map.ground.set(1, 1, grass);
    map.elevation.set(1, 1, 3);
    let terrain = MapTerrain::new(&map);
    let ground = terrain.grow();

    assert_eq!(ground.sea_level, SEA_LEVEL);
    assert!(VOID_SURFACE < SEA_LEVEL, "the void is under the water line");
    assert!(VOID_SURFACE < 0, "and so lays nothing at all");

    let off = terrain.extent();
    assert_eq!(terrain.cell(off, off), None, "the corner is off the map");
    assert_eq!(ground.surface(off, off), None, "and holds no voxel");
    assert_eq!(material_at(&ground, [off, 0, off]), 0);

    // An empty cell inside the map is the same void, over its whole footprint.
    let (x, z) = terrain.column(0, 0);
    for dz in 0..VOXELS_PER_TILE {
        for dx in 0..VOXELS_PER_TILE {
            assert_eq!(ground.surface(x + dx, z + dz), None);
        }
    }

    // The one painted cell stands alone at its elevation, corner to corner.
    let (x, z) = terrain.column(1, 1);
    let top = surface_of(3);
    assert_eq!(ground.surface(x, z), Some(top));
    assert_eq!(material_at(&ground, [x, top, z]), material_of(grass));
    let far = VOXELS_PER_TILE - 1;
    assert_eq!(
        material_at(&ground, [x + far, top, z + far]),
        material_of(grass),
        "the far corner of the same cell"
    );
}

/// Props are counted here and adapted in a later lane.
#[test]
fn props_are_counted_not_grown() {
    let map = demo_map();
    let terrain = MapTerrain::new(&map);
    let expected = map.props.iter().filter(|(_, _, k)| k.0 != 0).count();
    assert!(expected > 0, "the demo board carries trees");
    assert_eq!(terrain.prop_count(), expected);
}
