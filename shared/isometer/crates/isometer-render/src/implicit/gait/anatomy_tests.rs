// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use crate::{LiveBody, implicit::*};
use glam::Vec3;
use isometer_core::{Attachment, BodyDocument, PartId, VolumeRef, Yaw};
use isometer_mesh::{Volume, VolumeMap, mesh_body};
use std::collections::BTreeMap;

#[test]
fn graph_limbs_plant_fixed_length_links_and_queries_follow_the_posed_surface() {
    let tag = VolumeRef::from_tag(1);
    let mut document = BodyDocument::new(tag, [2; 3]);
    let leg = document
        .attach(
            tag,
            [2; 3],
            Attachment {
                parent: document.root,
                offset: [0, -4, 0],
                yaw: Yaw::Zero,
            },
            None,
        )
        .unwrap();
    let mut volumes = VolumeMap::new();
    volumes.insert(tag, Volume::solid([4; 3], 1));
    let mesh = mesh_body(&document, &volumes).unwrap();
    let mut gait = Gait::default();
    let mut ground = |_| Some(0.);
    let contacts = BTreeMap::from([(leg, Vec3::new(-0.5, 0., -0.5))]);
    let (first, stats) = walking_anatomy(
        &document,
        LiveBody::new(&mesh, [0., 6., 0.]),
        &mut ground,
        &contacts,
        &mut gait,
        0.02,
    )
    .unwrap();
    assert_eq!(stats.planted, 1);
    assert_eq!(stats.contacts, 1);
    let mut moving = LiveBody::new(&mesh, [0.3, 6., 0.]);
    let contacts = BTreeMap::from([(leg, Vec3::new(-0.2, 0., -0.5))]);
    let (second, stats) =
        walking_anatomy(&document, moving, &mut ground, &contacts, &mut gait, 0.02).unwrap();
    assert_eq!(stats.planted, 1);
    let foot_of = |s: &Surface| {
        s.parts
            .iter()
            .filter(|p| p.meta[1] == leg.0)
            .nth(1)
            .unwrap()
            .b
    };
    assert!(
        (0..3).all(|k| (foot_of(&first)[k] - foot_of(&second)[k]).abs() < 1e-5),
        "the IK sole stays planted while the hip moves"
    );
    let pad = second
        .parts
        .iter()
        .rev()
        .find(|p| p.meta[1] == leg.0)
        .unwrap();
    assert!(
        (pad.b[0] + 0.5).abs() < 1e-6,
        "stance must not slide with the hip"
    );
    assert_ne!(first.parts[1].a, second.parts[1].a);
    let links: Vec<_> = second
        .parts
        .iter()
        .filter(|p| p.meta[1] == leg.0)
        .take(2)
        .collect();
    for (a, b) in first
        .parts
        .iter()
        .filter(|p| p.meta[1] == leg.0)
        .take(2)
        .zip(links)
    {
        let length = |p: &Primitive| {
            Vec3::from_array(p.a[..3].try_into().unwrap())
                .distance(Vec3::from_array(p.b[..3].try_into().unwrap()))
        };
        assert!((length(a) - length(b)).abs() < 0.001);
    }
    let hit = second
        .pick(
            [-0.5, 0.1, -10.],
            [0., 0., 1.],
            20.,
            crate::ClipSlab::new([0., 0., 1.], -100., 100.),
        )
        .unwrap()
        .unwrap();
    assert_eq!(hit.part, leg);
    let mut stepping = Gait::default();
    let original = BTreeMap::from([(leg, Vec3::new(-0.5, 0., -0.5))]);
    walking_anatomy(
        &document,
        LiveBody::new(&mesh, [0., 6., 0.]),
        &mut ground,
        &original,
        &mut stepping,
        0.02,
    )
    .unwrap();
    stepping.start_step();
    stepping.set_envelope_offset([0.; 3], [1., 0., 0.]);
    let ahead = BTreeMap::from([(leg, Vec3::new(0.5, 0., -0.5))]);
    let (swing, stats) = walking_anatomy(
        &document,
        LiveBody::new(&mesh, [0., 6., 0.]),
        &mut ground,
        &ahead,
        &mut stepping,
        0.02,
    )
    .unwrap();
    assert_eq!(
        stats.swinging, 1,
        "an admitted cell step must start the limb's swing"
    );
    assert!(
        foot_of(&swing)[1] > foot_of(&first)[1] + 0.01,
        "the fixed links must lift the sole, not leave a static leg behind"
    );
    // The old sole is outside the instantaneous rigid box after this move,
    // but remains within the native-cleared union of the two adjacent poses.
    gait.set_envelope_offset([-0.9, 0., 0.], [0.1, 0., 0.]);
    moving.origin[0] = 0.9;
    let held = BTreeMap::from([(leg, Vec3::new(-0.5, 0., -0.5))]);
    let (swept, stats) =
        walking_anatomy(&document, moving, &mut ground, &held, &mut gait, 0.02).unwrap();
    assert_eq!(stats.planted, 1);
    assert!((0..3).all(|k| (foot_of(&first)[k] - foot_of(&swept)[k]).abs() < 1e-5));
    document.parts[leg.0 as usize].severed = true;
    let mesh = mesh_body(&document, &volumes).unwrap();
    moving.mesh = &mesh;
    let (_, stats) =
        walking_anatomy(&document, moving, &mut ground, &contacts, &mut gait, 0.02).unwrap();
    assert_eq!(stats.planted, 0);
    assert_eq!(stats.joints, 0);
    assert!(!gait.feet.contains_key(&PartId(leg.0)));
}
