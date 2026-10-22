// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Product policy: native anatomy and actual Ground contacts into Isometer's
//! shared implicit renderer. No derived pose is written back to the simulation.
use super::{SiteScene, light::Block};
use isometer::render::implicit::{self, Frame};
use isometer::{BodyLayer, SlabCamera};
mod motion;

pub(super) struct Bodies {
    renderer: Option<implicit::Renderer>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    size: [u32; 2],
    motion: std::collections::BTreeMap<u64, motion::Motion>,
    previous: std::time::Instant,
}
impl Bodies {
    pub fn active(&self) -> bool {
        self.motion.values().any(motion::Motion::active)
    }
    pub fn new(host: &tenant::HostDevice, format: wgpu::TextureFormat, size: [u32; 2]) -> Self {
        Self {
            renderer: None,
            device: host.device.clone(),
            queue: host.queue.clone(),
            format,
            size,
            motion: Default::default(),
            previous: std::time::Instant::now(),
        }
    }
    pub fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        self.size = size;
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(device, size);
        }
    }
    pub fn encode(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        layer: &mut BodyLayer,
        camera: SlabCamera,
        scene: &SiteScene,
        block: Block,
        light: f32,
        target: &wgpu::Texture,
    ) -> Result<(), String> {
        let mut surfaces = std::collections::BTreeMap::new();
        let mut joints = 0;
        let mut contacts = 0;
        let mut supports = 0;
        let mut planted = 0;
        let mut swinging = 0;
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.previous).as_secs_f32().min(0.1);
        self.previous = now;
        let mut retained = std::collections::BTreeSet::new();
        for (subject, mut live) in layer.render_bodies() {
            let body = scene
                .body(subject.0)
                .ok_or("smooth body has no native document")?;
            let mut ground = |p| height(scene, p);
            let (surface, stats) = if body.physical.is_none() {
                implicit::from_anatomy(&body.document, live, ground)?
            } else {
                retained.insert(subject.0);
                let motion = self
                    .motion
                    .entry(body.id)
                    .or_insert_with(|| motion::Motion::new(body, scene));
                live.origin = motion.sample(body, scene, elapsed);
                let elapsed = motion.elapsed(elapsed);
                let witnesses = motion
                    .contacts()
                    .into_iter()
                    .map(|(part, at)| (part, at.into()))
                    .collect();
                implicit::walking_anatomy(
                    &body.document,
                    live,
                    &mut ground,
                    &witnesses,
                    &mut motion.gait,
                    elapsed,
                )?
            };
            joints += stats.joints;
            contacts += stats.contacts;
            supports += stats.supports;
            planted += stats.planted;
            swinging += stats.swinging;
            surfaces.insert(subject, surface);
        }
        self.motion.retain(|id, _| retained.contains(id));
        let batch: Vec<_> = surfaces.values().collect();
        let renderer = self
            .renderer
            .get_or_insert_with(|| implicit::Renderer::new(&self.device, self.format, self.size));
        let bytes = renderer.encode(
            &self.device,
            &self.queue,
            encoder,
            target,
            layer.depth_texture(),
            Frame {
                clip_from_world: camera.clip_from_world(),
                clip: camera.clip(),
                toward_sun: block.toward,
                ambient: block.ambient * light,
                sun: block.sun.map(|v| v * light),
            },
            &batch,
        )?;
        layer.stats.smooth_primitives = batch.iter().map(|s| s.parts.len()).sum();
        layer.stats.posed_joints = joints;
        layer.stats.foot_contacts = contacts;
        layer.stats.support_contacts = supports;
        layer.stats.planted_contacts = planted;
        layer.stats.swinging_contacts = swinging;
        layer.stats.draw_parts = layer.stats.voxel_parts;
        layer.stats.frame_upload_bytes = bytes;
        layer.stats.body_upload_measurement = Some("implicit primitive and frame bytes");
        layer.set_surfaces(surfaces)
    }
}

fn height(scene: &SiteScene, p: [f32; 3]) -> Option<f32> {
    let sole = [
        p[0].floor() as i32,
        p[1].round() as i32,
        p[2].floor() as i32,
    ];
    if (p[1] - sole[1] as f32).abs() < 0.025
        && scene
            .ground
            .material([sole[0], sole[1] - 1, sole[2]])
            .is_some_and(|m| m != scene.window.materials.air && m != scene.window.materials.water)
        && scene.ground.material(sole) == Some(scene.window.materials.air)
    {
        return Some(sole[1] as f32);
    }
    scene
        .ground
        .surface(p[0].floor() as i32, p[2].floor() as i32)
        .filter(|y| {
            scene
                .ground
                .material([p[0].floor() as i32, *y, p[2].floor() as i32])
                .is_some_and(|m| {
                    m != scene.window.materials.air && m != scene.window.materials.water
                })
        })
        .map(|y| (y + 1) as f32)
}
