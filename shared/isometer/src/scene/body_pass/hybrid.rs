// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroU64;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use isometer_render::{
    live_body::body_bounds,
    sprites::{SpriteAtlas, SpriteDraw, SpriteProjection},
};

use super::super::residency::body_images::{self, Slots};
use crate::hybrid::{
    BodyChoice, BodyDetail, HybridSettings, HybridStats, SpriteCache, SpritePlacement,
    SpriteRevisions, SpriteSource, projected_size,
};
use crate::{BodyLayer, SceneHost, SlabCamera, SubjectKey, bodies::HybridBody};

#[derive(Default)]
pub(in crate::scene) struct HybridFrame {
    frame: u64,
    previous: BTreeMap<SubjectKey, BodyChoice>,
    coverage: BTreeMap<SubjectKey, u32>,
    resident: Option<Resident>,
    retiring: Option<(u64, Arc<AtomicBool>)>,
    pub(in crate::scene) stats: HybridStats,
    pub(in crate::scene) needs_frame: bool,
}

struct Resident {
    resolution: u32,
    capacity: u32,
    atlas: SpriteAtlas,
    images: SpriteCache<Image>,
    slots: Slots,
}

#[derive(Clone, Copy)]
struct Image {
    slot: u32,
    projection: SpriteProjection,
}

struct Request<'a> {
    body: HybridBody<'a>,
    revisions: SpriteRevisions,
    period: NonZeroU64,
    coverage: u32,
}

impl HybridFrame {
    fn update_fence(&mut self) {
        if self
            .retiring
            .as_ref()
            .is_some_and(|(_, done)| done.load(Ordering::Acquire))
        {
            self.retiring = None;
        }
        self.stats.retiring_bytes = self.retiring.as_ref().map_or(0, |(bytes, _)| *bytes);
    }

    fn retire(&mut self, queue: &wgpu::Queue) {
        if let Some(old) = self.resident.take() {
            let done = Arc::new(AtomicBool::new(false));
            let signal = done.clone();
            queue.on_submitted_work_done(move || signal.store(true, Ordering::Release));
            self.retiring = Some((old.atlas.retained_bytes(), done));
        }
        self.update_fence();
    }

