// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! SP5's border clause: a crossing the profile says passes reads passable
//! in the joined volumes, one it says climbs passes only a climber, and one
//! it says stops passes nobody dry. Each beside a control that must fail.

use super::*;
use crate::border::{Span, across_cell, cells, cells_with, edge_cell, spans};
use crate::fixture::Grid;
use crate::{Atlas, SPANS};

const SIDE: i64 = 64;
/// A climb of nothing, so every step shows as a climb.
const FLAT: Climb = Climb { rise: 0, run: 1 };

fn rules() -> Rules {
    Rules {
        climb: FLAT,
        ..Rules::default()
    }
}

fn body(climb: u32, wades: bool) -> Body {
    Body {
        width: 1,
        height: 1,
        climb,
        wades,
    }
}

/// What the joined volumes say of one cell of site 0's east side.
fn read(grid: &Grid, t: i64) -> Span {
    let (_, b) = grid.border(0, 1).unwrap();
    let [x, z] = edge_cell(1, t, SIDE);
    let [fx, _, fz] = across_cell([x + 1, 0, z], b, SIDE);
    let near = Volume::lift(grid, 0, [x, z], [x + 1, z + 1], 4, &[]).unwrap();
    let far = Volume::lift(grid, 1, [fx, fz], [fx + 1, fz + 1], 4, &[]).unwrap();
    let (pn, pf) = (Places::derive(&near, rules()).unwrap(), Places::derive(&far, rules()).unwrap());
    let joined = join(grid, (&near, &pn), (&far, &pf)).unwrap();
    let dry: Vec<&Passage> = joined.iter().filter(|p| !p.wet).collect();
    if dry.iter().any(|p| body(0, false).fits(p).is_some()) {
        return Span::Passes;
    }
    let step = dry.iter().flat_map(|p| &p.clearances).map(|c| u64::from(c.step)).min();
    step.map_or(Span::Stops, |step| Span::Climbs { step })
}

#[test]
fn every_border_cell_reads_in_the_volume_as_the_lift_classes_it() {
    let (mut seen, mut blind) = ([0u32; 3], 0);
    for seed in 0..4 {
        let grid = Grid::drawn(seed, 2, 1, SIDE as u64, false);
        let lifted = cells(&grid, 0, 1, FLAT).unwrap();
        let unwet = cells_with(&grid, 0, 1, FLAT, false).unwrap();
        for t in 0..SIDE {
            let (class, volume) = (lifted[t as usize], read(&grid, t));
            assert_eq!(class, volume, "seed {seed}, cell {t}");
            seen[match class {
                Span::Passes => 0,
                Span::Climbs { .. } => 1,
                Span::Stops => 2,
            }] += 1;
            // Control: a reading blind to water disagrees where it lies.
            blind += u32::from(unwet[t as usize] != volume);
        }
    }
    assert!(seen.iter().all(|&n| n > 0), "every class must be drawn: {seen:?}");
    assert!(blind > 0, "no water on the border, so the control never ran");
}

#[test]
fn a_drawn_cliff_stops_a_dry_route() {
    let mut cliffs = 0;
    for seed in 0..4 {
        let mut grid = Grid::drawn(seed, 2, 1, SIDE as u64, false);
        for s in &mut grid.skeleton {
            s[2] -= 200;
        }
        let lifted = cells(&grid, 0, 1, FLAT).unwrap();
        for t in 0..SIDE {
            let volume = read(&grid, t);
            assert_eq!(lifted[t as usize], volume, "seed {seed}, cell {t}");
            cliffs += u32::from(volume == Span::Stops);
        }
    }
    assert!(cliffs > 0, "no dry cliff was drawn");
}

#[test]
fn both_sides_class_a_border_alike_and_spans_take_the_easiest_cell() {
    let grid = Grid::drawn(5, 2, 1, SIDE as u64, false);
    let here = cells(&grid, 0, 1, FLAT).unwrap();
    let mut there = cells(&grid, 1, 3, FLAT).unwrap();
    there.reverse();
    assert_eq!(here, there);
    let spans = spans(&grid, 0, 1, FLAT).unwrap();
    assert_eq!(spans.len(), SPANS as usize);
    let per = SIDE as usize / SPANS as usize;
    for (k, span) in spans.iter().enumerate() {
        let part = &here[k * per..(k + 1) * per];
        assert_eq!(*span == Span::Passes, part.contains(&Span::Passes));
    }
}

#[test]
fn a_route_runs_from_one_site_into_the_next() {
    let grid = Grid::drawn(7, 2, 1, SIDE as u64, false);
    let (near, far) = (
        Volume::lift(&grid, 0, [SIDE - 16, 0], [SIDE, 16], 8, &[]).unwrap(),
        Volume::lift(&grid, 1, [0, 0], [16, 16], 8, &[]).unwrap(),
    );
    let (pn, pf) = (Places::derive(&near, Rules::default()).unwrap(), Places::derive(&far, Rules::default()).unwrap());
    let joined = join(&grid, (&near, &pn), (&far, &pf)).unwrap();
    assert!(joined.iter().all(|p| p.between[0].site != p.between[1].site));
    let from = *pn.places.keys().next().unwrap();
    let to = *pf.places.keys().next().unwrap();
    let all = || pn.passages.values().chain(pf.passages.values()).chain(&joined);
    let walker = body(64, true);
    assert!(route_over(all(), from, to, |p| walker.fits(p).is_some()).is_some());
    // Control: without the join the sites are islands.
    let apart = pn.passages.values().chain(pf.passages.values());
    assert!(route_over(apart, from, to, |p| walker.fits(p).is_some()).is_none());
}
