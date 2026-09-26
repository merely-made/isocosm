// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Receipts for paged terrain, on the CPU: which bricks a frame names, and
//! what the brick map holds after each retarget, rebuild and refresh.

use std::collections::BTreeMap;

use super::paging::span_of;
use super::*;

/// A heightfield `side` columns each way from the origin, stepped so its
/// bricks stand one and two layers tall, with columns an edit can reset.
struct Hills {
    side: i32,
    edits: BTreeMap<(i32, i32), i32>,
}

impl Hills {
    fn new() -> Self {
        Self {
            side: 96,
            edits: BTreeMap::new(),
        }
    }

    fn surface(&self, x: i32, z: i32) -> i32 {
        if x < -self.side || x >= self.side || z < -self.side || z >= self.side {
            return -1;
        }
        match self.edits.get(&(x, z)) {
            Some(surface) => *surface,
            None => (x.div_euclid(7) + z.div_euclid(5)).rem_euclid(11) + 1,
        }
    }

    fn brick(&self, key: [i16; 3]) -> Vec<u8> {
        let mut out = vec![0; BRICK_BYTES];
        self.fill(key, &mut out);
        out
    }
}

impl BrickSource for Hills {
    fn bounds(&self) -> Option<[[i16; 3]; 2]> {
        let key = |v: i32| v.div_euclid(BRICK) as i16;
        Some([
            [key(-self.side), 0, key(-self.side)],
            [key(self.side - 1), key(20), key(self.side - 1)],
        ])
    }

    fn layers(&self, column: [i16; 2]) -> Range<i16> {
        let mut top = -1;
        for dz in 0..BRICK {
            for dx in 0..BRICK {
                let at = [
                    i32::from(column[0]) * BRICK + dx,
                    i32::from(column[1]) * BRICK + dz,
                ];
                top = top.max(self.surface(at[0], at[1]));
            }
        }
        if top < 0 {
            0..0
        } else {
            0..(top.div_euclid(BRICK) as i16 + 1)
        }
    }

    fn fill(&self, key: [i16; 3], out: &mut [u8]) {
        for y in 0..BRICK {
            for z in 0..BRICK {
                for x in 0..BRICK {
                    let at = key_origin(key).map(|v| v as i32);
                    let (vx, vy, vz) = (at[0] + x, at[1] + y, at[2] + z);
                    let surface = self.surface(vx, vz);
                    out[((y * BRICK + z) * BRICK + x) as usize] = match vy {
                        _ if vy < 0 || vy > surface => 0,
                        _ if vy == surface => 2,
                        _ => 1,
                    };
                }
            }
        }
    }
}

fn key_origin(key: [i16; 3]) -> [i64; 3] {
    key.map(|k| i64::from(k) * i64::from(BRICK))
}

fn camera(centre: [f32; 2]) -> SlabCamera {
    SlabCamera::dimetric_2_1([centre[0], 6.0, centre[1]], 24.0, 1.5, 600.0)
        .expect("the test camera frames")
}

/// The bricks a camera shows, by projecting every corner of every brick the
/// terrain holds through the camera's own `ndc_of`, with the frame grown by
/// `margin` world units and then by `slack` in clip space.
fn brute(camera: SlabCamera, margin: f32, slack: f32, hills: &Hills) -> BTreeSet<[i16; 3]> {
    let [low, high] = hills.bounds().expect("bounded");
    let reach = [
        1.0 + margin / (camera.half_height * camera.aspect) + slack,
        1.0 + margin / camera.half_height + slack,
    ];
    let mut shown = BTreeSet::new();
    for z in low[2]..=high[2] {
        for x in low[0]..=high[0] {
            for y in hills.layers([x, z]) {
                let origin = key_origin([x, y, z]);
                let mut box_low = [f32::INFINITY; 2];
                let mut box_high = [f32::NEG_INFINITY; 2];
                for bits in 0..8 {
                    let corner =
                        [0, 1, 2].map(|axis| (origin[axis] + (bits >> axis & 1) * 8) as f32);
                    let ndc = camera.ndc_of(corner).expect("finite");
                    for i in 0..2 {
                        box_low[i] = box_low[i].min(ndc[i]);
                        box_high[i] = box_high[i].max(ndc[i]);
                    }
                }
                if (0..2).all(|i| box_low[i] <= reach[i] && box_high[i] >= -reach[i]) {
                    shown.insert([x, y, z]);
                }
            }
        }
    }
    shown
}

fn keys(framed: &FramedBricks) -> BTreeSet<[i16; 3]> {
    framed.keys.iter().copied().collect()
}

fn frame(hills: &Hills, centre: [f32; 2]) -> FramedBricks {
    framed_bricks(camera(centre), 8.0, hills, usize::MAX)
}

