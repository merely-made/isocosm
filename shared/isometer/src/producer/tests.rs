// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The producer's two contracts, over the same hand-made documents the query
//! receipts use: no product world is in scope here either.

use cambium_rootstock::{ProducerFrameInfo, ResolvedAppearance};
use mesocosm_core::{BodyDocument, SpeciesId, VolumeRef};

use super::*;
use crate::bodies::{SceneVolumes, SubjectKey};
use crate::camera::Cutaway;
use crate::scene::{Scene, SceneFrame, SceneHost};
use crate::volumes::DeclaredExtentVolumes;

const SIZE: [u32; 2] = [33, 33];

struct Host;
impl SceneHost for Host {}

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&Default::default())).ok()?;
    pollster::block_on(adapter.request_device(&Default::default())).ok()
}

/// The shape a product's source takes: it owns the scene, builds the frame,
/// and declares its inputs without drawing them.
struct Specimen {
    scene: Scene,
    documents: Vec<BodyDocument>,
    volumes: DeclaredExtentVolumes,
    poses: Vec<Pose>,
    /// Bumped by a host that changed an anatomy; the pose is separate.
    revision: u64,
    half_height: f32,
    /// A host's own refusal, as the bench's non-opaque tint is.
    refuse: Option<String>,
    /// Encodes this source actually ran, against the producer's own count.
    draws: u64,
}

impl Specimen {
    fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let documents: Vec<_> = [80u8, 81]
            .into_iter()
            .map(|tag| BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(tag), 1_000, [1; 3]))
            .collect();
        let volumes = DeclaredExtentVolumes::from_documents(documents.iter(), 9);
        Self {
            scene: Scene::new(device, queue, SIZE[0], SIZE[1]).expect("a scene"),
            documents,
            volumes,
            poses: vec![
                Pose {
                    position: [-2.0, 0.0, 0.0],
                    yaw_radians: 0.0,
                },
                Pose {
                    position: [2.0, 0.0, 0.0],
                    yaw_radians: 0.0,
                },
            ],
            revision: 1,
            half_height: 4.0,
            refuse: None,
            draws: 0,
        }
    }

    fn camera(&self, aspect: f32) -> SlabCamera {
        SlabCamera {
            centre: [0.0; 3],
            forward: [0.0, 0.0, -1.0],
            half_height: self.half_height,
            aspect,
            depth: 32.0,
            cutaway: None,
        }
    }

    fn bodies(&self) -> Vec<SceneBody<'_>> {
        scene_bodies(&self.documents, &self.poses)
    }
}

fn scene_bodies<'a>(documents: &'a [BodyDocument], poses: &'a [Pose]) -> Vec<SceneBody<'a>> {
    documents
        .iter()
        .zip(poses)
        .enumerate()
        .map(|(index, (document, pose))| SceneBody {
            subject: SubjectKey(index as u64 + 1),
            document,
            pose: *pose,
            scale: 1.0,
            grounded: false,
            tint: [1.0; 3],
            materials: &[],
            always_visible: false,
        })
        .collect()
}

impl SceneSource for Specimen {
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        if let Some(why) = &self.refuse {
            return Err(why.clone());
        }
        let revision = self.revision;
        Ok(SceneSignature {
            size: request.size,
            camera: Some(self.camera(request.aspect)),
            terrain_revision: None,
            bodies: self
                .bodies()
                .iter()
                .map(|body| BodySignature::of(body, revision))
                .collect(),
            host: Vec::new(),
        })
    }

    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        let camera = self.camera(request.aspect);
        let Self {
            scene,
            documents,
            poses,
            volumes,
            draws,
            ..
        } = self;
        let bodies = scene_bodies(documents, poses);
        let mut encoder = request.device.create_command_encoder(&Default::default());
        scene.render(
            &mut encoder,
            SceneFrame {
                camera,
                bodies: &bodies,
                volumes: SceneVolumes::DeclaredSolid(volumes),
                terrain: None,
                dirty: &[],
                grade: mesocosm_lens::Grade::retro(3),
                terrain_appearance: None,
                body_budget: 8,
                capsules: None,
            },
            &mut Host,
        )?;
        request.queue.submit([encoder.finish()]);
        *draws += 1;
        Ok(Some(scene.display_view().clone()))
    }

    fn presented_camera(&self) -> Option<SlabCamera> {
        self.scene.presented_camera()
    }

    fn cached_bodies(&self) -> usize {
        self.scene.cached_bodies()
    }
}

fn request<'a>(
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    needs_frame: bool,
) -> FrameRequest<'a> {
    FrameRequest {
        device,
        queue,
        size: SIZE,
        aspect: 1.0,
        color: None,
        needs_frame,
    }
}

