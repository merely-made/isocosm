// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The producer's two contracts, over the same hand-made documents the query
//! receipts use: no product world is in scope here either.

use cambium_rootstock::{ProducerFrameInfo, ResolvedAppearance};
use isometer_core::{BodyDocument, SpeciesId, VolumeRef};

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
            // The producer writes this from the request either way; a source
            // that declares it names the same number.
            render_scale: request.render_scale,
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
        // A source sizes its scene from the request it is handed, which at a
        // render scale is already the scaled size.
        if [self.scene.width, self.scene.height] != request.size {
            self.scene.resize(request.size[0], request.size[1]);
        }
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
                grade: isometer_lens::Grade::retro(3),
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
        render_scale: 1,
    }
}

/// One request at a leaf size and a pixel grid: `size` is the leaf's, and the
/// scene inside it draws at `size / scale`.
fn scaled<'a>(
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    size: [u32; 2],
    scale: u32,
) -> FrameRequest<'a> {
    FrameRequest {
        device,
        queue,
        size,
        aspect: size[0] as f32 / size[1] as f32,
        color: None,
        needs_frame: false,
        render_scale: scale,
    }
}

/// A texture's RGBA8 bytes, straight off the texture rather than through
/// [`Scene::capture`], so nothing but the pass under test is between the two
/// images a scale receipt compares.
fn read_back(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    size: [u32; 2],
) -> Vec<u8> {
    let unpadded = size[0] * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("scale receipt readback"),
        size: (padded * size[1]) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(size[1]),
            },
        },
        wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("the readback maps");
    let mapped = slice.get_mapped_range().expect("mapped bytes");
    let mut pixels = Vec::with_capacity((unpadded * size[1]) as usize);
    for row in 0..size[1] {
        let start = (row * padded) as usize;
        pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();
    pixels
}

fn pixel(bytes: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * width + x) * 4) as usize;
    bytes[at..at + 4].try_into().expect("four channels")
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

/// I3: scale 1 is the path that shipped. No presentation pass is built, the
/// view the leaf receives is the scene's own, and the bytes are the bytes an
/// unscaled producer draws.
#[test]
fn scale_one_takes_the_unscaled_path_byte_for_byte() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer scale-1 receipt");
        return;
    };
    let mut plain = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    let view = plain
        .render_scene(&request(&device, &queue, false))
        .unwrap()
        .expect("the unscaled frame");
    assert_eq!(
        &view,
        plain.scene.display_view(),
        "an unscaled frame presents the scene's own view"
    );
    assert!(
        plain.display.is_none() && plain.upscale.is_none(),
        "and builds no presentation pass"
    );
    let unscaled = read_back(&device, &queue, plain.scene.display_texture(), SIZE);

    let mut declared = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    declared.set_render_scale(1);
    let view = declared
        .render_scene(&scaled(&device, &queue, SIZE, 1))
        .unwrap()
        .expect("the scale-1 frame");
    assert_eq!(&view, declared.scene.display_view());
    assert!(declared.display.is_none() && declared.upscale.is_none());
    assert_eq!(
        read_back(&device, &queue, declared.scene.display_texture(), SIZE),
        unscaled,
        "scale 1 is byte-identical to no scale at all"
    );
    assert_eq!(declared.signature().render_scale, 1);
}

/// I3's done-condition: a scene requested at scale 4 draws a quarter-sized
/// image, and the leaf receives a texture of its own size in which every 4 by
/// 4 block is one colour — the scene pixel it came from, not a blend of its
/// neighbours.
#[test]
fn a_scale_four_frame_presents_each_scene_pixel_as_one_block() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer render-scale receipt");
        return;
    };
    const LEAF: [u32; 2] = [64, 64];
    const SCALE: u32 = 4;
    let small = [LEAF[0] / SCALE, LEAF[1] / SCALE];

    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer.set_render_scale(SCALE);
    producer
        .render_scene(&scaled(&device, &queue, LEAF, SCALE))
        .unwrap()
        .expect("the scaled frame");

    assert_eq!(
        [producer.scene.width, producer.scene.height],
        small,
        "the scene drew at the scaled size"
    );
    assert_eq!(producer.signature().size, small, "and declared it");
    assert_eq!(producer.signature().render_scale, SCALE);
    let (texture, _, size) = producer.display.as_ref().expect("a presentation target");
    assert_eq!(*size, LEAF, "the leaf is handed its own physical size");

    let scene = read_back(&device, &queue, producer.scene.display_texture(), small);
    let presented = read_back(&device, &queue, texture, LEAF);
    for y in 0..small[1] {
        for x in 0..small[0] {
            let want = pixel(&scene, small[0], x, y);
            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let got = pixel(&presented, LEAF[0], x * SCALE + dx, y * SCALE + dy);
                    assert_eq!(
                        got, want,
                        "block ({x}, {y}) offset ({dx}, {dy}) is not the scene pixel"
                    );
                }
            }
        }
    }
}

/// A scale change with nothing else moved is a real frame, even where the
/// scaled size floors to the same pixel and the signature's size cannot tell
/// the two apart.
#[test]
fn a_changed_render_scale_alone_is_a_new_frame() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer scale-change receipt");
        return;
    };
    const LEAF: [u32; 2] = [4, 4];
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer
        .render_scene(&scaled(&device, &queue, LEAF, 4))
        .unwrap()
        .expect("the first scaled frame");
    assert_eq!(producer.renders(), 1);
    let banked = producer.signature().clone();
    assert_eq!(banked.size, [1, 1]);

    producer
        .render_scene(&scaled(&device, &queue, LEAF, 5))
        .unwrap()
        .expect("a changed scale must produce a frame");
    assert_eq!(producer.renders(), 2);
    assert_eq!(
        producer.signature().size,
        banked.size,
        "the scene size floored to the same pixel"
    );
    assert_eq!(producer.signature().render_scale, 5);
}

/// The skip is the skip at every scale: an unchanged scaled frame re-encodes
/// nothing and advances no generation.
#[test]
fn an_unchanged_scaled_frame_still_skips() {
    let Some((device, queue)) = device() else {
        eprintln!("no adapter; skipping isometer scaled-skip receipt");
        return;
    };
    const LEAF: [u32; 2] = [64, 64];
    let mut producer = SceneProducer::new(Specimen::new(device.clone(), queue.clone()));
    producer
        .render_scene(&scaled(&device, &queue, LEAF, 4))
        .unwrap()
        .expect("the first scaled frame");
    assert_eq!(producer.draws, 1);

    assert!(
        producer
            .render_scene(&scaled(&device, &queue, LEAF, 4))
            .unwrap()
            .is_none(),
        "an unchanged scaled frame must produce nothing"
    );
    assert_eq!(producer.renders(), 1, "a skip advances no generation");
    assert_eq!(producer.draws, 1, "and reaches no encode");
}
