// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    edit::{Op, Shape},
    fixture::Grid,
    places::{Places, Rules},
};

fn flat() -> Grid {
    let mut atlas = Grid::drawn(1, 1, 1, 64, false);
    atlas.skeleton[0] = [0, 0, -8];
    atlas
}

#[test]
fn an_authored_grounded_and_elevated_pose_fits_both_windows() {
    let a = flat();
    let dense = Volume::lift(&a, 0, [8, 8], [24, 24], 4, &[]).unwrap();
    let paged = Volume::paged(&a, 0, [8, 8], [24, 24], 4, &[]).unwrap();
    let top = dense.surface(12, 13).unwrap();
    let poses = [[12, top + 1, 13], [12, dense.ceiling() - 1, 13]];
    assert!(
        poses[1][1] > poses[0][1],
        "fixture has air above the grounded pose"
    );
    let patches = Places::derive(&dense, Rules::default()).unwrap();
    let patch = patches.place_at(poses[0]).unwrap();
    for volume in [&dense, &paged] {
        let places = Places::derive(volume, Rules::default()).unwrap();
        for at in poses {
            assert!(
                (volume.datum..volume.ceiling()).contains(&at[1]),
                "pose {at:?} outside {:?}",
                volume.datum..volume.ceiling()
            );
            assert_eq!(volume.material(at), Some(volume.materials.air));
            assert_eq!(places.place_at(at), Some(patch));
        }
        assert_eq!(volume.datum..volume.ceiling(), dense.datum..dense.ceiling());
    }
}

#[test]
fn procedural_subwindows_match_the_realized_frame_across_cliff_spans() {
    let mut a = Grid::drawn(17, 2, 2, 512, false);
    a.skeleton[0] = [19, 6, -8];
    a.skeleton[1] = [23, 240, 9];
    a.skeleton[2] = [70, 170, 18];
    assert!(
        !a.lattice(0).unwrap().cliffs.is_empty(),
        "fixture has actual cliff spans"
    );
    // Large enough to take the procedural path. The off-centre rectangle
    // cuts the lattice and cliff ramps both along and across their spans.
    for (min, max) in [([210, 1], [501, 267]), ([1, 239], [297, 504])] {
        let rule = SiteRule::of(&a, 0).unwrap();
        let brute = (min[1]..max[1])
            .flat_map(|z| (min[0]..max[0]).map(move |x| (x, z)))
            .map(|(x, z)| rule.top(x, z))
            .fold([i64::MAX, i64::MIN], |[lo, hi], y| [lo.min(y), hi.max(y)]);
        assert_eq!(rule.heights(min, max), brute);
        let dense = Volume::lift(&a, 0, min, max, 2, &[]).unwrap();
        let paged = Volume::paged(&a, 0, min, max, 2, &[]).unwrap();
        assert!(paged.columns.as_ref().unwrap().columns.is_none());
        assert_eq!(paged.datum..paged.ceiling(), dense.datum..dense.ceiling());
    }
}

#[test]
fn a_whole_large_site_keeps_only_its_procedural_columns() {
    let mut a = flat();
    a.side = 65_536;
    let paged = Volume::paged(&a, 0, [0, 0], [65_536, 65_536], 4, &[]).unwrap();
    let local = Volume::lift(&a, 0, [8, 8], [24, 24], 4, &[]).unwrap();
    assert!(paged.columns.as_ref().unwrap().columns.is_none());
    assert_eq!(paged.ground().brick_count(), 0);
    assert_eq!(paged.datum..paged.ceiling(), local.datum..local.ceiling());
    assert_eq!(paged.surface(65_535, 65_535), local.surface(12, 13));
}

#[test]
fn replayed_edits_do_not_shift_the_frame_or_grant_deeper_bedrock() {
    let a = flat();
    let build = |edits: &[(u64, Edit)]| {
        (
            Volume::lift(&a, 0, [8, 8], [24, 24], 4, edits),
            Volume::paged(&a, 0, [8, 8], [24, 24], 4, edits),
        )
    };
    let box_edit = |op, lo, hi| {
        (
            0,
            Edit {
                op,
                shape: Shape::Box {
                    min: [10, lo, 10],
                    max: [11, hi, 11],
                },
            },
        )
    };
    let base = build(&[]).0.unwrap();
    let high = base.ceiling() + i64::from(BRICK) * 2;
    for edits in [
        vec![box_edit(Op::Fill(3), high, high + 1)],
        vec![box_edit(Op::Carve, high, high + 1)],
        vec![
            box_edit(Op::Fill(3), high, high + 1),
            box_edit(Op::Carve, high, high + 1),
        ],
    ] {
        let (dense, paged) = build(&edits);
        let (dense, paged) = (dense.unwrap(), paged.unwrap());
        assert_eq!(
            paged.datum..paged.ceiling(),
            dense.datum..dense.ceiling(),
            "{edits:?}"
        );
    }
    let (dense, paged) = build(&[box_edit(Op::Carve, base.datum, high)]);
    assert_eq!(
        paged.unwrap().surface(10, 10),
        dense.unwrap().surface(10, 10)
    );
    let (dense, paged) = build(&[box_edit(Op::Carve, base.datum - 1, base.datum)]);
    assert!(dense.is_err() && paged.is_err());
}
