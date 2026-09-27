// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receipts for the map that follows the framed bricks: retargets,
//! refreshes, rebuilds and headroom.

use super::*;

#[test]
fn a_pan_retargets_and_the_map_reads_the_source() {
    let hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default(), LIMITS));
    let from = [0.0, 0.0];
    let first = frame(&hills, from);
    let mut map = PagedTerrain::new(&residency, &hills, &first, 1)
        .brick_map()
        .expect("the first map");
    assert_eq!(residency.borrow().stats().rebuilt, Some(Rebuild::First));
    assert_reads(&map, &residency.borrow(), &hills);

    let to = pan(&hills, from, false);
    let next = frame(&hills, to);
    let before = residency.borrow().resident().clone();
    let refresh = step(&residency, &mut map, &hills, &next, &[]);
    let stats = residency.borrow().stats();
    let arrived = keys(&next).difference(&before).count();
    let left = before.difference(&keys(&next)).count();
    assert_eq!(stats.rebuilt, None, "a growing pan retargets");
    assert_eq!((stats.loaded, stats.evicted), (arrived, left));
    assert!(matches!(refresh, TerrainRefresh::Slots(ref slots) if slots.len() == arrived));
    assert_eq!(residency.borrow().resident(), &keys(&next));
    assert_reads(&map, &residency.borrow(), &hills);
    let gone = before.difference(&keys(&next)).next().copied();
    if let Some(gone) = gone {
        let at = key_origin(gone).map(|v| v as i32);
        assert_eq!(map.material_at(at), 0, "a brick let go reads as air");
    }
    let unchanged = step(&residency, &mut map, &hills, &next, &[]);
    assert_eq!(
        unchanged,
        TerrainRefresh::Current,
        "a still frame moves nothing"
    );
}

/// A shrinking selection retargets like any other: a kept brick left in a
/// slot past the resident count reads right, and an edit to it refreshes in
/// place.
#[test]
fn a_shrinking_selection_retargets_and_edits_still_land() {
    let mut hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default(), LIMITS));
    let from = [0.0, 0.0];
    let first = frame(&hills, from);
    let mut map = PagedTerrain::new(&residency, &hills, &first, 1)
        .brick_map()
        .expect("the first map");
    let to = pan(&hills, from, true);
    let next = frame(&hills, to);
    assert!(next.keys.len() < first.keys.len());
    let refresh = step(&residency, &mut map, &hills, &next, &[]);
    assert!(matches!(refresh, TerrainRefresh::Slots(_)));
    assert_eq!(
        residency.borrow().stats().rebuilt,
        None,
        "a shrinking pan retargets"
    );
    assert_reads(&map, &residency.borrow(), &hills);

    // The case a packed map never meets: a kept brick in a slot past the
    // resident count, with a column whose surface lies inside it, so an edit
    // can move that surface without touching any other brick.
    let origin = map.origin();
    let slot = |key: [i16; 3]| {
        let coord = [0, 1, 2].map(|i| (i32::from(key[i]) - i32::from(origin[i])) as u32);
        map.pointer_at(coord).unwrap_or(0) as usize
    };
    let inside = |key: [i16; 3]| {
        let at = key_origin(key).map(|v| v as i32);
        let low = i32::from(key[1]) * BRICK;
        (0..BRICK * BRICK)
            .map(|i| (at[0] + i % BRICK, at[2] + i / BRICK))
            .find(|(x, z)| (low..low + BRICK).contains(&hills.surface(*x, *z)))
    };
    let (kept, (x, z)) = next
        .keys
        .iter()
        .copied()
        .filter(|key| slot(*key) > next.keys.len())
        .find_map(|key| inside(key).map(|column| (key, column)))
        .expect("the pan leaves a kept brick past the resident count");

    // The edit: that column's surface moved within the brick's layer.
    let (surface, low) = (hills.surface(x, z), i32::from(kept[1]) * BRICK);
    let moved = if surface + 1 < low + BRICK {
        surface + 1
    } else {
        low
    };
    hills.edits.insert((x, z), moved);
    let refresh = step(&residency, &mut map, &hills, &next, &[kept]);
    assert!(matches!(refresh, TerrainRefresh::Slots(ref slots) if slots.len() == 1));
    assert_eq!(residency.borrow().stats().refreshed, 1);
    assert_reads(&map, &residency.borrow(), &hills);
}

