// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP5's checks over seeded draws, each beside a control that must fail.

use super::*;
use crate::edit::{Edit, Op, Shape};
use crate::fixture::Grid;
use crate::draw;

const SIDE: i64 = 32;
const BODY: Body = Body {
    width: 1,
    height: 2,
    climb: 1,
    wades: false,
};

fn window(grid: &Grid, edits: &[(SiteId, Edit)]) -> Volume {
    Volume::lift(grid, 0, [0, 0], [SIDE, SIDE], 12, edits).unwrap()
}

/// A flat site at height 20, its water ten below.
fn flat() -> Grid {
    let mut grid = Grid::drawn(1, 1, 1, 64, false);
    grid.skeleton = vec![[20, 0, 10]];
    grid
}

fn edit(op: Op, min: [i64; 3], max: [i64; 3]) -> Edit {
    Edit {
        op,
        shape: Shape::Box { min, max },
    }
}

fn place(places: &Places, at: [i64; 3]) -> PlaceId {
    places.place_at(at).unwrap()
}

#[test]
fn a_whole_derivation_is_the_same_twice_and_names_every_stance() {
    let grid = Grid::drawn(23, 1, 1, 64, false);
    let v = window(&grid, &[]);
    for cover in [Cover::Sealed, Cover::Roofed] {
        let a = Places::derive(&v, Rules::ruled(cover)).unwrap();
        assert_eq!(a, Places::derive(&v, Rules::ruled(cover)).unwrap());
        let c = cells::Cells::new(&v, a.rules);
        for at in c.everywhere() {
            if c.stance(at) {
                assert!(a.place_at(at).is_some(), "{at:?} under {cover:?}");
            }
        }
    }
}

/// Spheres carved and filled about the surface of a drawn site.
fn edits(seed: u64, grid: &Grid, n: u64) -> Vec<(SiteId, Edit)> {
    let rule = crate::edit::SiteRule::of(grid, 0).unwrap();
    (0..n)
        .map(|i| {
            let r = |k: u64, m: u64| (draw(seed, "place-edits", &[k, i]) % m) as i64;
            let (x, z) = (r(0, SIDE as u64), r(1, SIDE as u64));
            let y = rule.top(x, z) - 1 + r(2, 4);
            let op = if r(3, 3) == 0 { Op::Fill(3) } else { Op::Carve };
            let shape = Shape::Sphere {
                centre: [x, y, z],
                radius: 1 + r(4, 3) as u64,
            };
            (0, Edit { op, shape })
        })
        .collect()
}

#[test]
fn local_rederivation_equals_whole_after_every_edit() {
    for (seed, cover) in [(29, Cover::Sealed), (31, Cover::Roofed)] {
        let grid = Grid::drawn(seed, 1, 1, 64, false);
        let rules = Rules {
            cap: Cap::Own([8, 8]),
            neck: Some(2),
            ..Rules::ruled(cover)
        };
        let mut v = window(&grid, &[]);
        let mut local = Places::derive(&v, rules).unwrap();
        let mut stale = local.clone();
        let mut caught = 0;
        for (made, e) in edits(seed, &grid, 16) {
            v.apply(&grid, made, &e).unwrap();
            let dirty = v.drain_dirty();
            local.rederive(&v, &dirty).unwrap();
            let whole = Places::derive(&v, rules).unwrap();
            assert_eq!(local, whole, "{cover:?} after {e:?}");
            // Control: a stale dirty region misses what the edit moved.
            stale.rederive(&v, &[]).unwrap();
            caught += u32::from(stale != whole);
            stale = whole;
        }
        assert!(caught > 0, "no edit moved a place, so staleness went untested");
    }
}

#[test]
fn a_cliff_splits_patches_and_only_a_climber_crosses_it() {
    let grid = flat();
    let v = window(&grid, &[(0, edit(Op::Fill(3), [0, 21, 0], [16, 24, SIDE]))]);
    let places = Places::derive(&v, Rules::ruled(Cover::Sealed)).unwrap();
    let (high, low) = (place(&places, [4, 24, 4]), place(&places, [24, 21, 4]));
    assert_ne!(high, low, "a three-cell step within one patch");
    assert!(places.route(high, low, &BODY).is_none());
    let climber = Body { climb: 3, ..BODY };
    assert_eq!(places.route(high, low, &climber).unwrap().1, vec![high, low]);
    let p = places.passages.get(&[high.min(low), high.max(low)]).unwrap();
    assert!(p.clearances.iter().all(|c| c.step == 3));
}

#[test]
fn water_is_a_place_and_only_a_wader_enters_it() {
    let grid = flat();
    // A basin below the water line, filled with the world's water.
    let basin = [
        (0, edit(Op::Carve, [8, 15, 8], [24, 21, 24])),
        (0, edit(Op::Fill(1), [8, 15, 8], [24, 21, 24])),
    ];
    let v = window(&grid, &basin);
    let places = Places::derive(&v, Rules::ruled(Cover::Sealed)).unwrap();
    let lake = place(&places, [16, 18, 16]);
    assert_eq!(places.places[&lake].kind, Kind::Water);
    let shore = place(&places, [2, 21, 2]);
    assert!(places.route(shore, lake, &BODY).is_none());
    let wader = Body { wades: true, ..BODY };
    assert!(places.route(shore, lake, &wader).is_some());
}

