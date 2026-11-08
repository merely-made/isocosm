// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::hybrid::{HybridPolicy, HybridSettings, HybridStats};
use crate::{DeclaredExtentVolumes, Pick, Pose, SceneBody, SceneVolumes, SlabCamera, SubjectKey};
use isometer_core::{BodyDocument, VolumeRef};
use std::num::NonZeroU64;

const VIEW: u32 = 65;
const SLOT_BYTES: u64 = 32 * 32 * 8;

mod light;

struct Host {
    settings: HybridSettings,
    examined: Option<SubjectKey>,
    /// A subject whose world holds only a sprite.
    held_sprite: Option<SubjectKey>,
    light: Option<crate::BodyLight>,
}

impl SceneHost for Host {
    fn hybrid_settings(&self) -> Option<HybridSettings> {
        Some(self.settings)
    }
    fn hybrid_examined(&self, subject: SubjectKey) -> bool {
        self.examined == Some(subject)
    }
    fn hybrid_held(&self, subject: SubjectKey) -> crate::hybrid::BodyDetail {
        if self.held_sprite == Some(subject) {
            crate::hybrid::BodyDetail::Sprite
        } else {
            crate::hybrid::BodyDetail::Live
        }
    }
    fn body_light(&self) -> Option<crate::BodyLight> {
        self.light
    }
}

fn host() -> Host {
    let policy = HybridPolicy::new(
        [[4.0, 6.0], [10.0, 14.0], [24.0, 32.0]],
        [60, 60, 60].map(|period| NonZeroU64::new(period).unwrap()),
    )
    .unwrap();
    Host {
        settings: HybridSettings::new(policy, 32, 2 * SLOT_BYTES, 1).with_transition_frames(4),
        examined: None,
        held_sprite: None,
        light: None,
    }
}

fn camera(half_height: f32) -> SlabCamera {
    SlabCamera::new([0.0; 3], [0.0, 0.0, 1.0], half_height, 1.0, 64.0).unwrap()
}

fn body(id: u64, x: f32, document: &BodyDocument) -> SceneBody<'_> {
    SceneBody {
        subject: SubjectKey(id),
        document,
        pose: Pose {
            position: [x, 0.0, 0.0],
            yaw_radians: 0.0,
        },
        scale: 1.0,
        grounded: false,
        tint: [1.0; 3],
        materials: &[],
        always_visible: false,
    }
}

