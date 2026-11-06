// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Picking under the perspective projection (cases 1131, 1140): a real
//! encoded frame, two bodies at different places in view and one behind the
//! eye; the pixel each rasters to picks it, and the one behind is never drawn.

use isometer_core::{BodyDocument, VolumeRef};

use super::*;
use crate::bodies::Pose;
use crate::scene::{SceneFrame, SceneHost};
use crate::volumes::DeclaredExtentVolumes;

const SIZE: u32 = 96;

struct Host;
impl SceneHost for Host {}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    crate::test_gpu::device()
}

fn camera() -> SlabCamera {
    SlabCamera::first_person([0.0, 0.5, 10.0], [0.0, 0.0, -1.0], 0.0, 1.0, 1.0, 0.05, 40.0).unwrap()
}

fn body<'a>(subject: SubjectKey, document: &'a BodyDocument, position: [f32; 3]) -> SceneBody<'a> {
    SceneBody {
        subject,
        document,
        pose: Pose { position, yaw_radians: 0.0 },
        scale: 1.0,
        grounded: false,
        tint: [1.0; 3],
        materials: &[],
        always_visible: false,
    }
}

#[test]
fn a_pick_under_perspective_returns_the_body_under_the_cursor() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer perspective pick");
        return;
    };
    let docs = [70, 71, 72].map(|tag| BodyDocument::new(VolumeRef::from_tag(tag), [1; 3]));
    let volumes = DeclaredExtentVolumes::from_documents(docs.iter(), 9);
    let at = [[-2.5, 0.5, 2.0], [3.0, 1.0, -6.0], [0.0, 0.5, 14.0]];
    let bodies = [
        body(SubjectKey(1), &docs[0], at[0]),
        body(SubjectKey(2), &docs[1], at[1]),
        // Behind the eye: culled, so it can be neither drawn nor picked.
        body(SubjectKey(3), &docs[2], at[2]),
    ];
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene.render(&mut encoder, SceneFrame {
        camera: camera(),
        bodies: &bodies,
        volumes: SceneVolumes::DeclaredSolid(&volumes),
        terrain: None,
        dirty: &[],
        grade: isometer_lens::Grade::retro(3),
        terrain_appearance: None,
        body_budget: 8,
        capsules: None,
    }, &mut Host).expect("a perspective frame encodes");
    scene.queue.submit([encoder.finish()]);

    for (subject, point) in [(SubjectKey(1), at[0]), (SubjectKey(2), at[1])] {
        let pixel = camera().pixel_of(point, [SIZE; 2]).expect("in view");
        let hit = scene.pick_pixel(pixel).unwrap().unwrap_or_else(|| panic!("{subject:?} at {pixel:?}"));
        assert_eq!(hit.address.subject, subject, "the body under {pixel:?}");
    }
    assert_eq!(camera().pixel_of(at[2], [SIZE; 2]), None, "behind the eye has no pixel");
    // Away from both bodies there is nothing to pick.
    assert_eq!(scene.pick_pixel([SIZE / 2, 2]).unwrap(), None);
}

/// The eye sits on top of the named part as the draw poses it, and on top
/// of the root when no part is named or the named one draws nothing
/// (case 1143).
#[test]
fn the_eye_sits_on_the_sensing_part_or_the_roots_top() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer eye");
        return;
    };
    use isometer_core::{Attachment, PartId, Yaw};
    let mut document = BodyDocument::new(VolumeRef::from_tag(80), [2, 2, 2]);
    let head = document
        .attach(VolumeRef::from_tag(80), [1, 1, 1], Attachment { parent: PartId(0), offset: [0, 3, 1], yaw: Yaw::Zero }, None)
        .unwrap();
    let volumes = DeclaredExtentVolumes::from_documents([&document], 9);
    let body = body(SubjectKey(5), &document, [4.0, 1.0, -2.0]);
    let mut scene = Scene::new(device, queue, SIZE, SIZE).unwrap();
    let solid = SceneVolumes::DeclaredSolid(&volumes);
    let (head_box, root_box) = (
        scene.bodies_mut().posed_part_bounds(&body, solid, head).unwrap().unwrap(),
        scene.bodies_mut().posed_part_bounds(&body, solid, PartId(0)).unwrap().unwrap(),
    );
    let top = |(min, max): ([f32; 3], [f32; 3])| [(min[0] + max[0]) * 0.5, max[1], (min[2] + max[2]) * 0.5];
    assert!(top(head_box)[1] > top(root_box)[1], "the head stands above the root");
    assert_eq!(scene.eye(&body, solid, Some(head)).unwrap(), Some(top(head_box)));
    assert_eq!(scene.eye(&body, solid, None).unwrap(), Some(top(root_box)));
    assert_eq!(scene.eye(&body, solid, Some(PartId(9))).unwrap(), Some(top(root_box)), "an absent part falls back");
    assert_eq!(crate::bodies::eye_of(&body, solid, Some(head)).unwrap(), Some(top(head_box)), "without a scene too");
}
