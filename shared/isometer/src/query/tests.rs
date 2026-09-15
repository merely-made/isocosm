// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The query surface against real encoded frames, with no product world in
//! scope: hand-made body documents, declared-extent volumes, no terrain.
//!
//! The terrain occlusion, capsule-fallback and cutaway arms of this surface
//! need a host's own policy to set up and stay in Mesocosm's
//! `section/query_tests.rs`.

use isometer_core::{BodyDocument, PartId, SpeciesId, VolumeRef};

use super::*;
use crate::bodies::Pose;
use crate::scene::{SceneFrame, SceneHost};
use crate::volumes::DeclaredExtentVolumes;

const SIZE: u32 = 33;
/// The forward `CameraMode::Side` produces, and Paredros's level default.
const SIDE: [f32; 3] = [0.0, 0.0, -1.0];

struct Host;
impl SceneHost for Host {}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

fn document(tag: u8) -> BodyDocument {
    BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(tag), 1_000, [1; 3])
}

fn camera() -> SlabCamera {
    SlabCamera {
        centre: [0.0; 3],
        forward: SIDE,
        half_height: 4.0,
        aspect: 1.0,
        depth: 32.0,
        cutaway: None,
    }
}

fn body<'a>(subject: SubjectKey, document: &'a BodyDocument, position: [f32; 3]) -> SceneBody<'a> {
    SceneBody {
        subject,
        document,
        pose: Pose {
            position,
            yaw_radians: 0.0,
        },
        scale: 1.0,
        grounded: false,
        tint: [1.0; 3],
        materials: &[],
        always_visible: false,
    }
}

/// One frame with no terrain: the isolated-preview branch, which still
/// completes a query receipt.
fn render(scene: &mut Scene, bodies: &[SceneBody<'_>], volumes: &DeclaredExtentVolumes) {
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera: camera(),
                bodies,
                volumes: SceneVolumes::DeclaredSolid(volumes),
                terrain: None,
                dirty: &[],
                grade: mesocosm_lens::Grade::retro(3),
                terrain_appearance: None,
                body_budget: 8,
                capsules: None,
            },
            &mut Host,
        )
        .expect("a bodies-only frame encodes");
    scene.queue.submit([encoder.finish()]);
}

#[test]
fn a_pick_addresses_the_nearer_body_and_expires_with_its_frame() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer query receipt");
        return;
    };
    let (near, far) = (document(70), document(71));
    let volumes = DeclaredExtentVolumes::from_documents([&near, &far], 9);
    // The camera looks down -z, so the larger z is the nearer surface.
    let bodies = [
        body(SubjectKey(4), &near, [0.0, 0.0, 3.0]),
        body(SubjectKey(9), &far, [0.0, 0.0, -3.0]),
    ];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();

    assert_eq!(scene.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    render(&mut scene, &bodies, &volumes);

    let hit = scene.pick_pixel([SIZE / 2, SIZE / 2]).unwrap().unwrap();
    assert_eq!(hit.address.subject, SubjectKey(4));
    assert!(!hit.tied);
    assert!((hit.point[2] - 4.0).abs() < 1e-4, "the near face at z=4");
    assert_eq!(scene.pick_ndc([0.0; 2]).unwrap(), Some(hit));
    assert!(scene.validate_pick(hit, &bodies[0], SceneVolumes::DeclaredSolid(&volumes)));
    // The address belongs to its own subject and to no other.
    assert!(!scene.validate_pick(hit, &bodies[1], SceneVolumes::DeclaredSolid(&volumes)));

    assert_eq!(
        scene.pick_ndc([f32::NAN, 0.0]),
        Err(BodyPickError::InvalidCoordinates)
    );
    assert_eq!(
        scene.pick_pixel([SIZE, 0]),
        Err(BodyPickError::InvalidCoordinates)
    );

    // A second encode mints a new generation, so the old receipt is spent
    // even though the address it carries is still current.
    render(&mut scene, &bodies, &volumes);
    let current = scene.pick_ndc([0.0; 2]).unwrap().unwrap();
    assert_eq!(current.address, hit.address);
    assert_ne!(current.frame, hit.frame);
    assert!(!scene.validate_pick(hit, &bodies[0], SceneVolumes::DeclaredSolid(&volumes)));

    // Every visual change a host owns rather than a frame drops the receipt.
    scene.resize(SIZE, SIZE + 2);
    assert_eq!(scene.pick_ndc([0.0; 2]), Err(BodyPickError::NotReady));
    assert_eq!(scene.select_part(SubjectKey(4), None, false), None);
}

