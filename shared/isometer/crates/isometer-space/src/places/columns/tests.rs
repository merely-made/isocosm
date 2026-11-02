// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! PG7's checks: places from the columns equal the cell-by-cell derivation
//! over the realized volume, on seeded edit sequences, under both covers,
//! with necks and a cutting cap; beside a control that seeds rooms only at
//! carved cells and misses a room that solid exceptions wall in.

use crate::draw;
use crate::edit::{Edit, Op, Shape, SiteRule};
use crate::fixture::Grid;
use crate::places::{Cap, Cover, Kind, Places, Rules, cells::Cells, flood};
use crate::volume::Volume;
use crate::SiteId;
use std::collections::BTreeMap;

const SIDE: i64 = 32;

/// Spheres and boxes carved and filled about the surface, some of them
/// shells that wall rule air in.
fn edits(seed: u64, grid: &Grid, n: u64) -> Vec<(SiteId, Edit)> {
    let rule = SiteRule::of(grid, 0).unwrap();
    (0..n)
        .map(|i| {
            let r = |k: u64, m: u64| (draw(seed, "pg7-edits", &[k, i]) % m) as i64;
            let (x, z) = (r(0, SIDE as u64), r(1, SIDE as u64));
            let y = rule.top(x, z) - 2 + r(2, 6);
            let op = if r(3, 3) == 0 { Op::Fill(3) } else { Op::Carve };
            let shape = match r(5, 3) {
                0 => Shape::Box { min: [x, y, z], max: [x + 1 + r(4, 6), y + 1 + r(6, 4), z + 1 + r(7, 6)] },
                _ => Shape::Sphere { centre: [x, y, z], radius: 1 + r(4, 3) as u64 },
            };
            (0, Edit { op, shape })
        })
        .collect()
}

fn rules(cover: Cover, cap: Cap, neck: Option<u32>) -> Rules {
    Rules { cap, neck, ..Rules::ruled(cover) }
}

#[test]
fn columns_equal_cells_over_seeded_edits_paged_and_realized() {
    let mut rooms = 0;
    for seed in [3, 5, 8, 13, 21, 34] {
        let grid = Grid::drawn(seed, 2, 2, 64, false);
        let all = edits(seed, &grid, 24);
        for n in [0, 6, 24] {
            let made = &all[..n];
            let realized = Volume::lift(&grid, 0, [0, 0], [SIDE, SIDE], 10, made).unwrap();
            let paged = Volume::paged(&grid, 0, [0, 0], [SIDE, SIDE], 10, made).unwrap();
            for cover in [Cover::Roofed, Cover::Sealed] {
                for (cap, neck) in [(Cap::Large, Some(2)), (Cap::Own([8, 8]), Some(3)), (Cap::Own([5, 7]), None)] {
                    let rules = rules(cover, cap, neck);
                    let reference = Places::by_cells(&realized, rules).unwrap();
                    assert_eq!(Places::derive(&realized, rules).unwrap(), reference, "realized {seed} {n} {rules:?}");
                    let ours = Places::derive(&paged, rules).unwrap();
                    assert_eq!(ours.places, reference.places, "paged places {seed} {n} {rules:?}");
                    assert_eq!(ours.passages, reference.passages, "paged passages {seed} {n} {rules:?}");
                    rooms += reference.places.values().filter(|p| p.kind == Kind::Room).count();
                }
            }
        }
    }
    assert!(rooms > 0, "no room was drawn, so rooms went untested");
}

#[test]
fn a_window_without_edits_derives_without_realizing_a_volume() {
    let grid = Grid::drawn(9, 2, 2, 64, false);
    let paged = Volume::paged(&grid, 0, [0, 0], [SIDE, SIDE], 10, &[]).unwrap();
    assert_eq!(paged.ground().brick_count(), 0);
    let realized = Volume::lift(&grid, 0, [0, 0], [SIDE, SIDE], 10, &[]).unwrap();
    for cover in [Cover::Roofed, Cover::Sealed] {
        let rules = Rules::ruled(cover);
        let ours = Places::derive(&paged, rules).unwrap();
        assert_eq!(ours.places, Places::by_cells(&realized, rules).unwrap().places);
        assert!(ours.places.values().all(|p| p.kind != Kind::Room), "rule air under the sky is no room");
    }
}

#[test]
fn a_room_walled_in_by_fills_is_found_and_a_control_without_seeds_beside_them_misses_it() {
    let mut grid = Grid::drawn(1, 1, 1, 64, false);
    grid.skeleton = vec![[20, 0, 10]];
    let fill = |min: [i64; 3], max: [i64; 3]| (0, Edit { op: Op::Fill(3), shape: Shape::Box { min, max } });
    // A shell on the ground: four walls and a roof over rule air.
    let shell = [
        fill([4, 21, 4], [12, 26, 5]),
        fill([4, 21, 11], [12, 26, 12]),
        fill([4, 21, 4], [5, 26, 12]),
        fill([11, 21, 4], [12, 26, 12]),
        fill([4, 26, 4], [12, 27, 12]),
    ];
    let v = Volume::lift(&grid, 0, [0, 0], [16, 16], 10, &shell).unwrap();
    for cover in [Cover::Roofed, Cover::Sealed] {
        let rules = Rules::ruled(cover);
        let ours = Places::derive(&v, rules).unwrap();
        assert_eq!(ours, Places::by_cells(&v, rules).unwrap());
        let inside = ours.place_at([8, 22, 8]).unwrap();
        assert_eq!(ours.places[&inside].kind, Kind::Room, "{cover:?}");
        // Control: the seeds beside solid exceptions dropped, as if only a
        // carve could make a room.
        let cells = Cells::new(&v, rules);
        let walled = |a: [i64; 3]| shell.iter().any(|(_, e)| {
            let [lo, hi] = e.shape.extent();
            (lo[0]..hi[0]).contains(&a[0]) && (lo[2]..hi[2]).contains(&a[2])
        });
        let seeds = cells.everywhere().into_iter().filter(|&a| !walled(a)).collect();
        let mut control = Places { site: v.site, rules, places: BTreeMap::new(), passages: BTreeMap::new(), labels: BTreeMap::new() };
        control.insert(&cells, flood::components(&cells, seeds));
        assert!(control.places.values().all(|p| p.kind != Kind::Room), "the control found the room");
        assert_ne!(control, ours);
    }
}
