// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isometer::core::BodyDocument;
use isometer::core as isometer_core;
use isometer::render::{self as isometer_render, ClipSlab};
use isometer::{
    BodyLayer, DeclaredExtentVolumes, Pose, SceneBody, SceneVolumes, SlabCamera, SubjectKey,
};
use std::collections::BTreeMap;
#[test]
fn posed_surface_replaces_box_picking_and_expires_with_its_native_revision() {
    let gpu =
        isometer_render::Renderer::headless(16, 16).expect("surface query gate requires a GPU");
    let mut layer = BodyLayer::new(gpu.device(), 16, 16, 16.0);
    let document = BodyDocument::new(isometer_core::VolumeRef::from_tag(1), [1; 3]);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 1);
    let subject = SubjectKey(9);
    let body = SceneBody {
        subject,
        document: &document,
        pose: Pose::default(),
        scale: 1.,
        grounded: false,
        tint: [1.; 3],
        materials: &[],
        always_visible: true,
    };
    let camera = SlabCamera::new([0.; 3], [0., 0., -1.], 4., 1., 16.).unwrap();
    layer.prepare(
        &[body],
        SceneVolumes::DeclaredSolid(&volumes),
        camera.window(),
        |_, _| panic!("unexpected fallback"),
    );
    let clip = ClipSlab::new([0., 0., 1.], -10., 10.);
    assert!(
        layer
            .pick([0.98, 0.98, -5.], [0., 0., 1.], 10., clip)
            .unwrap()
            .is_some()
    );
    let live = layer.render_bodies().next().unwrap().1;
    let (surface, _) = isometer_render::implicit::from_anatomy(&document, live, |_| None).unwrap();
    assert!(
        layer
            .set_surfaces(BTreeMap::from([(SubjectKey(99), surface.clone())]))
            .is_err()
    );
    layer
        .set_surfaces(BTreeMap::from([(subject, surface)]))
        .unwrap();
    assert!(
        layer
            .pick([0.98, 0.98, -5.], [0., 0., 1.], 10., clip)
            .unwrap()
            .is_none()
    );
    let (address, _) = layer
        .pick([0., 0., -5.], [0., 0., 1.], 10., clip)
        .unwrap()
        .unwrap();
    assert_eq!(address.subject, subject);
    assert!(layer.validate_address(address, &body, SceneVolumes::DeclaredSolid(&volumes)));
    let mut changed = document.clone();
    changed
        .attach(
            isometer_core::VolumeRef::from_tag(1),
            [1; 3],
            isometer_core::Attachment {
                parent: changed.root,
                offset: [2, 0, 0],
                yaw: isometer_core::Yaw::Zero,
            },
            None,
        )
        .unwrap();
    let changed_body = SceneBody {
        document: &changed,
        ..body
    };
    assert!(!layer.validate_address(
        address,
        &changed_body,
        SceneVolumes::DeclaredSolid(&volumes)
    ));
    layer.clear_inspection();
    assert!(layer.part_bounds(subject, address.part).is_none());
    assert!(
        layer
            .pick([0., 0., -5.], [0., 0., 1.], 10., clip)
            .unwrap()
            .is_none()
    );
}
