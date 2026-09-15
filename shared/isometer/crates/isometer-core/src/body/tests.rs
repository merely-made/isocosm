// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Body-document unit tests. Split from `body.rs` at the 600-line ceiling,
//! same module, a separate file, per the `places/bricks/tests.rs` precedent.

use super::*;

fn seed_body() -> BodyDocument {
    BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2])
}

#[test]
fn root_sits_at_origin() {
    let body = seed_body();
    // The root's *pivot* is the origin, so a body is centred on it rather
    // than cornered at it, and the midline is genuinely zero.
    assert_eq!(body.world_pivot(body.root), Some([0, 0, 0]));
    assert_eq!(body.world_offset(body.root), Some([-2, -2, -2]));
    assert_eq!(body.total_mass_mg(), 1_000);
}

#[test]
fn attaching_extends_the_collision_box() {
    let mut body = seed_body();
    let before = body.aabb();
    body.attach(
        VolumeRef::from_tag(2),
        500,
        [1, 1, 1],
        Attachment {
            parent: body.root,
            offset: [6, 0, 0],
            yaw: Yaw::Zero,
        },
        Provenance {
            origin: Origin::Incorporated {
                from_species: SpeciesId(9),
                from_part: PartId(0),
            },
            epoch: 1,
        },
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
    assert_eq!(body.centre_of_mass(), [0, 0, 0]);

    body.attach(
        VolumeRef::from_tag(2),
        1_000,
        [1, 1, 1],
        Attachment {
            parent: body.root,
            offset: [10, 0, 0],
            yaw: Yaw::Zero,
        },
        Provenance::founding(),
    )
    .unwrap();

    // Equal masses at [0,0,0] and [10,0,0]: halfway between.
    assert_eq!(body.centre_of_mass(), [5, 0, 0]);
}

#[test]
fn yaw_rotates_child_offsets_exactly() {
    let mut body = seed_body();
    let arm = body
        .attach(
            VolumeRef::from_tag(2),
            100,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [4, 0, 0],
                yaw: Yaw::Quarter,
            },
            Provenance::founding(),
        )
        .unwrap();
    let hand = body
        .attach(
            VolumeRef::from_tag(3),
            100,
            [1, 1, 1],
            Attachment {
                parent: arm,
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
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
            100,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [4, 0, 0],
                yaw: Yaw::Quarter,
            },
            Provenance::founding(),
        )
        .unwrap();
    let hand = body
        .attach(
            VolumeRef::from_tag(3),
            100,
            [1, 1, 1],
            Attachment {
                parent: arm,
                offset: [2, 0, 0],
                yaw: Yaw::Half,
            },
            Provenance::founding(),
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
            1,
            [1, 1, 1],
            Attachment {
                parent: PartId(99),
                offset: [0, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap_err();
    assert_eq!(err, AttachError::UnknownParent(PartId(99)));
}

#[test]
fn incorporated_parts_are_listed_in_order() {
    let mut body = seed_body();
    for tag in 2..5u8 {
        body.attach(
            VolumeRef::from_tag(tag),
            10,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset: [tag as i32, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance {
                origin: Origin::Incorporated {
                    from_species: SpeciesId(tag as u32),
                    from_part: PartId(0),
                },
                epoch: 1,
            },
        )
        .unwrap();
    }
    let ids: Vec<_> = body.incorporated().map(|p| p.id).collect();
    assert_eq!(ids, vec![PartId(1), PartId(2), PartId(3)]);
}
