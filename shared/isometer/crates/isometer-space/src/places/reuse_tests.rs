// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Compare the complete graph against the same terrain without step reuse.
use super::*;
use crate::{
    edit::{Edit, Op, Shape},
    fixture::Grid,
};
use std::time::Instant;

fn derive(volume: &Volume, rules: Rules, reuse: bool) -> (Places, usize, u128) {
    let started = Instant::now();
    let mut cells = Cells::new(volume, rules);
    cells.reuse_steps = reuse;
    let mut graph = Places {
        site: volume.site,
        rules,
        places: BTreeMap::new(),
        passages: BTreeMap::new(),
        labels: BTreeMap::new(),
    };
    let found = flood::components(&cells, cells.everywhere());
    graph.insert(&cells, found);
    (graph, cells.step_reads.get(), started.elapsed().as_micros())
}

#[test]
fn cached_steps_preserve_rooms_water_necks_and_passages_under_both_cover_rules() {
    let mut grid = Grid::drawn(17, 1, 1, 64, false);
    for water in [10, 23] {
        grid.skeleton = vec![[20, 0, water]];
        let edits = vec![
            (
                0,
                Edit {
                    op: Op::Carve,
                    shape: Shape::Box {
                        min: [8, 17, 8],
                        max: [24, 20, 24],
                    },
                },
            ),
            (
                0,
                Edit {
                    op: Op::Fill(3),
                    shape: Shape::Box {
                        min: [8, 22, 8],
                        max: [24, 24, 24],
                    },
                },
            ),
            (
                0,
                Edit {
                    op: Op::Fill(3),
                    shape: Shape::Sphere {
                        centre: [24, 22, 24],
                        radius: 3,
                    },
                },
            ),
        ];
        let volume = Volume::paged(&grid, 0, [0; 2], [32; 2], 8, &edits).unwrap();
        for cover in [Cover::Roofed, Cover::Sealed] {
            let rules = Rules {
                cap: Cap::Own([8; 2]),
                ..Rules::ruled(cover)
            };
            let (reference, reads, _) = derive(&volume, rules, false);
            let (cached, reused_reads, _) = derive(&volume, rules, true);
            assert_eq!(cached, reference, "water {water}, {cover:?}");
            assert_eq!(cached, Places::derive(&volume, rules).unwrap());
            assert!(
                reused_reads < reads,
                "the control must exercise repeated steps"
            );
            assert!(cached.places.values().any(|p| p.kind == Kind::Room));
            if water > 20 {
                assert!(cached.places.values().any(|p| p.kind == Kind::Water));
            }
        }
    }
}

#[test]
fn cold_place_derivation_reuses_stances_instead_of_rescanning_each_pass() {
    let mut grid = Grid::drawn(29, 1, 1, 256, false);
    grid.skeleton = vec![[32, 8, 0]];
    let volume = Volume::paged(&grid, 0, [32; 2], [160; 2], 8, &[]).unwrap();
    let (reference, reads, reference_us) = derive(&volume, Rules::default(), false);
    let (cached, reused_reads, cached_us) = derive(&volume, Rules::default(), true);
    assert_eq!(cached, reference);
    assert!(
        reused_reads * 2 < reads,
        "must avoid at least half the terrain step reads"
    );
    eprintln!(
        "[place-reuse] columns=16384 reference_reads={reads} reused_reads={reused_reads} reference_us={reference_us} cached_us={cached_us} exact=true"
    );
}
