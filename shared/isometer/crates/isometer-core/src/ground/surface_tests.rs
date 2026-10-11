// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

struct Tall;

impl Terrain for Tall {
    fn sea_level(&self, _extent: i32) -> i32 {
        1
    }

    fn surface(&self, _extent: i32, _x: i32, _z: i32) -> i32 {
        73
    }
}

#[test]
fn surface_reads_resident_height_above_the_generation_band() {
    let ground = Ground::grow(&Tall, 1);
    for [x, z] in [[0, 0], [-1, -1], [1, 1]] {
        assert_eq!(ground.surface(x, z), Some(73));
        assert!(ground.stands([x, 74, z], 2));
    }
    assert_eq!(ground.surface(2, 0), None);
}

#[test]
fn surface_tracks_edited_and_deleted_upper_voxels_in_the_same_column() {
    let mut ground = Ground::default();
    ground
        .write([
            ([-9, 0, -17], ROCK),
            ([-9, 31, -17], ROCK),
            ([-9, 73, -17], ROCK),
            ([-9, 96, -16], ROCK),
            ([-8, 101, -17], ROCK),
        ])
        .unwrap();
    assert_eq!(ground.surface(-9, -17), Some(73));
    ground.write([([-9, 73, -17], AIR)]).unwrap();
    assert_eq!(ground.surface(-9, -17), Some(31));
    ground.carve([-9, 31, -17], 0);
    assert_eq!(ground.surface(-9, -17), Some(0));
    ground.write([([-9, 0, -17], AIR)]).unwrap();
    assert_eq!(ground.surface(-9, -17), None);
    assert_eq!(ground.surface(-9, -16), Some(96));
    assert_eq!(ground.surface(-8, -17), Some(101));
    ground.write([([-9, 144, -17], SOIL)]).unwrap();
    assert_eq!(ground.surface(-9, -17), Some(144));
}

#[test]
fn surface_queries_preserve_revision_bytes_and_projection_dirt() {
    let mut ground = Ground::default();
    ground.write([([-1, 80, -1], ROCK)]).unwrap();
    let revision = ground.revision();
    let dirty = ground.dirty.clone();
    let bytes = crate::snapshot::encode(&ground).unwrap();
    for _ in 0..3 {
        assert_eq!(ground.surface(-1, -1), Some(80));
        assert_eq!(ground.surface(-1, 0), None);
    }
    assert_eq!(ground.revision(), revision);
    assert_eq!(ground.dirty, dirty);
    assert_eq!(crate::snapshot::encode(&ground).unwrap(), bytes);
}

#[test]
fn surface_keeps_absent_columns_distinct_from_virtual_bedrock() {
    let mut ground = Ground::default();
    assert!(ground.solid([-9, -1, -17]));
    assert_eq!(ground.surface(-9, -17), None);
    ground.write([([-9, 0, -17], ROCK)]).unwrap();
    assert_eq!(ground.surface(-9, -17), Some(0));
    assert_eq!(ground.surface(-9, -18), None);
    assert_eq!(ground.surface(i32::MIN, -17), None);
    ground.write([([-9, 0, -17], AIR)]).unwrap();
    assert_eq!(ground.surface(-9, -17), None);
    assert!(ground.solid([-9, -1, -17]));
}
