// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isometer_core::{Attachment, VolumeRef, Yaw};
use isometer_mesh::{Volume, VolumeMap, mesh_body};

#[test]
fn actual_three_part_chain_lifts_a_rigid_foot_with_fixed_links_and_part_identity() {
    let root = VolumeRef::from_tag(1);
    let mut doc = BodyDocument::new(root, [2; 3]);
    let mut volumes = VolumeMap::new();
    volumes.insert(root, Volume::solid([4; 3], 1));
    let mut parent = doc.root;
    let mut ids = Vec::new();
    for (n, half, offset) in [
        (2, [3, 1, 1], [5, 0, 0]),
        (3, [1, 3, 1], [3, -4, 0]),
        (4, [1, 1, 3], [0, -3, -4]),
    ] {
        let tag = VolumeRef::from_tag(n);
        parent = doc
            .attach(
                tag,
                half,
                Attachment {
                    parent,
                    offset,
                    yaw: Yaw::Zero,
                },
                None,
            )
            .unwrap();
        volumes.insert(tag, Volume::solid(half.map(|h| h as u32 * 2), 1));
        ids.push(parent);
    }
    let mesh = mesh_body(&doc, &volumes).unwrap();
    let foot = ids[2];
    let mut gait = Gait::default();
    let mut ground = |_| Some(0.);
    let before = BTreeMap::from([(foot, Vec3::new(8.5, 0., -3.5))]);
    let (rest, stats) = crate::implicit::walking_anatomy(
        &doc,
        LiveBody::new(&mesh, [0., 8., 0.]),
        &mut ground,
        &before,
        &mut gait,
        0.02,
    )
    .unwrap();
    assert_eq!(
        stats.contacts, 1,
        "the real chain must reach its native foot witness"
    );
    gait.set_native_origin([1., 8., 0.]);
    gait.set_envelope_offset([-0.2, 0., 0.], [0.8, 0., 0.]);
    gait.start_step();
    let after = BTreeMap::from([(foot, Vec3::new(9.5, 0., -3.5))]);
    let (moving, stats) = crate::implicit::walking_anatomy(
        &doc,
        LiveBody::new(&mesh, [0.2, 8., 0.]),
        &mut ground,
        &after,
        &mut gait,
        0.02,
    )
    .unwrap();
    assert_eq!(
        stats.swinging, 1,
        "an admitted move must produce a visible real-foot swing"
    );
    let pad = |surface: &Surface| {
        surface
            .parts
            .iter()
            .find(|p| p.meta[1] == foot.0 && p.meta[0] == 0)
            .copied()
            .unwrap()
    };
    assert!(pad(&moving).a[1] > pad(&rest).a[1]);
    for id in &ids[..2] {
        let link = |surface: &Surface| *surface.parts.iter().find(|p| p.meta[1] == id.0).unwrap();
        let len = |p: Primitive| Vec3::from_slice(&p.a).distance(Vec3::from_slice(&p.b));
        assert!((len(link(&rest)) - len(link(&moving))).abs() < 0.001);
    }
    assert_eq!(doc.sever(ids[0]).len(), 3);
    let mesh = mesh_body(&doc, &volumes).unwrap();
    let (severed, _) = crate::implicit::walking_anatomy(
        &doc,
        LiveBody::new(&mesh, [0.2, 8., 0.]),
        &mut ground,
        &after,
        &mut gait,
        0.02,
    )
    .unwrap();
    assert!(!severed.parts.iter().any(|p| p.meta[1] == foot.0));
}

#[test]
fn capsule_clearance_covers_shared_faces_and_refuses_narrow_gaps() {
    let adjacent = [
        [Vec3::new(-1.,0.,-1.), Vec3::new(1.,2.,1.)],
        [Vec3::new(-1.,2.,-1.), Vec3::new(1.,4.,1.)],
    ];
    assert!(segment_fits(Vec3::Y, Vec3::Y*3., &corridors(&adjacent), 0.3));
    let mut gap = adjacent;
    gap[1][0].y += 0.01;
    assert!(!segment_fits(Vec3::Y, Vec3::Y*3., &corridors(&gap), 0.3));
}
