// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP4's checks over seeded draws, each beside a control that must fail.

use super::*;
use crate::fixture::Grid;
use crate::volume::Volume;
use crate::{Atlas, SiteId, draw};

const SIDE: u64 = 64;

/// Edits drawn near the surface of the grid's sites, some across borders.
fn edits(seed: u64, grid: &Grid, n: u64) -> Vec<(SiteId, Edit)> {
    let r = |k: u64, i: u64, m: u64| draw(seed, "edits", &[k, i]) % m;
    (0..n)
        .map(|i| {
            let site = r(0, i, grid.width * grid.height);
            let (x, z) = (r(1, i, SIDE + 8) as i64 - 4, r(2, i, SIDE + 8) as i64 - 4);
            let (xc, zc) = (x.clamp(0, SIDE as i64 - 1), z.clamp(0, SIDE as i64 - 1));
            let top = crate::edit::SiteRule::of(grid, site).unwrap().top(xc, zc);
            let y = top - 2 + r(3, i, 5) as i64;
            let radius = 1 + r(4, i, 5);
            let shape = match r(5, i, 3) {
                0 => Shape::Sphere {
                    centre: [x, y, z],
                    radius,
                },
                1 => Shape::Box {
                    min: [x, y - 2, z],
                    max: [x + 1 + radius as i64, y + 1, z + 3],
                },
                _ => Shape::Route {
                    points: vec![[x, y, z], [x + 5, y - 1, z + 3], [x + 2, y - 3, z + 7]],
                    radius: radius.min(3),
                },
            };
            let op = if r(6, i, 3) == 0 { Op::Fill(2 + r(7, i, 2) as u8) } else { Op::Carve };
            (site, Edit { op, shape })
        })
        .collect()
}

fn inside(shape: &Shape, [x, y, z]: [i64; 3]) -> bool {
    let d2 = |p: [i64; 3], c: [i64; 3]| -> i128 {
        (0..3).map(|k| i128::from(2 * p[k] + 1 - 2 * c[k]).pow(2)).sum()
    };
    match shape {
        Shape::Sphere { centre, radius } => d2([x, y, z], *centre) <= i128::from(2 * radius).pow(2),
        Shape::Box { min, max } => (0..3).all(|k| min[k] <= [x, y, z][k] && [x, y, z][k] < max[k]),
        Shape::Route { points, radius } => {
            let p = [2 * x + 1, 2 * y + 1, 2 * z + 1].map(i128::from);
            let lim = i128::from(2 * radius).pow(2);
            points.windows(2).any(|w| {
                let (a, b) = (w[0].map(|v| 2 * i128::from(v)), w[1].map(|v| 2 * i128::from(v)));
                let d = [0, 1, 2].map(|k| b[k] - a[k]);
                let den: i128 = d.iter().map(|v| v * v).sum();
                let w = [0, 1, 2].map(|k| p[k] - a[k]);
                let tn: i128 = (0..3).map(|k| w[k] * d[k]).sum();
                let ww: i128 = w.iter().map(|v| v * v).sum();
                let v = [0, 1, 2].map(|k| p[k] - b[k]);
                let vv: i128 = v.iter().map(|v| v * v).sum();
                if tn <= 0 {
                    ww <= lim
                } else if tn >= den {
                    vv <= lim
                } else {
                    ww * den - tn * tn <= lim * den
                }
            })
        },
    }
}

#[test]
fn runs_hold_exactly_the_cells_whose_centres_lie_inside() {
    let grid = Grid::drawn(3, 2, 2, SIDE, false);
    for (_, edit) in edits(11, &grid, 60) {
        let [lo, hi] = edit.shape.extent();
        for x in lo[0] - 1..hi[0] + 1 {
            for z in lo[2] - 1..hi[2] + 1 {
                let runs = runs(&edit.shape, x, z);
                for y in lo[1] - 2..hi[1] + 2 {
                    let ran = runs.iter().any(|r| (r[0]..r[1]).contains(&y));
                    assert_eq!(ran, inside(&edit.shape, [x, y, z]), "{edit:?} at {x},{y},{z}");
                }
            }
        }
    }
}

#[test]
fn seeded_edits_replay_to_identical_bytes() {
    let grid = Grid::drawn(5, 3, 2, SIDE, false);
    let edits = edits(21, &grid, 40);
    for site in grid.sites() {
        for level in [0u8, 1, 3] {
            let a = grid.lift_edited(site, level, [0, 0], &edits).unwrap();
            let b = grid.lift_edited(site, level, [0, 0], &edits).unwrap();
            assert_eq!(postcard::to_allocvec(&a).unwrap(), postcard::to_allocvec(&b).unwrap());
        }
    }
    let unedited = grid.lift(0, 0, [0, 0]).unwrap();
    assert!(unedited.exceptions.is_empty());
    let edited = grid.lift_edited(0, 0, [0, 0], &edits).unwrap();
    assert_ne!(edited, unedited, "the draw left site 0 untouched");
}

#[test]
fn a_site_relifted_with_its_edits_equals_the_edited_site() {
    let grid = Grid::drawn(7, 2, 2, SIDE, false);
    let edits = edits(31, &grid, 30);
    let full = ([0, 0], [SIDE as i64, SIDE as i64]);
    let mut controlled = 0;
    for site in grid.sites() {
        let relifted = Volume::lift(&grid, site, full.0, full.1, 16, &edits).unwrap();
        let mut edited = Volume::lift(&grid, site, full.0, full.1, 16, &[]).unwrap();
        for (made, edit) in &edits {
            edited.apply(&grid, *made, edit).unwrap();
        }
        assert!(relifted.same_cells(&edited), "site {site}");
        let chunk = grid.lift_edited(site, 0, [0, 0], &edits).unwrap();
        for e in &chunk.exceptions {
            let at = [i64::from(e.column[0]), e.y, i64::from(e.column[1])];
            assert_eq!(relifted.material(at), Some(e.material));
        }
        // Control: the site lifted as if nothing had been asserted.
        if !chunk.exceptions.is_empty() {
            let bare = Volume::lift(&grid, site, full.0, full.1, 16, &[]).unwrap();
            assert!(!bare.same_cells(&edited), "site {site}");
            controlled += 1;
        }
    }
    assert!(controlled > 0, "no site was edited, so the control never ran");
}

