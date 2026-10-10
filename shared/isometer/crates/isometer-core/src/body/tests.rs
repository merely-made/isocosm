// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Body-document unit tests. Split from `body.rs` at the 600-line ceiling,
//! same module, a separate file, per the `places/bricks/tests.rs` precedent.

use super::*;

fn seed_body() -> BodyDocument {
    BodyDocument::new(VolumeRef::from_tag(1), [2, 2, 2])
}

#[test]
fn root_sits_at_origin() {
    let body = seed_body();
    // The root's *pivot* is the origin, so a body is centred on it rather
    // than cornered at it, and the midline is genuinely zero.
    assert_eq!(body.world_pivot(body.root), Some([0, 0, 0]));
    assert_eq!(body.world_offset(body.root), Some([-2, -2, -2]));
}

#[test]
fn attaching_extends_the_collision_box() {
    let mut body = seed_body();
    let before = body.aabb();
    body.attach(
        VolumeRef::from_tag(2),
        [1, 1, 1],
        Attachment {
            parent: body.root,
            offset: [6, 0, 0],
            yaw: Yaw::Zero,
        },
        Some(1),
    )
    .expect("root exists");
    let after = body.aabb();
    assert!(after.extent()[0] > before.extent()[0]);
    assert_eq!(after.max[0], 7);
}

#[test]
fn attaching_moves_the_centre_of_mass() {
    let mut body = seed_body();
    // A lone body's centre of mass is its pivot, which is the origin.
    let equal = |_| 1_000;
    assert_eq!(body.centre_of_mass(equal), [0, 0, 0]);

    body.attach(
        VolumeRef::from_tag(2),
        [1, 1, 1],
        Attachment {
            parent: body.root,
            offset: [10, 0, 0],
            yaw: Yaw::Zero,
        },
        None,
    )
    .unwrap();

    // Equal masses at [0,0,0] and [10,0,0]: halfway between.
    assert_eq!(body.centre_of_mass(equal), [5, 0, 0]);
}

#[test]
fn yaw_rotates_child_offsets_exactly() {
    let mut body = seed_body();
    let arm = body
        .attach(
            VolumeRef::from_tag(2),
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [4, 0, 0],
                yaw: Yaw::Quarter,
            },
            None,
        )
        .unwrap();
    let hand = body
        .attach(
            VolumeRef::from_tag(3),
            [1, 1, 1],
            Attachment {
                parent: arm,
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
    // The hand's own offset is rotated by the arm's quarter turn. Pivots
    // chain exactly as corners used to, but now a rotation swings a part
    // about its own centre instead of flinging it off its corner.
    assert_eq!(body.world_pivot(hand), Some([4, 0, -4]));
}

#[test]
fn nested_yaw_accumulates_up_the_chain() {
    let mut body = seed_body();
    let arm = body
        .attach(
            VolumeRef::from_tag(2),
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [4, 0, 0],
                yaw: Yaw::Quarter,
            },
            None,
        )
        .unwrap();
    let hand = body
        .attach(
            VolumeRef::from_tag(3),
            [1, 1, 1],
            Attachment {
                parent: arm,
                offset: [2, 0, 0],
                yaw: Yaw::Half,
            },
            None,
        )
        .unwrap();

    assert_eq!(body.world_yaw(body.root), Some(Yaw::Zero));
    assert_eq!(body.world_yaw(arm), Some(Yaw::Quarter));
    // Quarter turn at the shoulder plus a half turn at the wrist.
    assert_eq!(body.world_yaw(hand), Some(Yaw::ThreeQuarter));
}

#[test]
fn unknown_parent_is_refused() {
    let mut body = seed_body();
    let err = body
        .attach(
            VolumeRef::from_tag(2),
            [1, 1, 1],
            Attachment {
                parent: PartId(99),
                offset: [0, 0, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap_err();
    assert_eq!(err, AttachError::UnknownParent(PartId(99)));
}

#[test]
fn tagged_parts_are_listed_in_order() {
    let mut body = seed_body();
    for tag in 2..5u8 {
        body.attach(
            VolumeRef::from_tag(tag),
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [tag as i32, 0, 0],
                yaw: Yaw::Zero,
            },
            Some(1),
        )
        .unwrap();
    }
    let ids: Vec<_> = body.tagged().map(|p| p.id).collect();
    assert_eq!(ids, vec![PartId(1), PartId(2), PartId(3)]);
}
