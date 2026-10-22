// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Product policy: native anatomy and actual Ground contacts into Isometer's
//! shared implicit renderer. No derived pose is written back to the simulation.
use super::{SiteScene, light::Block};
use isometer::render::implicit::{self, Frame};
use isometer::{BodyLayer, SlabCamera};

pub(super) struct Bodies {
    renderer: Option<implicit::Renderer>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    size: [u32; 2],
}
impl Bodies {
    pub fn new(host: &tenant::HostDevice, format: wgpu::TextureFormat, size: [u32; 2]) -> Self {
        Self {
            renderer: None,
            device: host.device.clone(),
            queue: host.queue.clone(),
            format,
            size,
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
        for (subject, live) in layer.render_bodies() {
            let body = scene
                .body(subject.0)
                .ok_or("smooth body has no native document")?;
            let (surface, stats) = implicit::from_anatomy(&body.document, live, |p| {
                // Ground::surface reads the edited solid column in the same
                // datum as the body, including support beneath rigid remains.
                scene
                    .ground
                    .surface(p[0].floor() as i32, p[2].floor() as i32)
                    .map(|y| (y + 1) as f32)
            })?;
            joints += stats.joints;
            contacts += stats.contacts;
            supports += stats.supports;
            surfaces.insert(subject, surface);
        }
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
        layer.stats.draw_parts = layer.stats.voxel_parts;
        layer.stats.frame_upload_bytes = bytes;
        layer.stats.body_upload_measurement = Some("implicit primitive and frame bytes");
        layer.set_surfaces(surfaces)
    }
}
