// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded opaque punctuation strokes in the scene's own attachments.
//! World positions and the slab camera determine actual raster depth.
use crate::camera::SlabCamera;
use crate::scene::Scene;

pub const MAX_SPATIAL_GLYPHS: usize = 128;
/// The stroke shapes the shader draws. A presentation vocabulary and nothing
/// more: what a stroke *means* belongs to the host that placed it.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Stroke {
    #[default]
    Quotes,
    Slashes,
    Backticks,
}
impl Stroke {
    pub const ALL: &'static [Self] = &[Self::Quotes, Self::Slashes, Self::Backticks];
}
/// The plane in which strokes are constructed. A world plane remains fixed
/// when the camera turns. Its axes must be finite orthonormal world vectors;
/// the caller supplies any surface offset needed to avoid coplanar depth ties.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GlyphOrientation {
    #[default]
    CameraFacing,
    WorldPlane {
        right: [f32; 3],
        up: [f32; 3],
    },
}
impl GlyphOrientation {
    fn valid(self) -> bool {
        match self {
            Self::CameraFacing => true,
            Self::WorldPlane { right, up } => {
                let dot = |a: [f32; 3], b: [f32; 3]| {
                    a.into_iter().zip(b).map(|(a, b)| a * b).sum::<f32>()
                };
                right.iter().chain(up.iter()).all(|v| v.is_finite())
                    && (dot(right, right) - 1.).abs() <= 1e-4
                    && (dot(up, up) - 1.).abs() <= 1e-4
                    && dot(right, up).abs() <= 1e-4
            },
        }
    }
}
/// Host-owned presentation only. Size is the glyph height in world units;
/// angle is radians in the selected orientation's plane. Colour is display-encoded
/// opaque RGBA, matching the section target's existing byte convention.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpatialGlyph {
    pub centre: [f32; 3],
    pub size: f32,
    pub angle: f32,
    pub glyph: Stroke,
    pub orientation: GlyphOrientation,
    pub color: [f32; 4],
}

pub(crate) struct GlyphLayer {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    clip: wgpu::Buffer,
    binding: wgpu::BindGroup,
    glyphs: Vec<SpatialGlyph>,
}
const STRIDE: u64 = 44;
fn bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    values.into_iter().flat_map(f32::to_le_bytes).collect()
}
impl GlyphLayer {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spatial glyph vertices"),
            size: MAX_SPATIAL_GLYPHS as u64 * 24 * STRIDE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let clip = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("spatial glyph clip"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("spatial glyph layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("spatial glyph binding"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: clip.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("spatial glyph shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("glyphs.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("spatial glyph pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shared depth glyph strokes"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: STRIDE,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x3],
                })],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: mesocosm_lens::FRAME_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
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
        Self {
            pipeline,
            vertices,
            clip,
            binding,
            glyphs: Vec::new(),
        }
    }
    pub fn set(&mut self, glyphs: Vec<SpatialGlyph>) -> Result<(), String> {
        if glyphs.len() > MAX_SPATIAL_GLYPHS {
            return Err("At most 128 spatial glyphs are admitted.".into());
        }
        if glyphs.iter().any(|g| {
            g.centre
                .iter()
                .chain(g.color.iter())
                .any(|v| !v.is_finite())
                || !g.size.is_finite()
                || g.size <= 0.
                || !g.angle.is_finite()
                || g.color.iter().any(|v| !(0.0..=1.0).contains(v))
                || g.color[3] != 1.
                || !g.orientation.valid()
        }) {
            return Err(
                "Spatial glyphs require finite poses, positive size, opaque colours in 0..1 and orthonormal world-plane axes."
                    .into(),
            );
        }
        self.glyphs = glyphs;
        Ok(())
    }
    pub fn draw(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        camera: SlabCamera,
    ) {
        if self.glyphs.is_empty() {
            return;
        }
        let data = geometry(&self.glyphs, camera);
        queue.write_buffer(&self.vertices, 0, &data);
        let clip = camera.clip();
        let (minimum, maximum) = clip.bounds.unwrap_or(([0.; 3], [0.; 3]));
        let uniform = bytes([
            clip.normal[0],
            clip.normal[1],
            clip.normal[2],
            0.,
            clip.min,
            clip.max,
            if clip.bounds.is_some() { 1. } else { 0. },
            0.,
            minimum[0],
            minimum[1],
            minimum[2],
            0.,
            maximum[0],
            maximum[1],
            maximum[2],
            0.,
        ]);
        queue.write_buffer(&self.clip, 0, &uniform);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("spatial glyph shared depth"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.binding, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.draw(0..(data.len() as u64 / STRIDE) as u32, 0..1);
    }
}
fn geometry(glyphs: &[SpatialGlyph], camera: SlabCamera) -> Vec<u8> {
    let [camera_right, camera_up, _] = camera.basis();
    let matrix = camera.clip_from_world();
    let mut data = Vec::new();
    for glyph in glyphs {
        let (right, up) = match glyph.orientation {
            GlyphOrientation::CameraFacing => (camera_right, camera_up),
            GlyphOrientation::WorldPlane { right, up } => (right, up),
        };
        let segments: &[[[f32; 2]; 2]] = match glyph.glyph {
            Stroke::Slashes => &[[[-0.28, -0.5], [0.28, 0.5]]],
            Stroke::Backticks => &[[[-0.2, 0.5], [0.15, 0.15]]],
            Stroke::Quotes => &[
                [[-0.32, 0.5], [-0.22, 0.22]],
                [[-0.22, 0.22], [-0.4, 0.02]],
                [[0.26, 0.5], [0.36, 0.22]],
                [[0.36, 0.22], [0.18, 0.02]],
            ],
        };
        for &[a, b] in segments {
            let dx = b[0] - a[0];
            let dy = b[1] - a[1];
            let length = dx.hypot(dy);
            let offset = [-dy / length * 0.065, dx / length * 0.065];
            let corners = [
                [a[0] + offset[0], a[1] + offset[1]],
                [a[0] - offset[0], a[1] - offset[1]],
                [b[0] - offset[0], b[1] - offset[1]],
                [b[0] + offset[0], b[1] + offset[1]],
            ];
            for index in [0, 1, 2, 0, 2, 3] {
                let [x, y] = corners[index];
                let x_rot = glyph.size * (x * glyph.angle.cos() - y * glyph.angle.sin());
                let y_rot = glyph.size * (x * glyph.angle.sin() + y * glyph.angle.cos());
                let world: [f32; 3] =
                    [0, 1, 2].map(|i| glyph.centre[i] + right[i] * x_rot + up[i] * y_rot);
                let clip: [f32; 4] = [0, 1, 2, 3].map(|row| {
                    matrix[3][row] + (0..3).map(|col| matrix[col][row] * world[col]).sum::<f32>()
                });
                data.extend(bytes(clip.into_iter().chain(glyph.color).chain(world)));
            }
        }
    }
    data
}
impl Scene {
    /// Replaces the complete bounded presentation list. Invalid input leaves
    /// the previous list intact. Empty input removes all glyphs.
    pub fn set_glyphs(&mut self, glyphs: Vec<SpatialGlyph>) -> Result<(), String> {
        if glyphs.is_empty() && self.glyphs.is_none() {
            return Ok(());
        }
        self.glyphs
            .get_or_insert_with(|| GlyphLayer::new(&self.device))
            .set(glyphs)
    }
}
#[cfg(test)]
mod tests;
