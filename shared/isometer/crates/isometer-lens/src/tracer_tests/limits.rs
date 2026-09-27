// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The card-sized atlas: a tracer reports its device's limits under the
//! default budget, and a brick in a slot past the old 2,047 cap draws.

use modulus::MAX_BRICKS;

use crate::{
    ATLAS_BUDGET_BYTES, BrickFrameInput, BrickMap, BrickProjectionRevision, BrickRevision,
    BrickTracer, Flight, Grade,
};

/// A 46 by 46 slab of bricks, 2,116 of them, whose last key is the target.
const SIDE: i16 = 46;
const TARGET: [i16; 3] = [SIDE - 1, 0, SIDE - 1];

fn slab() -> Vec<[i16; 3]> {
    (0..SIDE)
        .flat_map(|x| (0..SIDE).map(move |z| [x, 0, z]))
        .collect()
}

/// A card-sized map holding `selections` in turn, the target brick in its
/// own material, and the slot the target ends in.
fn map(tracer: &BrickTracer, selections: &[&[[i16; 3]]]) -> (BrickMap, u32) {
    let limits = tracer.atlas_limits();
    let (soil, target) = ([1u8; 512], [2u8; 512]);
    let mut map = BrickMap::with_limits(
        BrickProjectionRevision(0),
        limits.max_bricks(),
        [SIDE as u32, 1, SIDE as u32],
        limits,
    )
    .expect("a card-sized map");
    for (revision, keys) in selections.iter().enumerate() {
        map.retarget_with(
            BrickProjectionRevision(revision as u64 + 1),
            keys.iter().copied(),
            |key| {
                Some(if key == TARGET {
                    &target[..]
                } else {
                    &soil[..]
                })
            },
        )
        .expect("the card holds the slab");
    }
    let origin = map.origin();
    let coord = [0, 1, 2].map(|axis| (TARGET[axis] - origin[axis]) as u32);
    let slot = map.pointer_at(coord).expect("in the volume");
    (map, slot)
}

fn picture(map: &BrickMap) -> Option<Vec<u8>> {
    let mut tracer = BrickTracer::headless(32, 32)?;
    // Straight down onto the target brick's top face.
    let camera = Flight {
        eye: [364.0, 20.0, 364.0],
        yaw: 0.0,
        pitch: -1.52,
        fov: 0.15,
        far: 48.0,
    };
    let grade = Grade::clay();
    let frame = BrickFrameInput::new(map, BrickRevision(1), &camera, &grade);
    Some(tracer.capture(frame).expect("a frame").pixels)
}

/// The same slab loaded two ways: at once, which puts the target in slot
/// 2,116, and target first, which puts it in slot 1. Both pictures are the
/// same, and differ from the slab without the target, so the brick past the
/// old cap is drawn rather than read as air.
#[test]
fn a_card_sized_atlas_draws_a_brick_past_the_old_cap() {
    let Some(tracer) = BrickTracer::headless(32, 32) else {
        eprintln!("no adapter; skipping the card-sized atlas receipt");
        return;
    };
    let limits = tracer.atlas_limits();
    assert_eq!(
        limits.max_texture_dimension_3d,
        tracer.device().limits().max_texture_dimension_3d
    );
    assert_eq!(limits.max_atlas_bytes, ATLAS_BUDGET_BYTES);
    assert!(limits.max_bricks() > MAX_BRICKS, "{limits:?}");

    let slab = slab();
    let without: Vec<_> = slab.iter().copied().filter(|key| *key != TARGET).collect();
    let (late, late_slot) = map(&tracer, &[&slab]);
    let (early, early_slot) = map(&tracer, &[&[TARGET], &slab]);
    let (bare, _) = map(&tracer, &[&without]);
    assert!(late_slot as usize > MAX_BRICKS, "slot {late_slot}");
    assert_eq!(early_slot, 1);

    let late = picture(&late).expect("the adapter is still there");
    assert_eq!(late, picture(&early).expect("the early picture"));
    assert_ne!(late, picture(&bare).expect("the bare picture"));
}
