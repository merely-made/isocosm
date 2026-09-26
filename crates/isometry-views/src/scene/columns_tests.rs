//! The columns against the ground they replace: every brick the board makes on
//! demand is the brick `MapTerrain::grow` lays, and a change names every brick
//! whose bytes it moved.

use std::collections::{BTreeMap, BTreeSet};

use isometer::{BRICK_BYTES, BrickSource};
use isometry_core::MapDocument;

use super::columns::{BoardBricks, TileColumns};
use super::harness::relief_map;
use super::overlay::Overlays;
use super::terrain::{MapTerrain, surface_of};
use crate::demo::{demo_map, synth_map};

/// Every overlay the board draws, laid over a handful of tiles, fog and the
/// never-seen included.
fn busy(map: &MapDocument) -> Overlays {
    let (w, h) = (map.ground.width() as i32, map.ground.height() as i32);
    let at = |col: i32, row: i32| (col.min(w - 1), row.min(h - 1));
    Overlays {
        selected: Some(at(3, 4)),
        reach: [at(5, 5), at(5, 6), at(6, 5)].into(),
        path: [at(5, 6)].into(),
        template: [at(8, 8)].into(),
        doors: [at(9, 9)].into(),
        encounters: [at(10, 10), at(9, 9)].into(),
        dim: [at(2, 2), at(3, 4), at(12, 12)].into(),
        hidden: [at(1, 1), at(0, 5)].into(),
        focus: None,
    }
}

/// The grown ground's bricks, cut at `ceiling` as the focus rebuild cut them:
/// voxels above it are air, and a brick left all air is gone.
fn grown(map: &MapDocument, overlays: &Overlays) -> BTreeMap<[i16; 3], Vec<u8>> {
    let ground = MapTerrain::with_overlays(map, Some(overlays)).grow();
    let ceiling = overlays.focus.map(surface_of);
    ground
        .keys()
        .filter_map(|key| {
            let (brick, origin) = ground.brick_materials(key)?;
            let mut bytes = brick.raw().to_vec();
            if let Some(ceiling) = ceiling {
                for (index, byte) in bytes.iter_mut().enumerate() {
                    if origin[1] + (index / 64) as i32 > ceiling {
                        *byte = 0;
                    }
                }
            }
            bytes.iter().any(|m| *m != 0).then_some((key, bytes))
        })
        .collect()
}

/// Every brick the columns say exists, made.
fn made(bricks: &BoardBricks<'_>) -> BTreeMap<[i16; 3], Vec<u8>> {
    let Some([low, high]) = bricks.bounds() else {
        return BTreeMap::new();
    };
    let mut made = BTreeMap::new();
    for z in low[2]..=high[2] {
        for x in low[0]..=high[0] {
            for y in bricks.layers([x, z]) {
                let mut bytes = vec![0; BRICK_BYTES];
                bricks.fill([x, y, z], &mut bytes);
                made.insert([x, y, z], bytes);
            }
        }
    }
    made
}

fn assert_parity(name: &str, map: &MapDocument, overlays: &Overlays) {
    let columns = TileColumns::of(map, overlays);
    let bricks = BoardBricks::new(&columns, overlays.focus.map(surface_of));
    let (made, grown) = (made(&bricks), grown(map, overlays));
    assert!(!grown.is_empty(), "{name}: the board grows ground");
    assert_eq!(
        made.keys().collect::<Vec<_>>(),
        grown.keys().collect::<Vec<_>>(),
        "{name}: the columns name exactly the bricks the ground holds"
    );
    for (key, bytes) in &grown {
        assert!(made[key] == *bytes, "{name}: brick {key:?} differs");
    }
}

#[test]
fn a_made_brick_is_the_grown_brick() {
    for (name, map) in [
        ("demo", demo_map()),
        ("synth 30", synth_map(30, 30)),
        ("relief 40", relief_map(40)),
        ("synth 17 by 9", synth_map(17, 9)),
    ] {
        assert_parity(name, &map, &Overlays::default());
        let overlays = busy(&map);
        assert_parity(&format!("{name} with overlays"), &map, &overlays);
        for focus in [0, 2] {
            let cut = Overlays {
                focus: Some(focus),
                ..overlays.clone()
            };
            assert_parity(&format!("{name} cut at {focus}"), &map, &cut);
        }
    }
}

/// Every brick whose bytes a change moved — made anew, emptied, or gone — is
/// one the change names.
#[test]
fn a_change_names_every_brick_it_moved() {
    let map = relief_map(40);
    let before = TileColumns::of(&map, &Overlays::default());
    let mut edited = map.clone();
    edited.elevation.set(3, 3, 7);
    edited.elevation.set(20, 11, 0);
    let water = edited.intern_tile_kind("water");
    edited.ground.set(31, 30, water);
    let overlays = Overlays {
        selected: Some((12, 12)),
        dim: [(25, 2)].into(),
        hidden: [(38, 38)].into(),
        ..Overlays::default()
    };
    let after = TileColumns::of(&edited, &overlays);
    let change = after.changed(&before).expect("one board, edited");
    let (old, new) = (
        made(&BoardBricks::new(&before, None)),
        made(&BoardBricks::new(&after, None)),
    );
    let keys: BTreeSet<_> = old.keys().chain(new.keys()).copied().collect();
    let moved: Vec<_> = keys
        .into_iter()
        .filter(|key| old.get(key) != new.get(key))
        .collect();
    assert!(!moved.is_empty());
    let named: BTreeSet<_> = change.bricks.iter().copied().collect();
    for key in &moved {
        assert!(named.contains(key), "brick {key:?} moved but was not named");
    }
    assert_eq!(
        change.tiles, 6,
        "the six tiles the edit and the overlays touched"
    );
    eprintln!(
        "a change of {} tiles names {} bricks; {} of them moved",
        change.tiles,
        change.bricks.len(),
        moved.len()
    );
    let unchanged = after.changed(&after).expect("the same board");
    assert_eq!((unchanged.tiles, unchanged.bricks.len()), (0, 0));
    assert!(
        TileColumns::of(&synth_map(30, 30), &Overlays::default())
            .changed(&before)
            .is_none(),
        "another board is not an edit of this one"
    );
}