#[test]
fn an_edit_across_a_border_is_seen_from_both_sides() {
    let grid = Grid::drawn(9, 2, 1, SIDE, false);
    let carve = Edit {
        op: Op::Carve,
        shape: Shape::Sphere {
            centre: [SIDE as i64 - 1, grid.elevation(0).unwrap(), 16],
            radius: 5,
        },
    };
    let edits = vec![(0, carve.clone())];
    let far = grid.lift_edited(1, 0, [0, 0], &edits).unwrap();
    let ignoring = grid.lift_edited(1, 0, [0, 0], &[]).unwrap();
    let mut seen = 0;
    for x in 0..6i64 {
        for z in 0..i64::from(crate::CHUNK) {
            for y in grid.elevation(0).unwrap() - 6..grid.elevation(0).unwrap() + 6 {
                if inside(&carve.shape, [SIDE as i64 + x, y, z]) {
                    assert_eq!(far.material(x as u32, z as u32, y), 0);
                    seen += u32::from(ignoring.material(x as u32, z as u32, y) != 0);
                }
            }
        }
    }
    assert!(seen > 0, "a neighbour ignoring foreign edits must disagree");
}

#[test]
fn every_edit_conserves_matter_cell_for_cell() {
    let grid = Grid::drawn(13, 2, 2, SIDE, true);
    let edits = edits(41, &grid, 25);
    let full = ([0, 0], [SIDE as i64, SIDE as i64]);
    let census = |prior: &[(SiteId, Edit)]| {
        let mut n = Moved::new();
        for site in grid.sites() {
            let v = Volume::lift(&grid, site, full.0, full.1, 16, prior).unwrap();
            for x in 0..SIDE as i64 {
                for z in 0..SIDE as i64 {
                    for y in v.datum..v.ceiling() {
                        *n.entry(v.material([x, y, z]).unwrap()).or_default() += 1;
                    }
                }
            }
        }
        n.remove(&0);
        n
    };
    let mut before = census(&[]);
    let mut uncredited = 0;
    for i in 0..edits.len() {
        let (site, edit) = &edits[i];
        let moved = tally(&grid, *site, &edits[..i], edit).unwrap();
        let after = census(&edits[..=i]);
        for m in 1..4u8 {
            let lost = before.get(&m).copied().unwrap_or(0) - after.get(&m).copied().unwrap_or(0);
            assert_eq!(moved.get(&m).copied().unwrap_or(0), lost, "edit {i}, material {m}");
            // Control: a tally crediting nobody reads zero here.
            uncredited += u32::from(lost != 0);
        }
        before = after;
    }
    assert!(uncredited > 0, "no edit moved matter, so a silent tally would pass");
}

#[test]
fn stored_bytes_grow_with_edits_and_not_with_sites() {
    let small = Grid::drawn(17, 2, 2, SIDE, false);
    let large = Grid::drawn(17, 8, 8, SIDE, false);
    let bytes = |e: &[(SiteId, Edit)]| postcard::to_allocvec(e).unwrap().len();
    let few = edits(51, &small, 4);
    let many = edits(51, &small, 40);
    assert!(bytes(&many) > bytes(&few));
    // Lifting a larger world stores nothing: the facts are the edits alone.
    let lifted = |g: &Grid| g.lift_edited(0, 0, [0, 0], &few).unwrap();
    assert_eq!(lifted(&small).site, lifted(&large).site);
}

#[test]
fn a_coarse_lift_samples_each_edited_cell_at_its_origin() {
    let grid = Grid::drawn(19, 2, 2, SIDE, false);
    let edits = edits(61, &grid, 30);
    let fine = Volume::lift(&grid, 0, [0, 0], [SIDE as i64, SIDE as i64], 16, &edits).unwrap();
    let reach = reaching(&grid, 0, &edits);
    for level in 1..=2u8 {
        let step = 1i64 << level;
        let coarse = grid.lift_edited(0, level, [0, 0], &edits).unwrap();
        let cells = touched(&reach, coarse.materials, [0, 0], [SIDE as i64; 2], step);
        for (([x, z], y), m) in cells {
            let (cx, cz) = ((x / step) as u32, (z / step) as u32);
            assert_eq!(coarse.material(cx, cz, y / step), m);
            assert_eq!(fine.material([x, y, z]), Some(m));
        }
    }
}

#[test]
fn edits_spanning_a_side_or_turned_inside_out_are_refused() {
    let footprint = crate::Footprint { sides: 4, side: SIDE };
    let wide = Edit {
        op: Op::Carve,
        shape: Shape::Sphere {
            centre: [0, 0, 0],
            radius: SIDE / 2,
        },
    };
    assert!(wide.check(footprint).is_err());
    let flipped = Edit {
        op: Op::Carve,
        shape: Shape::Box {
            min: [4, 0, 0],
            max: [0, 1, 1],
        },
    };
    assert!(flipped.check(footprint).is_err());
    let empty = Edit {
        op: Op::Carve,
        shape: Shape::Route {
            points: vec![],
            radius: 1,
        },
    };
    assert!(empty.check(footprint).is_err());
}
