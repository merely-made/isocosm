// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Drawn cliffs (ruling 744): the lower site steps down at least `CLIFF` at
//! each cliff span, the two sites still meet exactly everywhere else, and
//! every site's mean is still its elevation. Each beside a control.

use super::*;
use crate::fixture::Grid;
use crate::{Atlas, border::edge_cell, border::far_cell, edit::SiteRule};

#[test]
fn cliffs_step_the_lower_site_and_keep_borders_and_means_exact() {
    let mut cliffs = 0;
    for seed in 0..4 {
        let grid = Grid::drawn(seed, 3, 2, 64, true);
        assert!(check::borders(&grid, |g, s| g.lattice(s)).unwrap() > 0);
        assert!(check::means(&grid, &grid.sites(), |g, s| g.lattice(s)).is_ok());
        for site in grid.sites() {
            for side in 0..4u8 {
                let (to, b) = grid.border(site, side).unwrap();
                let (near, far) = (SiteRule::of(&grid, site).unwrap(), SiteRule::of(&grid, to).unwrap());
                for (c, drops) in cliff_spans(&grid, site, side).unwrap() {
                    let t = i64::from(c.span) * 4 + 2;
                    let [x, z] = edge_cell(side, t, 64);
                    let [fx, _, fz] = far_cell(side, t, 64, b, 0);
                    let (here, there) = (near.top(x, z), far.top(fx, fz));
                    assert!(here.abs_diff(there) >= CLIFF as u64, "seed {seed} site {site} side {side}");
                    assert_eq!(here < there, drops);
                    cliffs += 1;
                    // Control: the shared line itself parts at a cliff, so a
                    // check that does not set cliff spans aside fails.
                    let (ln, lf) = (grid.lattice(site).unwrap(), grid.lattice(to).unwrap());
                    let across = if b.flipped { t as u64 } else { 64 - t as u64 };
                    assert_ne!(ln.on_side(side, t as u64), lf.on_side(b.enters, across));
                }
            }
        }
    }
    assert!(cliffs > 0, "no cliff was drawn, so none was checked");
}

#[test]
fn both_sides_draw_the_same_cliffs_and_one_drops() {
    let grid = Grid::drawn(9, 3, 3, 64, true);
    for site in grid.sites() {
        for side in 0..4u8 {
            let (to, b) = grid.border(site, side).unwrap();
            let here: std::collections::BTreeSet<_> = cliff_spans(&grid, site, side)
                .unwrap()
                .into_iter()
                .map(|(c, d)| (SPANS - 1 - c.span, c.drop, d))
                .collect();
            let there: std::collections::BTreeSet<_> = cliff_spans(&grid, to, b.enters)
                .unwrap()
                .into_iter()
                .map(|(c, d)| (c.span, c.drop, !d))
                .collect();
            assert_eq!(here, there, "the same spans, mirrored, the other side dropping");
        }
    }
}