fn record(
    scene: &mut Scene,
    camera: SlabCamera,
    bodies: &[SceneBody<'_>],
    volumes: &DeclaredExtentVolumes,
    host: &mut Host,
) -> wgpu::CommandEncoder {
    let mut encoder = scene.device.create_command_encoder(&Default::default());
    scene
        .render(
            &mut encoder,
            SceneFrame {
                camera,
                bodies,
                volumes: SceneVolumes::DeclaredSolid(volumes),
                terrain: None,
                dirty: &[],
                grade: isometer_lens::Grade::clay(),
                terrain_appearance: None,
                body_budget: 16,
                capsules: None,
            },
            host,
        )
        .unwrap();
    encoder
}

fn render(
    scene: &mut Scene,
    camera: SlabCamera,
    bodies: &[SceneBody<'_>],
    volumes: &DeclaredExtentVolumes,
    host: &mut Host,
) -> HybridStats {
    let encoder = record(scene, camera, bodies, volumes, host);
    scene.queue.submit([encoder.finish()]);
    scene.hybrid_stats()
}

#[test]
fn an_abandoned_encoder_does_not_leave_reusable_images_or_queries() {
    let (device, queue) =
        crate::test_gpu::device().expect("GPU is required for the sprite receipt");
    let doc = BodyDocument::new(VolumeRef::from_tag(81), [2; 3]);
    let volumes = DeclaredExtentVolumes::from_documents([&doc], 1);
    let bodies = [body(1, 0.0, &doc)];
    let mut scene = Scene::new(device, queue, VIEW, VIEW).unwrap();
    let mut host = host();
    let camera = camera(6.0);
    drop(record(&mut scene, camera, &bodies, &volumes, &mut host));
    assert_eq!(scene.hybrid_stats().refreshes, 1);
    scene.abandon_frame();
    assert!(scene.presented_camera().is_none());
    scene
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    let retry = render(&mut scene, camera, &bodies, &volumes, &mut host);
    assert_eq!(
        (retry.hits, retry.refreshes, retry.sprite_bodies),
        (0, 1, 1)
    );
}

#[test]
fn resident_budget_caps_synchronized_refresh_and_keeps_current_pick() {
    let (device, queue) =
        crate::test_gpu::device().expect("GPU is required for the sprite receipt");
    let doc = BodyDocument::new(VolumeRef::from_tag(81), [2; 3]);
    let volumes = DeclaredExtentVolumes::from_documents([&doc], 1);
    let a = body(1, -2.0, &doc);
    let b = body(2, 2.0, &doc);
    let mut scene = Scene::new(device, queue, VIEW, VIEW).unwrap();
    let mut host = host();
    let camera = camera(6.0);

    let first = render(&mut scene, camera, &[a, b], &volumes, &mut host);
    assert_eq!((first.retained_bytes, first.capacity), (2 * SLOT_BYTES, 2));
    assert_eq!(
        (first.refreshes, first.sprite_bodies, first.live_fallbacks),
        (1, 1, 1)
    );
    let second = render(&mut scene, camera, &[a, b], &volumes, &mut host);
    assert_eq!(
        (second.hits, second.refreshes, second.resident_slots),
        (1, 1, 2)
    );

    let moved = body(1, -3.5, &doc);
    let held = render(&mut scene, camera, &[moved, b], &volumes, &mut host);
    assert_eq!((held.hits, held.refreshes), (2, 0));
    let (low, high) = scene.bodies().part_bounds(moved.subject, doc.root).unwrap();
    let ndc = camera
        .ndc_of(std::array::from_fn(|i| (low[i] + high[i]) * 0.5))
        .unwrap();
    assert!(
        matches!(scene.pick(ndc).unwrap(), Some(Pick::Body(hit)) if hit.address.subject == moved.subject)
    );

    fn tinted(b: SceneBody<'_>) -> SceneBody<'_> {
        SceneBody {
            tint: [0.7, 0.9, 1.0],
            ..b
        }
    }
    let changed = render(
        &mut scene,
        camera,
        &[tinted(moved), tinted(b)],
        &volumes,
        &mut host,
    );
    assert_eq!(
        (changed.misses, changed.refreshes, changed.live_fallbacks),
        (2, 1, 1)
    );
    render(
        &mut scene,
        camera,
        &[tinted(moved), tinted(b)],
        &volumes,
        &mut host,
    );
    let c = body(3, 0.0, &doc);
    let evicted = render(&mut scene, camera, &[c], &volumes, &mut host);
    assert_eq!(
        (
            evicted.evictions,
            evicted.resident_slots,
            evicted.sprite_bodies
        ),
        (1, 2, 1)
    );

    let coverage = |scene: &Scene| {
        scene
            .capture(|_, _, _| {})
            .unwrap()
            .2
            .chunks_exact(4)
            .filter(|p| p[..3] != [0, 0, 0])
            .count()
    };
    let old_coverage = coverage(&scene);
    host.settings.resolution = 24;
    host.settings.max_resident_bytes = 2 * 24 * 24 * 8;
    let switching = render(&mut scene, camera, &[c], &volumes, &mut host);
    assert!(switching.retiring_bytes == 0 || switching.retained_bytes == 0);
    scene
        .device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    let smaller = render(&mut scene, camera, &[c], &volumes, &mut host);
    assert_eq!(smaller.retained_bytes, 2 * 24 * 24 * 8);
    // Let the ordered switch complete before comparing the images.
    for _ in 0..4 {
        render(&mut scene, camera, &[c], &volumes, &mut host);
    }
    let smaller_coverage = coverage(&scene);
    assert!(old_coverage > 0 && smaller_coverage > 0);
    eprintln!(
        "32px atlas: {old_coverage} covered pixels, {} bytes; 24px atlas: {smaller_coverage} covered pixels, {} bytes",
        evicted.retained_bytes, smaller.retained_bytes
    );
}

#[test]
fn zoom_reverses_the_transition_and_examination_promotes_immediately() {
    let (device, queue) =
        crate::test_gpu::device().expect("GPU is required for the sprite receipt");
    let doc = BodyDocument::new(VolumeRef::from_tag(81), [2; 3]);
    let volumes = DeclaredExtentVolumes::from_documents([&doc], 1);
    let body = body(1, 0.0, &doc);
    let mut scene = Scene::new(device, queue, VIEW, VIEW).unwrap();
    let mut host = host();
    let far = camera(6.0);
    let near = camera(1.5);
    assert_eq!(
        render(&mut scene, near, &[body], &volumes, &mut host).sprite_bodies,
        0
    );
    let demote = render(&mut scene, far, &[body], &volumes, &mut host);
    assert_eq!((demote.transition_bodies, demote.refreshes), (1, 1));
    assert!(scene.hybrid_needs_frame());
    assert_eq!(
        render(&mut scene, near, &[body], &volumes, &mut host).sprite_bodies,
        0
    );
    assert!(!scene.hybrid_needs_frame());

    for _ in 0..4 {
        let stats = render(&mut scene, far, &[body], &volumes, &mut host);
        assert_eq!(stats.refreshes, 0);
    }
    assert!(!scene.hybrid_needs_frame());
    host.examined = Some(body.subject);
    assert_eq!(
        render(&mut scene, far, &[body], &volumes, &mut host).sprite_bodies,
        0
    );
    assert!(!scene.hybrid_needs_frame());
}
