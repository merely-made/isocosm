// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Primitive, Surface};
use crate::ClipSlab;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

const BODIES: usize = 128;
const PARTS: usize = 8192;
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Uniform {
    clip: [[f32; 4]; 4],
    inverse: [[f32; 4]; 4],
    slab: [f32; 4],
    limits: [f32; 4],
    bounds_min: [f32; 4],
    bounds_max: [f32; 4],
    light: [f32; 4],
    sun: [f32; 4],
}
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Draw {
    min: [f32; 4],
    max: [f32; 4],
    range: [u32; 4],
    rect: [f32; 4],
}

pub struct Frame {
    pub clip_from_world: [[f32; 4]; 4],
    pub clip: ClipSlab,
    pub toward_sun: [f32; 3],
    pub ambient: f32,
    pub sun: [f32; 3],
}

/// One caller-owned encoder, the existing terrain depth, and a bounded batch
/// of per-body ray rectangles. Static pipelines/buffers survive every frame.
pub struct Renderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    frame: wgpu::Buffer,
    parts: wgpu::Buffer,
    draws: wgpu::Buffer,
    depth: wgpu::Texture,
}
impl Renderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, size: [u32; 2]) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("implicit bodies"),
            entries: &[
                buffer_entry(
                    0,
                    wgpu::BufferBindingType::Uniform,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                ),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                buffer_entry(
                    2,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                buffer_entry(
                    3,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                ),
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("implicit anatomy"),
            source: wgpu::ShaderSource::Wgsl(include_str!("surface.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("implicit anatomy"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("implicit anatomy"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let make = |label, size, usage| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        Self {
            pipeline,
            layout,
            frame: make(
                "implicit frame",
                std::mem::size_of::<Uniform>() as u64,
                wgpu::BufferUsages::UNIFORM,
            ),
            parts: make(
                "implicit parts",
                (PARTS * std::mem::size_of::<Primitive>()) as u64,
                wgpu::BufferUsages::STORAGE,
            ),
            draws: make(
                "implicit draws",
                (BODIES * std::mem::size_of::<Draw>()) as u64,
                wgpu::BufferUsages::STORAGE,
            ),
            depth: depth(device, size),
        }
    }
    pub fn resize(&mut self, device: &wgpu::Device, size: [u32; 2]) {
        self.depth = depth(device, size);
    }
    pub fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::Texture,
        terrain: &wgpu::Texture,
        frame: Frame,
        surfaces: &[&Surface],
    ) -> Result<u64, String> {
        if surfaces.len() > BODIES || surfaces.iter().map(|s| s.parts.len()).sum::<usize>() > PARTS
        {
            return Err("implicit body batch exceeds its bounded capacity".into());
        }
        if surfaces.iter().any(|s| !s.valid()) {
            return Err("invalid implicit surface".into());
        }
        let matrix = Mat4::from_cols_array_2d(&frame.clip_from_world);
        let inverse = matrix.inverse();
        if !inverse.is_finite() {
            return Err("invalid implicit body camera".into());
        }
        let mut parts = Vec::new();
        let mut draws = Vec::new();
        for surface in surfaces {
            let Some((min, max)) = surface.bounds() else {
                continue;
            };
            let mut lo = Vec3::splat(f32::INFINITY);
            let mut hi = -lo;
            for x in [min[0], max[0]] {
                for y in [min[1], max[1]] {
                    for z in [min[2], max[2]] {
                        let p = matrix.project_point3(Vec3::new(x, y, z));
                        lo = lo.min(p);
                        hi = hi.max(p);
                    }
                }
            }
            draws.push(Draw {
                min: [min[0], min[1], min[2], surface.blend],
                max: [max[0], max[1], max[2], 0.0],
                range: [parts.len() as u32, surface.parts.len() as u32, 0, 0],
                rect: [lo.x.max(-1.0), lo.y.max(-1.0), hi.x.min(1.0), hi.y.min(1.0)],
            });
            parts.extend_from_slice(&surface.parts);
        }
        let clip = frame.clip;
        let (min, max, enabled) = clip
            .bounds
            .map_or(([0.0; 3], [0.0; 3], 0.0), |(a, b)| (a, b, 1.0));
        let uniform = Uniform {
            clip: frame.clip_from_world,
            inverse: inverse.to_cols_array_2d(),
            slab: [clip.normal[0], clip.normal[1], clip.normal[2], clip.min],
            limits: [clip.max, 0.0, 0.0, 0.0],
            bounds_min: [min[0], min[1], min[2], enabled],
            bounds_max: [max[0], max[1], max[2], 0.0],
            light: [
                frame.toward_sun[0],
                frame.toward_sun[1],
                frame.toward_sun[2],
                frame.ambient,
            ],
            sun: [frame.sun[0], frame.sun[1], frame.sun[2], 0.0],
        };
        queue.write_buffer(&self.frame, 0, bytemuck::bytes_of(&uniform));
        if !parts.is_empty() {
            queue.write_buffer(&self.parts, 0, bytemuck::cast_slice(&parts));
        }
        if !draws.is_empty() {
            queue.write_buffer(&self.draws, 0, bytemuck::cast_slice(&draws));
        }
        let terrain = terrain.create_view(&Default::default());
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("implicit frame"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.frame.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&terrain),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.parts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.draws.as_entire_binding(),
                },
            ],
        });
        let target = target.create_view(&Default::default());
        let depth = self.depth.create_view(&Default::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("implicit bodies over traced terrain"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..6, 0..draws.len() as u32);
        Ok((std::mem::size_of::<Uniform>()
            + parts.len() * std::mem::size_of::<Primitive>()
            + draws.len() * std::mem::size_of::<Draw>()) as u64)
    }
}
fn buffer_entry(
    binding: u32,
    ty: wgpu::BufferBindingType,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
fn depth(device: &wgpu::Device, size: [u32; 2]) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("implicit body depth"),
        size: wgpu::Extent3d {
            width: size[0].max(1),
            height: size[1].max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}