    pub(in crate::scene) fn inactive(&mut self, queue: &wgpu::Queue) {
        self.previous.clear();
        self.coverage.clear();
        self.needs_frame = false;
        self.stats = HybridStats::default();
        self.retire(queue);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        layer: &BodyLayer,
        camera: SlabCamera,
        size: [u32; 2],
        host: &dyn SceneHost,
        settings: HybridSettings,
        light: Option<isometer_render::live_body::BodyLight>,
    ) -> BTreeMap<SubjectKey, u32> {
        self.frame += 1;
        self.stats = HybridStats::default();
        self.needs_frame = false;
        self.update_fence();
        let capacity = body_images::capacity(settings, &device.limits());
        if self.resident.as_ref().is_some_and(|old| {
            old.resolution != settings.resolution || Some(old.capacity) != capacity
        }) {
            self.retire(queue);
        }

        let axes = camera.basis();
        let mut facing = DefaultHasher::new();
        axes.map(|axis| axis.map(f32::to_bits)).hash(&mut facing);
        let facing = facing.finish();
        let lighting = light.map_or(0, |light| light.stamp());
        self.stats.light_stamp = lighting;
        let previous = std::mem::take(&mut self.previous);
        let last_coverage = std::mem::take(&mut self.coverage);
        let step = 16_u32.div_ceil(settings.transition_frames.clamp(1, 16));
        let mut requests = Vec::new();
        for body in layer.hybrid_bodies() {
            let Some(pixels) = projected_size(camera, body.bounds, size) else {
                continue;
            };
            let prior = previous.get(&body.subject).copied();
            let examined = body.examined || host.hybrid_examined(body.subject);
            let held = host.hybrid_held(body.subject);
            let choice = settings.policy.select(pixels, prior, examined, held).choice;
            self.previous.insert(body.subject, choice);
            let old = last_coverage.get(&body.subject).copied().unwrap_or(0);
            // The chosen rung decides, so a held bound keeps an examined
            // subject a sprite; examination promotes held live detail at once.
            let coverage = match (choice.detail, examined, prior.is_some()) {
                (BodyDetail::Sprite, _, false) => 16,
                (BodyDetail::Sprite, _, true) => (old + step).min(16),
                (BodyDetail::Live, false, _) => old.saturating_sub(step),
                _ => 0,
            };
            if coverage == 0 {
                continue;
            }
            if !fully_inside(camera, body.bounds) {
                self.stats.live_fallbacks += 1;
                continue;
            }
            requests.push(Request {
                revisions: body.revisions(facing, lighting),
                period: settings.policy.refresh_period(choice.refresh),
                coverage,
                body,
            });
        }

        if self.resident.is_none()
            && !requests.is_empty()
            && self.retiring.is_none()
            && let Some(capacity) = capacity
            && let Ok(atlas) = SpriteAtlas::new(
                device,
                isometer_lens::FRAME_FORMAT,
                settings.resolution,
                capacity,
                isometer_mesh::DEFAULT_MESH_CACHE_CAPACITY,
            )
        {
            self.resident = Some(Resident {
                resolution: settings.resolution,
                capacity,
                atlas,
                images: SpriteCache::default(),
                slots: Slots::new(capacity),
            });
        }
        let Some(resident) = &mut self.resident else {
            self.stats.misses = requests.len() as u32;
            self.stats.live_fallbacks += requests.len() as u32;
            self.needs_frame =
                !requests.is_empty() && self.retiring.is_some() && capacity.is_some();
            return BTreeMap::new();
        };
        resident.atlas.set_light(light);
        self.stats.retained_bytes = resident.atlas.retained_bytes();
        self.stats.sprite_instance_buffer_bytes = resident.atlas.instance_buffer_bytes();
        self.stats.capacity = resident.capacity;

        let mut draws = Vec::new();
        let mut sprites = BTreeMap::new();
        let mut misses = Vec::new();
        for request in requests {
            let subject = request.body.subject;
            if let Some(image) =
                resident
                    .images
                    .reusable(subject, request.revisions, self.frame, request.period)
            {
                self.stats.hits += 1;
                resident.slots.touch(image.slot, self.frame);
                draws.push(image.draw(&request.body, request.coverage));
                sprites.insert(subject, request.coverage);
            } else {
                if let Some(stale) = resident.images.take(subject) {
                    resident.slots.remove(stale.slot);
                }
                self.stats.misses += 1;
                misses.push(request);
            }
        }
        // Use a rotating starting point when a synchronized invalidation
        // produces more work than the frame budget can admit.
        if !misses.is_empty() {
            let start = (self.frame - 1) as usize % misses.len();
            misses.rotate_left(start);
        }
        for request in misses {
            if self.stats.refreshes == settings.max_refreshes {
                self.needs_frame = sprites.len() < resident.capacity as usize;
                break;
            }
            let mut local = request.body.live;
            local.origin = [0.0; 3];
            local.scale = 1.0;
            let Some(projection) = body_bounds(local)
                .ok()
                .flatten()
                .and_then(|bounds| SpriteProjection::new(axes, bounds, settings.resolution))
            else {
                continue;
            };
            let subject = request.body.subject;
            let Some((slot, displaced)) = resident.slots.admit(subject, self.frame) else {
                break;
            };
            if let Some(displaced) = displaced {
                resident.images.remove(displaced);
                self.stats.evictions += 1;
            }
            self.stats.refreshes += 1;
            let placement = SpritePlacement {
                subject,
                origin: request.body.live.origin,
                scale: request.body.live.scale,
            };
            let source = SpriteSource {
                revisions: request.revisions,
                pose_frame: self.frame,
            };
            match resident
                .images
                .resolve(placement, source, self.frame, request.period, || {
                    let stats = resident
                        .atlas
                        .bake_voxel(device, queue, encoder, slot, projection, local)?;
                    self.stats.bake_upload_bytes += (stats.mesh_upload_bytes
                        + stats.frame_upload_bytes
                        + stats.instance_upload_bytes
                        + stats.palette_upload_bytes)
                        as u64;
                    Ok::<_, isometer_render::sprites::SpriteError>(Image { slot, projection })
                }) {
                Ok(Some(instance)) => {
                    draws.push(instance.image.draw(&request.body, request.coverage));
                    sprites.insert(subject, request.coverage);
                },
                _ => resident.slots.remove(slot),
            }
        }
        if !draws.is_empty() {
            match resident.atlas.draw(
                device,
                queue,
                encoder,
                colour,
                &layer.depth_view,
                camera.clip_from_world(),
                &draws,
            ) {
                Ok(upload) => self.stats.sprite_instance_upload_bytes = upload,
                Err(_) => sprites.clear(),
            }
        }
        self.stats.resident_slots = resident.slots.used();
        self.stats.bake_mesh_bytes = resident.atlas.bake_mesh_bytes();
        self.stats.sprite_bodies = sprites.len() as u32;
        self.stats.transition_bodies =
            sprites.values().filter(|&&weight| weight < 16).count() as u32;
        self.needs_frame |= self.stats.transition_bodies > 0;
        self.stats.live_fallbacks += self.stats.hits + self.stats.misses - self.stats.sprite_bodies;
        self.coverage.clone_from(&sprites);
        sprites
    }
}

impl Image {
    fn draw(self, body: &HybridBody<'_>, coverage: u32) -> SpriteDraw {
        SpriteDraw {
            slot: self.slot,
            projection: self.projection,
            origin: body.live.origin,
            scale: body.live.scale,
            coverage,
        }
    }
}

// An orthographic bake cannot supply the newly exposed surfaces of a cut.
// Keep all bodies intersecting a section wall or depth plane on the live path.
fn fully_inside(camera: SlabCamera, (min, max): ([f32; 3], [f32; 3])) -> bool {
    let clip = camera.clip();
    let frustum = camera.frustum().map(|frustum| (frustum, camera.basis()[2]));
    let dot = |normal: [f32; 3], point: [f32; 3]| (0..3).map(|i| normal[i] * point[i]).sum::<f32>();
    (0..8).all(|corner| {
        let point = std::array::from_fn(|i| {
            if corner & (1 << i) == 0 {
                min[i]
            } else {
                max[i]
            }
        });
        let section = dot(clip.normal, point);
        section >= clip.min
            && section <= clip.max
            && clip
                .bounds
                .is_none_or(|(lo, hi)| (0..3).all(|i| lo[i] <= point[i] && point[i] <= hi[i]))
            && frustum.is_none_or(|(frustum, forward)| {
                let depth = dot(forward, std::array::from_fn(|i| point[i] - frustum.eye[i]));
                (frustum.near..=frustum.far).contains(&depth)
            })
    })
}