/// Plan §7 condition 5: the skip signature covers the camera, the size and
/// every body's identity, revision and pose, and a frame that repeats them
/// costs no encode.
#[test]
fn an_unchanged_frame_neither_encodes_nor_advances_the_generation() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer producer receipt");
        return;
    };
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));

    assert!(
        producer
            .render_scene(&request(&device, &queue, false))
            .unwrap()
            .is_some(),
        "the first frame"
    );
    assert_eq!(producer.renders(), 1);
    assert_eq!(producer.draws, 1);
    let banked = producer.signature().clone();
    let camera = producer.presented_camera().expect("a presented camera");

    assert!(
        producer
            .render_scene(&request(&device, &queue, false))
            .unwrap()
            .is_none(),
        "an unchanged frame must produce nothing"
    );
    assert_eq!(producer.renders(), 1, "a skip advances no generation");
    assert_eq!(producer.draws, 1, "a skip reaches no encode");
    assert_eq!(producer.signature(), &banked, "and reports the same inputs");
    assert_eq!(producer.presented_camera(), Some(camera));

    assert!(
        producer
            .render_scene(&request(&device, &queue, true))
            .unwrap()
            .is_some(),
        "needs_frame must force a frame"
    );
    assert_eq!(producer.renders(), 2);
    assert_eq!(producer.signature(), &banked, "the same inputs, redrawn");

    // A changed pose is a changed picture with needs_frame false.
    producer.source_mut().poses[1].position[1] = 1.5;
    assert!(
        producer
            .render_scene(&request(&device, &queue, false))
            .unwrap()
            .is_some(),
        "a moved pose must produce a frame"
    );
    assert_eq!(producer.renders(), 3);
    assert_eq!(producer.draws, 3);
    assert_ne!(producer.signature(), &banked);
    assert_eq!(producer.signature().bodies[1].pose.position[1], 1.5);

    // And so is a changed anatomy revision at an unchanged pose, and a camera
    // the host reframed.
    for change in [
        (|s: &mut Specimen| s.revision += 1) as fn(&mut Specimen),
        |s: &mut Specimen| s.half_height = 6.0,
    ] {
        let before = producer.renders();
        change(producer.source_mut());
        assert!(
            producer
                .render_scene(&request(&device, &queue, false))
                .unwrap()
                .is_some()
        );
        assert_eq!(producer.renders(), before + 1);
    }
    assert_eq!(producer.signature().bodies[0].revision, 2);
    assert_eq!(
        producer.signature().camera.unwrap().half_height,
        6.0,
        "the camera is in the signature"
    );
    assert_eq!(
        producer.signature().camera.unwrap().cutaway,
        None::<Cutaway>
    );

    // A suspension keeps the retained geometry and spends the banked frame.
    let cached = producer.cached_bodies();
    assert!(cached > 0, "geometry was cached");
    producer.suspend();
    assert_eq!(
        producer.cached_bodies(),
        cached,
        "suspension keeps the retained geometry"
    );
    assert_eq!(producer.signature(), &SceneSignature::default());
    let renders = producer.renders();
    assert!(
        producer
            .render_scene(&request(&device, &queue, false))
            .unwrap()
            .is_some(),
        "a resumed producer must draw"
    );
    assert_eq!(producer.renders(), renders + 1);
}

/// Plan §7 condition 5: the output contract is fixed, and a skip leaves the
/// document's last image in place rather than an empty slot.
#[test]
fn the_produced_texture_is_straight_alpha_encoded_srgb() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer output contract receipt");
        return;
    };
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    let frame = ProducerFrameInfo {
        logical_size: (SIZE[0] as f32, SIZE[1] as f32),
        physical_size: SIZE,
        layout_scale: 1.0,
        needs_frame: false,
        appearance: ResolvedAppearance::default(),
    };
    let cx = ProducerContext {
        device: &device,
        queue: &queue,
        frame: &frame,
    };

    let produced = producer.render(&cx).expect("the first frame");
    assert_eq!(produced.alpha, SCENE_ALPHA);
    assert_eq!(produced.encoding, SCENE_ENCODING);
    assert_eq!(produced.alpha, SourceAlpha::Straight);
    assert_eq!(produced.encoding, SourceEncoding::Srgb);
    assert_eq!(produced.generation, producer.renders());
    assert!(producer.last_error().is_none());

    assert!(
        producer.render(&cx).is_none(),
        "the unchanged frame keeps the document's last image"
    );
    assert_eq!(producer.renders(), 1);
}

/// A source that refuses the request leaves the banked frame alone, so the
/// inputs it already drew still skip once the refusal clears.
#[test]
fn a_refused_request_neither_banks_nor_spends_the_last_frame() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer refusal receipt");
        return;
    };
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer
        .render_scene(&request(&device, &queue, false))
        .unwrap()
        .expect("the first frame");
    let banked = producer.signature().clone();

    producer.source_mut().refuse = Some("the leaf is not drawable".into());
    assert_eq!(
        producer
            .render_scene(&request(&device, &queue, true))
            .unwrap_err(),
        "the leaf is not drawable"
    );
    assert_eq!(producer.renders(), 1, "a refusal draws nothing");
    assert_eq!(producer.draws, 1);
    assert_eq!(producer.signature(), &banked, "and forgets nothing");

    // Cleared, with nothing else moved, the banked frame still skips.
    producer.source_mut().refuse = None;
    assert!(
        producer
            .render_scene(&request(&device, &queue, false))
            .unwrap()
            .is_none()
    );
    assert_eq!(producer.renders(), 1);
}
