// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receipts for the map that follows the framed bricks: retargets, the hold,
//! refreshes, rebuilds and headroom.

use super::*;

#[test]
fn a_pan_retargets_and_the_map_reads_the_source() {
    let hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default()));
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

/// The hold: a shrinking selection rebuilds from empty, and a kept brick an
/// edit then touches is refreshed in place and read back right.
///
/// The control is the same pan driven straight through modulus's retarget:
/// it leaves a kept brick drawn by the pointer volume yet read as air. When
/// the pin moves past the fix the control fails, and the hold and this
/// control retire together.
#[test]
fn a_shrinking_selection_rebuilds_packed_and_edits_still_land() {
    let mut hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default()));
    let from = [0.0, 0.0];
    let first = frame(&hills, from);
    let mut map = PagedTerrain::new(&residency, &hills, &first, 1)
        .brick_map()
        .expect("the first map");
    let to = pan(&hills, from, true);
    let next = frame(&hills, to);
    assert!(next.keys.len() < first.keys.len());
    let refresh = step(&residency, &mut map, &hills, &next, &[]);
    assert_eq!(refresh, TerrainRefresh::Full);
    assert_eq!(residency.borrow().stats().rebuilt, Some(Rebuild::Shrink));
    assert_reads(&map, &residency.borrow(), &hills);

    // An edit on a kept brick: raise one column inside it.
    let kept = *next.keys.last().expect("bricks are framed");
    let origin = key_origin(kept).map(|v| v as i32);
    hills.edits.insert((origin[0] + 3, origin[2] + 4), 20);
    let refresh = step(&residency, &mut map, &hills, &next, &[kept]);
    assert!(matches!(refresh, TerrainRefresh::Slots(ref slots) if slots.len() == 1));
    assert_eq!(residency.borrow().stats().refreshed, 1);
    assert_reads(&map, &residency.borrow(), &hills);

    // The control: the same two selections straight through modulus.
    let hills = Hills::new();
    let extent = first.extent(first.layers().expect("terrain"));
    let mut raw =
        BrickMap::with_capacity(BrickProjectionRevision(1), 8, extent).expect("a raw map");
    let made = |framed: &FramedBricks| -> Vec<Vec<u8>> {
        framed.keys.iter().map(|key| hills.brick(*key)).collect()
    };
    for (revision, framed) in [(2, &first), (3, &next)] {
        let bricks = made(framed);
        raw.retarget_with(
            BrickProjectionRevision(revision),
            framed.keys.iter().copied(),
            |key| {
                let index = framed.keys.binary_search(&key).ok()?;
                Some(bricks[index].as_slice())
            },
        )
        .expect("modulus takes the selection");
    }
    let origin = raw.origin();
    let unreadable = next.keys.iter().find(|key| {
        let coord = [0, 1, 2].map(|i| (i32::from(key[i]) - i32::from(origin[i])) as u32);
        let slot = raw.pointer_at(coord).unwrap_or(0) as usize;
        let brick = hills.brick(**key);
        let Some(solid) = brick.iter().position(|material| *material != 0) else {
            return false;
        };
        let at = key_origin(**key).map(|v| v as i32);
        let voxel = [
            at[0] + (solid % 8) as i32,
            at[1] + (solid / 64) as i32,
            at[2] + (solid / 8 % 8) as i32,
        ];
        slot > next.keys.len() && raw.material_at(voxel) == 0
    });
    assert!(
        unreadable.is_some(),
        "modulus now reads a kept brick after a shrinking retarget: the pin has \
         moved past the fix, so retire Rebuild::Shrink and this control"
    );
}

#[test]
fn an_edit_to_an_absent_brick_lands_when_it_returns() {
    let mut hills = Hills::new();
    let residency = RefCell::new(Residency::new(ResidencySettings::default()));
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
    let residency = RefCell::new(Residency::new(ResidencySettings::default()));
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

#[test]
fn the_capacity_is_the_atlas_modulus_builds() {
    let rows = |rows| {
        Residency::new(ResidencySettings {
            rows,
            ..ResidencySettings::default()
        })
        .capacity()
    };
    for count in 1..=8 {
        let map = BrickMap::with_capacity(BrickProjectionRevision(0), count, [1, 1, 1])
            .expect("a capacity map");
        assert_eq!(rows(count), map.capacity());
    }
    assert_eq!(rows(0), rows(1));
    assert_eq!(rows(99), rows(8));
    assert_eq!(
        Residency::new(ResidencySettings::default()).capacity(),
        rows(8),
        "the default is every row modulus allows"
    );
}

/// Headroom: a pointer volume reserving spare layers takes an edit that lifts
/// the terrain's tallest point into them as a retarget, and only an edit past
/// them rebuilds. The control is the same edits with no headroom, where the
/// first one already rebuilds.
#[test]
fn an_edit_within_the_headroom_retargets_and_one_past_it_rebuilds() {
    for headroom in [1, 0] {
        let mut hills = Hills::new();
        let settings = ResidencySettings {
            headroom,
            ..ResidencySettings::default()
        };
        let residency = RefCell::new(Residency::new(settings));
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
        // gone, so the framing shrinks, and while the hold stands that alone
        // rebuilds, at the same reserve.
        let dirty = lift(&mut hills, 5);
        let lowered = frame(&hills, [0.0, 0.0]);
        step(&residency, &mut map, &hills, &lowered, &dirty);
        let stats = residency.borrow().stats();
        assert!(
            matches!(stats.rebuilt, None | Some(Rebuild::Shrink)),
            "lowering is never a height rebuild: {:?}",
            stats.rebuilt
        );
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
    let residency = RefCell::new(Residency::new(ResidencySettings {
        headroom: 0,
        ..ResidencySettings::default()
    }));
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