#[test]
fn an_edit_to_an_absent_brick_lands_when_it_returns() {
    let mut hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default(), LIMITS));
    let home = [0.0, 0.0];
    let first = frame(&hills, home);
    let mut map = PagedTerrain::new(&residency, &hills, &first, 1)
        .brick_map()
        .expect("the first map");
    let away = frame(&hills, [150.0, -150.0]);
    let absent = *first
        .keys
        .iter()
        .find(|key| key[1] == 0 && !away.keys.contains(key))
        .expect("the far view lets some ground-layer brick go");
    step(&residency, &mut map, &hills, &away, &[]);
    assert!(!residency.borrow().resident().contains(&absent));

    // Lower every column of the absent brick's footprint to the floor: every
    // one stood at least two voxels, so its bytes are sure to change, and one
    // voxel still stands, so it is still framed when the view comes back.
    let before = hills.brick(absent);
    let origin = key_origin(absent).map(|v| v as i32);
    for dz in 0..BRICK {
        for dx in 0..BRICK {
            hills.edits.insert((origin[0] + dx, origin[2] + dz), 0);
        }
    }
    assert_ne!(hills.brick(absent), before, "the edit reaches the brick");
    assert_eq!(
        step(&residency, &mut map, &hills, &away, &[absent]),
        TerrainRefresh::Current,
        "a brick the map does not hold is not refreshed"
    );

    step(&residency, &mut map, &hills, &frame(&hills, home), &[]);
    assert!(residency.borrow().resident().contains(&absent));
    assert_reads(&map, &residency.borrow(), &hills);
}

#[test]
fn a_requested_rebuild_replaces_the_map() {
    let hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default(), LIMITS));
    let framed = frame(&hills, [0.0, 0.0]);
    let mut map = PagedTerrain::new(&residency, &hills, &framed, 1)
        .brick_map()
        .expect("the first map");
    residency.borrow_mut().rebuild();
    assert_eq!(
        step(&residency, &mut map, &hills, &framed, &[]),
        TerrainRefresh::Full
    );
    assert_eq!(residency.borrow().stats().rebuilt, Some(Rebuild::Requested));
    assert_eq!(
        step(&residency, &mut map, &hills, &framed, &[]),
        TerrainRefresh::Current
    );
}

/// Framing may change its omitted outer bricks while retaining exactly the
/// same keys. Reporting that demand must not pretend the atlas changed.
#[test]
fn current_overflow_changes_without_uploading_the_same_retained_keys() {
    let hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default(), LIMITS));
    let mut framed = frame(&hills, [0.0, 0.0]);
    assert!(!framed.keys.is_empty(), "the retained set is real terrain");
    framed.overflow = 7;
    let mut map = PagedTerrain::new(&residency, &hills, &framed, 1)
        .brick_map()
        .expect("the initial map");
    assert_eq!(residency.borrow().overflow(), 7);
    let last_change = residency.borrow().stats();
    let changes = residency.borrow().changes();
    let projection = map.projection_revision();
    let atlas = map.atlas().to_vec();
    for omitted in [12, 3, 0] {
        framed.overflow = omitted;
        assert_eq!(
            step(&residency, &mut map, &hills, &framed, &[]),
            TerrainRefresh::Current,
            "changing only the omission count requires no upload or rebuild"
        );
        assert_eq!(residency.borrow().overflow(), omitted, "current frame");
        assert_eq!(residency.borrow().stats(), last_change, "last map change");
        assert_eq!(residency.borrow().changes(), changes);
        assert_eq!(map.projection_revision(), projection);
        assert_eq!(map.atlas(), atlas);
        assert_reads(&map, &residency.borrow(), &hills);
    }
}

/// The capacity is every brick the card allows, in whole rows of 256 slots
/// less the air slot, and modulus builds a map of exactly that capacity:
/// the historical atlas, one row, a budget between rows, the 8 MiB default,
/// and a card whose texture edge binds before the budget.
#[test]
fn the_capacity_is_the_atlas_the_card_allows() {
    let row = 16 * 16 * 512;
    let card = |edge, bytes| AtlasLimits {
        max_texture_dimension_3d: edge,
        max_atlas_bytes: bytes,
    };
    for (limits, bricks) in [
        (LIMITS, 2_047),
        (card(2048, row), 255),
        (card(2048, 3 * row + 1), 767),
        (card(2048, ATLAS_BUDGET_BYTES), 16_383),
        (card(256, ATLAS_BUDGET_BYTES), 8_191),
    ] {
        let capacity = Residency::new(ResidencySettings::default(), limits).capacity();
        assert_eq!(capacity, bricks, "{limits:?}");
        let map = BrickMap::with_limits(BrickProjectionRevision(0), capacity, [1, 1, 1], limits)
            .expect("a map at the card's size");
        assert_eq!(map.capacity(), capacity, "{limits:?}");
    }
}

