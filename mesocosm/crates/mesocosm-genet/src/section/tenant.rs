// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's smooth-body target, using the host's device and frame encoder.
//! Voxel bodies draw through Isometer's shared body pass (case 1179); only
//! the smooth implicit bodies still render here, and the frame composites
//! this target over the traced terrain.

use super::light::Block;
use isometer::{BodyLayer, SlabCamera};
use tenant::HostDevice;

const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

pub(super) struct BodyTenant {
    target: wgpu::Texture,
    light: f32,
    block: Block,
    smooth: super::smooth::Bodies,
}

impl BodyTenant {
    pub fn animating(&self) -> bool {
        self.smooth.active()
    }
    pub fn new(host: &HostDevice, size: [u32; 2]) -> Self {
        Self {
            target: target(&host.device, size),
            light: 1.0,
            block: Block::fixed(),
            smooth: super::smooth::Bodies::new(host, FORMAT, size),
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        self.target = target(device, size);
        self.smooth.resize(device, size);
    }

    pub fn view(&self) -> wgpu::TextureView {
        self.target.create_view(&Default::default())
    }

    pub fn set_light(&mut self, intensity: f32) {
        self.light = intensity;
    }

    pub fn light(&self) -> f32 {
        self.light
    }

    pub fn block(&self) -> Block {
        self.block
    }

    pub fn set_block(&mut self, block: Block) {
        self.block = block;
    }

    /// Smooth bodies, sun-lit and exactly sRGB-encoded by the target format.
    pub fn encode(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        layer: &mut BodyLayer,
        camera: SlabCamera,
        scene: &super::SiteScene,
    ) -> Result<(), String> {
        self.smooth.encode(
            encoder,
            layer,
            camera,
            scene,
            self.block,
            self.light,
            &self.target,
        )
    }
}

fn target(device: &wgpu::Device, size: [u32; 2]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mesocosm lit bodies"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[FORMAT.remove_srgb_suffix()],
    })
}