/// Every resident brick reads back through the map exactly as the source
/// makes it, and a brick the map does not hold reads as air.
fn assert_reads(map: &BrickMap, residency: &Residency, hills: &Hills) {
    for key in residency.resident() {
        let expected = hills.brick(*key);
        let origin = key_origin(*key);
        for y in 0..8 {
            for z in 0..8 {
                for x in 0..8 {
                    let at = [origin[0] + x, origin[1] + y, origin[2] + z].map(|v| v as i32);
                    assert_eq!(
                        map.material_at(at),
                        expected[((y * 8 + z) * 8 + x) as usize],
                        "brick {key:?} voxel {at:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn every_framed_brick_is_one_the_frame_can_show() {
    let hills = Hills::new();
    for margin in [0.0, 8.0] {
        for centre in [[0.0, 0.0], [3.0, 9.0], [31.7, -12.3], [-70.2, 55.9]] {
            let view = camera(centre);
            let framed = keys(&framed_bricks(view, margin, &hills, usize::MAX));
            let inside = brute(view, margin, -1e-3, &hills);
            let outside = brute(view, margin, 1e-3, &hills);
            assert!(!inside.is_empty(), "{centre:?}: the frame shows terrain");
            assert!(
                inside.is_subset(&framed),
                "{centre:?} margin {margin}: a brick the frame shows was not named"
            );
            assert!(
                framed.is_subset(&outside),
                "{centre:?} margin {margin}: a brick the frame cannot show was named"
            );
        }
    }
}

#[test]
fn a_framing_is_a_function_of_the_view_and_grows_with_its_margin() {
    let hills = Hills::new();
    let centre = [12.5, -4.25];
    assert_eq!(frame(&hills, centre), frame(&hills, centre));
    let at = |margin| keys(&framed_bricks(camera(centre), margin, &hills, usize::MAX));
    let (none, one, two) = (at(0.0), at(8.0), at(16.0));
    assert!(none.is_subset(&one) && one.is_subset(&two));
    assert!(none.len() < one.len() && one.len() < two.len());
}

#[test]
fn overflow_drops_the_margin_before_the_frame() {
    let hills = Hills::new();
    let view = camera([3.0, 9.0]);
    let exact = framed_bricks(view, 0.0, &hills, usize::MAX);
    let wide = framed_bricks(view, 8.0, &hills, usize::MAX);
    let squeezed = framed_bricks(view, 8.0, &hills, exact.keys.len());
    assert_eq!(wide.overflow, 0);
    assert_eq!(
        squeezed.keys, exact.keys,
        "held to the frame's own count, the frame is what stays"
    );
    assert_eq!(squeezed.overflow, wide.keys.len() - exact.keys.len());
    let tighter = framed_bricks(view, 8.0, &hills, exact.keys.len() / 2);
    assert_eq!(tighter.keys.len(), exact.keys.len() / 2);
    assert_eq!(
        tighter,
        framed_bricks(view, 8.0, &hills, exact.keys.len() / 2),
        "and what it drops is decided the same way every time"
    );
}

#[test]
fn the_extent_holds_every_framing_as_the_camera_pans() {
    let hills = Hills::new();
    let extent = frame(&hills, [0.0, 0.0]).extent;
    for step in 0..40 {
        let centre = [step as f32 * 3.7 - 70.0, 40.0 - step as f32 * 2.9];
        let framed = frame(&hills, centre);
        assert_eq!(framed.extent, extent, "{centre:?}: a pan does not move it");
        let span = span_of(&framed.keys);
        assert!(
            (0..3).all(|axis| span[axis] <= extent[axis]),
            "{centre:?}: {span:?} past {extent:?}"
        );
    }
}

/// Drives one frame's refresh the way the scene does.
fn step(
    residency: &RefCell<Residency>,
    map: &mut BrickMap,
    hills: &Hills,
    framed: &FramedBricks,
    dirty: &[[i16; 3]],
) -> TerrainRefresh {
    PagedTerrain::new(residency, hills, framed, 1)
        .refresh(map, dirty)
        .expect("the refresh")
}

/// The nearest pan from `from` along +x that moves the framing, shrinking it
/// or not as asked.
fn pan(hills: &Hills, from: [f32; 2], shrink: bool) -> [f32; 2] {
    let held = keys(&frame(hills, from));
    (1..400)
        .map(|step| [from[0] + step as f32 * 0.5, from[1]])
        .find(|to| {
            let framed = keys(&frame(hills, *to));
            framed != held && (framed.len() < held.len()) == shrink
        })
        .expect("some pan moves the framing that way")
}

#[test]
fn a_pan_retargets_and_the_map_reads_the_source() {
    let hills = Hills::new();
    let residency = RefCell::new(Residency::new(8));
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
    let residency = RefCell::new(Residency::new(8));
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
    let mut raw =
        BrickMap::with_capacity(BrickProjectionRevision(1), 8, first.extent).expect("a raw map");
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
    let residency = RefCell::new(Residency::new(8));
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
    let residency = RefCell::new(Residency::new(8));
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
    for rows in 1..=8 {
        let map = BrickMap::with_capacity(BrickProjectionRevision(0), rows, [1, 1, 1])
            .expect("a capacity map");
        assert_eq!(Residency::new(rows).capacity(), map.capacity());
    }
    assert_eq!(Residency::new(0).capacity(), Residency::new(1).capacity());
    assert_eq!(Residency::new(99).capacity(), Residency::new(8).capacity());
}