/// Headroom: a pointer volume reserving spare layers takes an edit that lifts
/// the terrain's tallest point into them as a retarget, and only an edit past
/// them rebuilds. The control is the same edits with no headroom, where the
/// first one already rebuilds.
#[test]
fn an_edit_within_the_headroom_retargets_and_one_past_it_rebuilds() {
    for headroom in [1, 0] {
        let mut hills = Hills::new();
        let residency = RefCell::new(Residency::new(ResidencySettings { headroom }, LIMITS));
        let first = frame(&hills, [0.0, 0.0]);
        assert_eq!(first.layers(), Some([0, 1]), "the field stands two layers");
        let mut map = PagedTerrain::new(&residency, &hills, &first, 1)
            .brick_map()
            .expect("the first map");
        let reserved = residency.borrow().stats().reserved;
        assert_eq!(reserved, [0, 1 + headroom as i16], "headroom {headroom}");

        // Into the next layer: surface 20 is layer 2.
        let dirty = lift(&mut hills, 20);
        let next = frame(&hills, [0.0, 0.0]);
        assert_eq!(next.layers(), Some([0, 2]));
        let refresh = step(&residency, &mut map, &hills, &next, &dirty);
        let stats = residency.borrow().stats();
        if headroom == 0 {
            assert_eq!(
                refresh,
                TerrainRefresh::Full,
                "no headroom: the control rebuilds"
            );
            assert_eq!(stats.rebuilt, Some(Rebuild::Headroom));
            assert_reads(&map, &residency.borrow(), &hills);
            continue;
        }
        assert!(
            matches!(refresh, TerrainRefresh::Slots(_)),
            "within the headroom"
        );
        assert_eq!(stats.rebuilt, None);
        assert!(
            residency.borrow().resident().contains(&[0, 2, 0]),
            "the new layer's brick is held"
        );
        assert_eq!(stats.extent, residency.borrow().stats().extent);
        assert_reads(&map, &residency.borrow(), &hills);

        // Lowered again: nothing rebuilds for height. The layer-2 brick is
        // gone, so the framing shrinks, and the map retargets to it.
        let dirty = lift(&mut hills, 5);
        let lowered = frame(&hills, [0.0, 0.0]);
        step(&residency, &mut map, &hills, &lowered, &dirty);
        let stats = residency.borrow().stats();
        assert_eq!(stats.rebuilt, None, "lowering never rebuilds");
        assert_eq!(stats.reserved, [0, 2]);
        assert_reads(&map, &residency.borrow(), &hills);

        // Past the headroom: surface 28 is layer 3.
        let dirty = lift(&mut hills, 28);
        let past = frame(&hills, [0.0, 0.0]);
        assert_eq!(past.layers(), Some([0, 3]));
        let refresh = step(&residency, &mut map, &hills, &past, &dirty);
        assert_eq!(refresh, TerrainRefresh::Full);
        let stats = residency.borrow().stats();
        assert_eq!(stats.rebuilt, Some(Rebuild::Headroom));
        assert_eq!(
            stats.reserved,
            [0, 4],
            "rebuilt with the headroom above the new top"
        );
        assert_reads(&map, &residency.borrow(), &hills);
    }
}

/// New settings rebuild the map at them, and the headroom reserves pointer
/// layers above the terrain across and up: the camera leans, so a taller slab
/// is seen over a wider stretch of ground.
#[test]
fn settings_rebuild_the_map_at_their_headroom() {
    let hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings { headroom: 0 }, LIMITS));
    let framed = frame(&hills, [0.0, 0.0]);
    let mut map = PagedTerrain::new(&residency, &hills, &framed, 1)
        .brick_map()
        .expect("the first map");
    let bare = map.pointer_extent();
    residency
        .borrow_mut()
        .set_settings(ResidencySettings::default());
    assert_eq!(
        step(&residency, &mut map, &hills, &framed, &[]),
        TerrainRefresh::Full
    );
    assert_eq!(residency.borrow().stats().rebuilt, Some(Rebuild::Requested));
    let spare = map.pointer_extent();
    assert_eq!(spare[1], bare[1] + 1, "one more layer up");
    assert!(spare[0] >= bare[0] && spare[2] >= bare[2]);
    assert_eq!(
        step(&residency, &mut map, &hills, &framed, &[]),
        TerrainRefresh::Current,
        "the same settings again change nothing"
    );
    residency
        .borrow_mut()
        .set_settings(ResidencySettings::default());
    assert_eq!(
        step(&residency, &mut map, &hills, &framed, &[]),
        TerrainRefresh::Current
    );
}
