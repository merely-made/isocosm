// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Op, Shape, fixture::Grid};

#[test]
fn chart_places_each_connected_plane_site_once_and_reads_the_neighbours_edits() {
    let atlas = Grid::drawn(9, 4, 3, 64, false);
    let edits = [(
        1,
        Edit {
            op: Op::Fill(2),
            shape: Shape::Box {
                min: [4, 42, 4],
                max: [8, 47, 8],
            },
        },
    )];
    let root = Volume::charted(&atlas, 5, 8, &edits).unwrap();
    let chart = root.chart.unwrap();
    assert_eq!(chart.tiles.len(), 12);
    for (&pos, &tile) in &chart.tiles {
        let p = tile.local([0, 0]);
        assert_eq!(p, [0, 0]);
        assert_eq!(tile.site, (pos[1] + 1) as u64 * 4 + (pos[0] + 1) as u64);
    }
    let tile = chart.tile([0, -1]).unwrap();
    let v = chart.volume(tile.site).unwrap();
    let direct = Volume::paged(&atlas, 1, [0; 2], [64; 2], 8, &edits).unwrap();
    for z in 0..64 {
        for x in 0..64 {
            assert_eq!(v.surface(x, z), direct.surface(x, z));
        }
    }
    assert_eq!(v.material([5, 46, 5]), Some(2));
    assert!(chart.tile([-2, 0]).is_none());
}

#[test]
fn rotated_and_reflected_edges_preserve_every_cell_and_round_trip() {
    let a = Grid::drawn(9, 2, 1, 64, false);
    let mut chart = Chart::new(&a, 0, &[], 8).unwrap();
    let root = chart.tile([0, 0]).unwrap();
    for enters in 0..4 {
        for flipped in [false, true] {
            let out = Border {
                side: 1,
                enters,
                flipped,
            };
            chart.skeleton.get_mut(&0).unwrap().borders[1] = Some((1, out));
            chart.skeleton.get_mut(&1).unwrap().borders[enters as usize] = Some((
                0,
                Border {
                    side: enters,
                    enters: 1,
                    flipped,
                },
            ));
            let tile = root.adjacent(&chart, [1, 0]).unwrap();
            assert_eq!(tile.adjacent(&chart, [-1, 0]), Some(root));
            for x in [0, 1, 63] {
                for z in 0..64 {
                    let [nx, _, nz] = across_cell([64 + x, 0, z], out, 64);
                    assert_eq!(tile.local([x, z]), [nx, nz]);
                }
            }
        }
    }
}