#[test]
fn a_body_wider_or_taller_than_a_doorway_is_refused() {
    let grid = flat();
    let wall = [
        (0, edit(Op::Fill(3), [16, 21, 0], [17, 30, SIDE])),
        (0, edit(Op::Carve, [16, 21, 10], [17, 23, 11])),
    ];
    let v = window(&grid, &wall);
    // Clearance lives on passages between places, so the cap cuts at the
    // wall: a gap inside one patch is no passage (ruling 418).
    for cover in [Cover::Sealed, Cover::Roofed] {
        let rules = Rules {
            cap: Cap::Own([16, 32]),
            ..Rules::ruled(cover)
        };
        let places = Places::derive(&v, rules).unwrap();
        let (west, east) = (place(&places, [4, 21, 4]), place(&places, [28, 21, 4]));
        assert!(places.route(west, east, &BODY).is_some(), "{cover:?}");
        let wide = Body { width: 2, ..BODY };
        let tall = Body { height: 3, ..BODY };
        assert!(places.route(west, east, &wide).is_none(), "{cover:?}");
        assert!(places.route(west, east, &tall).is_none(), "{cover:?}");
        // Control: a filter that forgets width lets the wide body through.
        let broken = |p: &Passage| p.clearances.iter().any(|c| c.height >= wide.height && c.step <= wide.climb);
        assert!(places.route_by(west, east, broken).is_some(), "{cover:?}");
        let walk = places.walk([4, 21, 4], [28, 21, 4], &BODY).unwrap();
        assert!(walk.iter().any(|c| c[0] == 16 && c[2] == 10), "{walk:?}");
    }
}

#[test]
fn the_cap_cuts_a_wide_patch_on_its_grid() {
    let grid = flat();
    let v = window(&grid, &[]);
    let whole = Places::derive(&v, Rules::ruled(Cover::Sealed)).unwrap();
    let patches = |p: &Places| p.places.values().filter(|p| p.kind == Kind::Patch).count();
    assert_eq!(patches(&whole), 1);
    let small = Rules {
        cap: Cap::Own([16, 8]),
        ..Rules::ruled(Cover::Sealed)
    };
    let cut = Places::derive(&v, small).unwrap();
    assert_eq!(patches(&cut), 2 * 4);
    let one = cut.places.values().next().unwrap().component;
    assert!(cut.places.values().all(|p| p.component == one));
}

#[test]
fn a_cave_with_a_mouth_is_outdoors_when_sealed_and_a_room_when_roofed() {
    let grid = flat();
    let cave = [(0, edit(Op::Carve, [4, 17, 4], [12, 20, 12])), (0, edit(Op::Carve, [12, 17, 6], [14, 21, 8]))];
    let v = window(&grid, &cave);
    let sealed = Places::derive(&v, Rules::ruled(Cover::Sealed)).unwrap();
    let roofed = Places::derive(&v, Rules::ruled(Cover::Roofed)).unwrap();
    let inside = [8, 17, 8];
    assert_eq!(sealed.places[&place(&sealed, inside)].kind, Kind::Patch);
    assert_eq!(roofed.places[&place(&roofed, inside)].kind, Kind::Room);
}

#[test]
fn rooms_are_roofed_and_necks_two_by_default() {
    assert_eq!(Rules::default(), Rules::ruled(Cover::Roofed));
    assert_eq!(Rules::default().neck, Some(2));
}

#[test]
fn a_patch_splits_at_a_neck_narrower_than_the_world_rule() {
    let grid = flat();
    // A wall with an open gap one wide: no lintel, so no room.
    let wall = [
        (0, edit(Op::Fill(3), [16, 21, 0], [17, 30, SIDE])),
        (0, edit(Op::Carve, [16, 21, 10], [17, 30, 11])),
    ];
    let v = window(&grid, &wall);
    let wide = Body { width: 2, ..BODY };
    let (w, e) = ([4, 21, 4], [28, 21, 4]);
    let necked = Places::derive(&v, Rules { neck: Some(2), ..Rules::default() }).unwrap();
    let (west, east) = (place(&necked, w), place(&necked, e));
    assert_ne!(west, east);
    assert!(necked.route(west, east, &BODY).is_some());
    assert!(necked.route(west, east, &wide).is_none());
    // Control: without the rule the gap sits inside one patch and the wide
    // body passes.
    let open = Places::derive(&v, Rules { neck: None, ..Rules::default() }).unwrap();
    assert_eq!(place(&open, w), place(&open, e));
}

#[test]
fn sight_is_a_clear_ray_to_a_candidate_place() {
    let grid = flat();
    let wall = [(0, edit(Op::Fill(3), [16, 21, 0], [17, 30, SIDE]))];
    let v = window(&grid, &wall);
    let places = Places::derive(&v, Rules::default()).unwrap();
    let (eye, beyond, here) = ([4, 22, 4], [28, 22, 4], [12, 22, 4]);
    assert!(places.sees(&v, eye, here, 16));
    assert!(!places.sees(&v, eye, beyond, 32), "through a wall");
    let open = window(&grid, &[]);
    let field = Places::derive(&open, Rules::default()).unwrap();
    assert!(field.sees(&open, eye, beyond, 32));
    // Control: out of range, the ray is clear but no place is a candidate.
    assert!(ray(&open, eye, beyond));
    assert!(!field.sees(&open, eye, beyond, 8));
    // Candidates are the places near the eye, not every place.
    let cut = Rules { cap: Cap::Own([8, 8]), ..Rules::default() };
    let tiles = Places::derive(&open, cut).unwrap();
    let near = tiles.candidates(eye, 4);
    assert!(near.contains(&place(&tiles, eye)));
    assert!(!near.contains(&place(&tiles, beyond)));
    assert!(near.len() < tiles.places.len());
}
