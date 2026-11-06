// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;
use std::ops::Range;

use isometer_lens::{AtlasLimits, BrickProjectionRevision};

use super::*;
use crate::SlabCamera;
use crate::scene::{
    BRICK_BYTES, BrickSource, HostTerrain, PagedTerrain, Residency, ResidencySettings,
    TerrainRefresh, TerrainSource, framed_bricks,
};

const HOME: [i16; 3] = [0, 0, 0];
const SOLID: [u8; BRICK_BYTES] = [3; BRICK_BYTES];
const EDITED: [u8; BRICK_BYTES] = [7; BRICK_BYTES];

struct Strip(u8);

impl BrickSource for Strip {
    fn bounds(&self) -> Option<[[i16; 3]; 2]> {
        Some([HOME, [3, 0, 0]])
    }

    fn layers(&self, _column: [i16; 2]) -> Range<i16> {
        0..1
    }

    fn fill(&self, _key: [i16; 3], out: &mut [u8]) {
        out.fill(self.0);
    }
}

fn view(x: f32) -> SlabCamera {
    SlabCamera {
        centre: [x, 4.0, 4.0],
        forward: [0.0, 0.0, -1.0],
        half_height: 2.0,
        aspect: 1.0,
        depth: 20.0,
        cutaway: None,
        projection: Default::default(),
    }
}

fn capture(cache: &TerrainQueryCache, map: &BrickMap, expected_copy: bool) -> Arc<BrickMap> {
    let (map, copied) = cache.capture(Some(map));
    assert_eq!(copied, expected_copy);
    map.unwrap()
}

fn sample(map: &BrickMap, x: f32) -> Option<u8> {
    map.trace_ray([x, 4.0, 12.0], [0.0, 0.0, -1.0], 20.0)
        .unwrap()
        .map(|hit| hit.material)
}

#[test]
fn current_and_byte_identical_dirty_refreshes_share_one_terrain_query_version() {
    let source = Strip(3);
    let framed = framed_bricks(view(4.0), 0.0, &source, 1);
    assert_eq!(framed.keys, [HOME]);
    let residency = RefCell::new(Residency::new(
        ResidencySettings::default(),
        AtlasLimits::DEFAULT,
    ));
    let terrain = PagedTerrain::new(&residency, &source, &framed, 1);
    let cache = TerrainQueryCache::default();
    let mut map = terrain.brick_map().unwrap();
    let frozen = capture(&cache, &map, true);

    let mut overflow = framed.clone();
    overflow.overflow += 1;
    let overflow = PagedTerrain::new(&residency, &source, &overflow, 1);
    assert_eq!(
        overflow.refresh(&mut map, &[]).unwrap(),
        TerrainRefresh::Current
    );
    assert_eq!(residency.borrow().overflow(), framed.overflow + 1);
    assert!(Arc::ptr_eq(&frozen, &capture(&cache, &map, false)));

    // Residency still reports this upload; the query payload is unchanged.
    assert!(
        matches!(terrain.refresh(&mut map, &[HOME]).unwrap(), TerrainRefresh::Slots(slots) if slots.len() == 1)
    );
    assert!(Arc::ptr_eq(&frozen, &capture(&cache, &map, false)));
    assert!(Arc::ptr_eq(&frozen, &capture(&cache, &map, false)));

    let edited = Strip(7);
    let edited = PagedTerrain::new(&residency, &edited, &framed, 2);
    edited.refresh(&mut map, &[HOME]).unwrap();
    let updated = capture(&cache, &map, true);
    assert_eq!(sample(&updated, 4.0), Some(7));
    assert_eq!(sample(&frozen, 4.0), Some(3));
}

#[test]
fn camera_only_retarget_and_removal_leave_prior_terrain_queries_frozen() {
    let source = Strip(3);
    let residency = RefCell::new(Residency::new(
        ResidencySettings::default(),
        AtlasLimits::DEFAULT,
    ));
    let home = framed_bricks(view(4.0), 0.0, &source, 1);
    let away = framed_bricks(view(20.0), 0.0, &source, 1);
    let empty = framed_bricks(view(100.0), 0.0, &source, 1);
    assert_eq!(home.keys, [HOME]);
    assert_eq!(away.keys, [[2, 0, 0]]);
    assert!(empty.keys.is_empty());
    let at = |frame| PagedTerrain::new(&residency, &source, frame, 1);
    let mut map = at(&home).brick_map().unwrap();
    let cache = TerrainQueryCache::default();
    let frozen = capture(&cache, &map, true);

    assert!(matches!(
        at(&away).refresh(&mut map, &[]).unwrap(),
        TerrainRefresh::Slots(_)
    ));
    assert_eq!(frozen.pointers(), map.pointers());
    assert_eq!(frozen.atlas(), map.atlas());
    assert_ne!(frozen.origin(), map.origin());
    let moved = capture(&cache, &map, true);
    assert_eq!(sample(&moved, 4.0), None);
    assert_eq!(sample(&moved, 20.0), Some(3));
    assert_eq!(sample(&frozen, 4.0), Some(3));

    assert!(
        matches!(at(&empty).refresh(&mut map, &[]).unwrap(), TerrainRefresh::Slots(slots) if slots.is_empty())
    );
    let removed = capture(&cache, &map, true);
    assert_eq!(sample(&removed, 20.0), None);
    assert_eq!(sample(&moved, 20.0), Some(3));
    assert_eq!(sample(&frozen, 4.0), Some(3));
}

fn single_brick(revision: u64, extent: [u32; 3]) -> BrickMap {
    let mut map = BrickMap::with_capacity(BrickProjectionRevision(revision), 1, extent).unwrap();
    map.retarget_with(BrickProjectionRevision(revision + 1), [HOME], |_| {
        Some(&SOLID[..])
    })
    .unwrap();
    map
}

#[test]
fn full_replacement_reuses_equal_payload_but_checks_query_extent() {
    let mut map = single_brick(1, [2, 1, 1]);
    let cache = TerrainQueryCache::default();
    let frozen = capture(&cache, &map, true);
    let identical = HostTerrain::new(2, Some(single_brick(8, [2, 1, 1])));
    assert_eq!(
        identical.refresh(&mut map, &[]).unwrap(),
        TerrainRefresh::Full
    );
    assert_ne!(frozen.projection_revision(), map.projection_revision());
    assert!(Arc::ptr_eq(&frozen, &capture(&cache, &map, false)));

    let resized = HostTerrain::new(3, Some(single_brick(9, [1, 2, 1])));
    resized.refresh(&mut map, &[]).unwrap();
    assert_eq!(frozen.pointers(), map.pointers());
    assert_eq!(frozen.atlas(), map.atlas());
    assert_ne!(frozen.pointer_extent(), map.pointer_extent());
    capture(&cache, &map, true);
}

#[test]
fn refused_refresh_and_retarget_keep_the_captured_terrain() {
    let mut map = single_brick(1, [1; 3]);
    let cache = TerrainQueryCache::default();
    let frozen = capture(&cache, &map, true);
    let version = map.query_version().clone();

    // Modulus validates the whole dirty set before writing the first brick.
    assert!(
        map.refresh_with([HOME, [1, 0, 0]], |_| Some(&EDITED[..]))
            .is_err()
    );
    assert!(
        map.retarget_with(BrickProjectionRevision(3), [HOME, [1, 0, 0]], |_| Some(
            &EDITED[..]
        ))
        .is_err()
    );
    assert_eq!(*map.query_version(), version);
    assert!(Arc::ptr_eq(&frozen, &capture(&cache, &map, false)));
    assert_eq!(sample(&map, 4.0), Some(3));
}
