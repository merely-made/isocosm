// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isometer_core::{VolumeRef, Yaw};
use mesocosm_mesh::Volume;

const SUBJECT: SubjectKey = SubjectKey(3);
const REVISION: BodyDependencyRevision = BodyDependencyRevision(11);

fn fixture() -> BodyMesh {
    BodyMesh::single(VolumeRef::from_tag(1), &Volume::solid([1, 4, 6], 1))
}

fn near(a: [f32; 3], b: [f32; 3]) {
    assert!(
        (0..3).all(|i| (a[i] - b[i]).abs() < 0.0001),
        "{a:?} != {b:?}"
    );
}

#[test]
fn largest_meshed_face_follows_part_and_continuous_body_pose() {
    let mut mesh = fixture();
    let p = &mut mesh.placements[0];
    p.pivot = [1, 2, 1];
    p.pivot_at = [8, 3, -5];
    p.yaw = Yaw::Quarter;
    let angle = 0.37_f32;
    let mut body = LiveBody::new(&mesh, [20.0, 7.0, 10.0]);
    body.scale = 2.0;
    body.yaw_radians = angle;
    let a = anchors(SUBJECT, REVISION, &mesh, body, None).unwrap()[0];
    let (s, c) = angle.sin_cos();
    near(
        a.centre,
        [
            20.0 + 2.0 * (10.0 * c - 4.0 * s),
            13.0,
            10.0 + 2.0 * (-10.0 * s - 4.0 * c),
        ],
    );
    near(a.normal, [s, 0.0, c]);
    near(cross(a.right, a.up), a.normal);
    assert!((a.extent[0] - 8.0).abs() < 0.0001);
    assert!((a.extent[1] - 12.0).abs() < 0.0001);
    let ray = [0, 1, 2].map(|i| a.centre[i] + a.normal[i] * 5.0);
    let hit =
        mesocosm_render::live_body::pick_bodies(&[body], ray, a.normal.map(|v| -v), 10.0, None)
            .unwrap()
            .unwrap();
    near(hit.point, a.centre);
    assert_eq!(hit.part, a.part);
}

#[test]
fn anchor_uses_occupied_mesh_face_not_declared_volume_box() {
    let mut volume = Volume::empty([12, 12, 12]);
    volume.set(7, 2, 3, 1);
    let mesh = BodyMesh::single(VolumeRef::from_tag(2), &volume);
    let body = LiveBody::new(&mesh, [0.0; 3]);
    let a = anchors(SUBJECT, REVISION, &mesh, body, None).unwrap()[0];
    near(a.centre, [7.0, 2.5, 3.5]);
    near(a.normal, [-1.0, 0.0, 0.0]);
    assert_eq!(a.extent, [1.0, 1.0]);
}

#[test]
fn selection_is_revision_checked_and_unselected_output_is_bounded_sorted() {
    let mut mesh = fixture();
    let template = mesh.placements[0].clone();
    mesh.placements = (0..40)
        .rev()
        .map(|id| {
            let mut p = template.clone();
            p.part = PartId(id);
            p.pivot_at[0] = id as i32 * 3;
            p
        })
        .collect();
    let body = LiveBody::new(&mesh, [0.0; 3]);
    let all = anchors(SUBJECT, REVISION, &mesh, body, None).unwrap();
    assert_eq!(all.len(), 32);
    assert_eq!(all[0].part, PartId(0));
    assert_eq!(all[31].part, PartId(31));
    let selected = PartAddress {
        subject: SUBJECT,
        revision: REVISION,
        part: PartId(39),
    };
    assert_eq!(
        anchors(SUBJECT, REVISION, &mesh, body, Some(selected)).unwrap()[0].part,
        PartId(39)
    );
    assert!(
        anchors(
            SUBJECT,
            REVISION,
            &mesh,
            body,
            Some(PartAddress {
                revision: BodyDependencyRevision(12),
                ..selected
            })
        )
        .is_err()
    );
    assert!(
        anchors(
            SUBJECT,
            REVISION,
            &mesh,
            body,
            Some(PartAddress {
                part: PartId(99),
                ..selected
            })
        )
        .is_err()
    );
    assert!(
        anchors(
            SUBJECT,
            REVISION,
            &mesh,
            body,
            Some(PartAddress {
                subject: SubjectKey(8),
                ..selected
            })
        )
        .is_err()
    );
}

/// A `SubjectKey` above `u32::MAX` survives the address round trip, so
/// resolving the projector's identity question by narrowing to `OrganismId`
/// would be caught here. Plan §7 condition 4.
#[test]
fn a_subject_key_beyond_u32_survives_an_anchor_address() {
    let wide = SubjectKey(u64::from(u32::MAX) + 7);
    let mesh = fixture();
    let body = LiveBody::new(&mesh, [0.0; 3]);
    let a = anchors(wide, REVISION, &mesh, body, None).unwrap()[0];
    assert_eq!(a.selection.subject, wide);
    assert_eq!(a.selection.revision, REVISION);
    assert_eq!(a.selection.part, a.part);
}