/// Plan §7 condition 4: resolving the projector's identity question by
/// narrowing a subject to 32 bits would be caught here rather than in a
/// product.
#[test]
fn a_subject_key_beyond_u32_survives_a_pick_and_its_queries() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping wide-subject query receipt");
        return;
    };
    let wide = SubjectKey(u64::from(u32::MAX) + 7);
    let document = document(72);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let bodies = [body(wide, &document, [0.0; 3])];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &bodies, &volumes);

    let hit = scene.pick_pixel([SIZE / 2, SIZE / 2]).unwrap().unwrap();
    assert_eq!(hit.address.subject, wide);
    assert_eq!(hit.address.part, PartId(0));
    assert_eq!(
        scene.select_part(wide, None, false).map(|a| a.subject),
        Some(wide)
    );
    assert_eq!(
        scene.select_part(SubjectKey(hit.address.subject.0 as u32 as u64), None, false),
        None
    );
    let anchors = scene
        .glyph_anchors(
            &bodies[0],
            SceneVolumes::DeclaredSolid(&volumes),
            Some(hit.address),
        )
        .unwrap();
    assert_eq!(anchors[0].selection.subject, wide);
}

/// The camera pair and the pick agree about which pixel a world point is on,
/// and `part_bounds` reports the drawn geometry rather than the whole body.
#[test]
fn the_camera_pixel_of_a_part_is_the_pixel_that_picks_it() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping camera/pick agreement receipt");
        return;
    };
    let document = document(73);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let bodies = [body(SubjectKey(2), &document, [2.0, 1.0, 0.0])];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &bodies, &volumes);

    let (min, max) = scene.part_bounds(SubjectKey(2), PartId(0)).unwrap();
    assert_eq!(min, [1.0, 0.0, -1.0]);
    assert_eq!(max, [3.0, 2.0, 1.0]);
    assert_eq!(scene.part_bounds(SubjectKey(2), PartId(3)), None);
    assert_eq!(scene.part_bounds(SubjectKey(5), PartId(0)), None);

    let centre = [0, 1, 2].map(|axis| (min[axis] + max[axis]) * 0.5);
    let pixel = camera().pixel_of(centre, [SIZE, SIZE]).unwrap();
    let hit = scene.pick_pixel(pixel).unwrap().unwrap();
    assert_eq!(hit.address.subject, SubjectKey(2));
    // A world point outside the frame has no pixel and no hit.
    assert_eq!(camera().pixel_of([40.0, 0.0, 0.0], [SIZE, SIZE]), None);
}

/// The focus emphasis a host sets is presentation, and it spends the receipt
/// because the next draw paints different pixels.
#[test]
fn setting_focus_spends_the_query_receipt_only_when_it_changes() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping focus receipt");
        return;
    };
    let document = document(74);
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let bodies = [body(SubjectKey(1), &document, [0.0; 3])];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    render(&mut scene, &bodies, &volumes);
    let generation = scene.query_generation().unwrap();

    scene.set_body_focus(None, None);
    assert_eq!(scene.query_generation(), Some(generation));

    let address = scene.select_part(SubjectKey(1), None, false).unwrap();
    scene.set_body_focus(Some(SubjectKey(1)), Some(address));
    assert_eq!(scene.query_generation(), None);
}
