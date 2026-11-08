// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared-device draw adapter for rigid, greedy-meshed body volumes.
//!
//! The cache is deliberately keyed only by immutable `VolumeRef` bytes. A
//! placement, an organism's movement, scale, and its presentation tint are
//! per-frame instance data, so they do not cause static geometry uploads.

use std::collections::{BTreeMap, BTreeSet};

use bytemuck::{Pod, Zeroable};
#[cfg(test)]
use glam::Mat4;
#[cfg(test)]
use isometer_core::Yaw;
use isometer_core::{PartId, VolumeRef};
use isometer_mesh::BodyMesh;
use wgpu::util::DeviceExt;

mod draws;
mod frame;
mod materials;
mod palette;
mod pose;
mod query;
mod resident;
mod vertex;

pub use frame::{BodyDisplay, BodyLight};
use frame::FrameUniform;
pub use materials::{PartMaterial, TISSUE_CHANNELS, tissue_average, tissue_colour};
pub use palette::{MaterialPalette, PALETTE_ENTRIES, PaletteColour, linear_from_display};
pub use query::{BodyHit, BodyQueryError, body_bounds, part_transform, pick_bodies, posed_quad};
pub(crate) use resident::upload_in_order;

use materials::part_appearance;
#[cfg(test)]
use materials::valid_materials;
use palette::{PaletteBlock, PaletteTable, palette_slot};
use pose::{model_matrix, validate_body, validate_clip};
use vertex::BodyVertex;

/// A body owned by the caller's projection. Identity remains alongside this
/// lightweight draw item in the host's attributed projection.
#[derive(Clone, Copy, Debug)]
pub struct LiveBody<'a> {
    pub mesh: &'a BodyMesh,
    /// World-space location of the body's local origin, in voxel units.
    pub origin: [f32; 3],
    /// Uniform world scale for this body.
    pub scale: f32,
    /// Continuous presentation yaw around the body's local origin.
    /// Positive radians follow `Yaw`; attachments turn with their parent body.
    pub yaw_radians: f32,
    /// Linear multiplier applied after the volume material colour.
    pub tint: [f32; 3],
    /// Gives this body a restrained inspection emphasis.
    pub focused: bool,
    /// The one addressed part that receives the inspection colour.
    pub selected_part: Option<PartId>,
    /// Per-part expression summaries from the host's phenotype projection.
    /// They are per-instance data and never alter the immutable volume cache.
    pub materials: &'a [PartMaterial],
    /// Optional colour table for this body's material ids. `None`, an empty
    /// table, or an id the table is too short to name all keep the hashed
    /// [`material_colour`](crate::material_colour) this pass has always used.
    ///
    /// Per-instance data like the tint: the immutable volume cache is keyed by
    /// volume bytes alone, and a palette never enters it.
    pub palette: Option<MaterialPalette<'a>>,
}

impl<'a> LiveBody<'a> {
    pub fn new(mesh: &'a BodyMesh, origin: [f32; 3]) -> Self {
        Self {
            mesh,
            origin,
            scale: 1.0,
            yaw_radians: 0.0,
            tint: [1.0; 3],
            focused: false,
            selected_part: None,
            materials: &[],
            palette: None,
        }
    }
}

/// Inclusive cut interval in world coordinates. This is the tracer's
/// world-vertical slab, not a camera-forward near/far substitute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipSlab {
    pub normal: [f32; 3],
    pub min: f32,
    pub max: f32,
    /// Optional fixed world volume, independent of camera orientation.
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

impl ClipSlab {
    pub const fn new(normal: [f32; 3], min: f32, max: f32) -> Self {
        Self {
            normal,
            min,
            max,
            bounds: None,
        }
    }
}

/// Receipts for the last draw. Upload counters stay at zero for an unchanged
/// frame, even though it still submits its draw calls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BodyDrawStats {
    pub cached_meshes: usize,
    /// Distinct immutable volume buffers created for this call.
    pub mesh_builds: usize,
    pub mesh_uploads: usize,
    pub mesh_upload_bytes: usize,
    pub static_vertices_uploaded: usize,
    pub frame_upload_bytes: usize,
    pub instances: usize,
    pub instance_upload_bytes: usize,
    /// Colour-table bytes written this call. Zero for every frame whose
    /// bodies carry no palette, including the first.
    pub palette_upload_bytes: usize,
    /// Rigid part placements submitted, before batching shared volumes.
    pub draw_parts: usize,
    pub draws: usize,
    pub evictions: usize,
}

/// The adapter declines a frame before recording a partial body draw when its
/// bounded immutable-volume cache cannot represent every required volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveBodyError {
    /// A public `BodyMesh` placement did not resolve to its promised geometry.
    /// Refuse the entire call so a host can use its counted fallback.
    MissingMesh {
        volume: VolumeRef,
    },
    CacheOverflow {
        required_meshes: usize,
        capacity: usize,
    },
    InvalidBody,
    /// The caller supplied a non-finite, negative, or over-capacity process
    /// summary. Refuse rather than making a decorative claim about tissue.
    InvalidMaterials,
    InvalidClip,
    InvalidSpriteCoverage,
}

struct CachedMesh {
    vertices: wgpu::Buffer,
    vertex_count: u32,
    last_used: u64,
}

struct CachedInstances {
    buffer: wgpu::Buffer,
    capacity: usize,
    bytes: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq)]
struct Instance {
    model: [[f32; 4]; 4],
    tint: [f32; 4],
    expression: [f32; 4],
    expression_tail: [f32; 4],
}

impl Instance {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 96,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    };
}

/// Draws cached volume meshes into a pass that the caller owns.
pub struct LiveBodyRenderer {
    pipeline: wgpu::RenderPipeline,
    frame_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    meshes: BTreeMap<[u8; 32], CachedMesh>,
    /// Keyed by volume *and* palette slot: two bodies of one volume under
    /// different colour tables are two batches over one cached geometry.
    instances: BTreeMap<([u8; 32], u32), CachedInstances>,
    palettes: PaletteTable,
    max_cached_meshes: usize,
    clock: u64,
    last_frame: Option<FrameUniform>,
    last_stats: BodyDrawStats,
    light: Option<BodyLight>,
    /// The style of bodies drawn without one of their own, and of bakes.
    display: BodyDisplay,
}

impl LiveBodyRenderer {
    /// Builds against the application's existing device. The adapter never
    /// requests a device or owns a target, depth texture, or command encoder.
    pub fn new(
        device: &wgpu::Device,
        colour_format: wgpu::TextureFormat,
        max_cached_meshes: usize,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesocosm live body"),
            source: wgpu::ShaderSource::Wgsl(
                concat!(
                    include_str!("ordered_dither.wgsl"),
                    include_str!("live_body.wgsl")
                )
                .into(),
            ),
        });
        let frame_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesocosm live body frame"),
            contents: bytemuck::bytes_of(&FrameUniform::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesocosm live body frame layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesocosm live body frame bind group"),
            layout: &frame_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });
        let palettes = PaletteTable::new(device);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesocosm live body layout"),
            bind_group_layouts: &[Some(&frame_layout), Some(&palettes.layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesocosm live body pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(BodyVertex::LAYOUT), Some(Instance::LAYOUT)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: colour_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
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
            frame_buffer,
            frame_bind_group,
            meshes: BTreeMap::new(),
            instances: BTreeMap::new(),
            palettes,
            max_cached_meshes: max_cached_meshes.max(1),
            clock: 0,
            last_frame: None,
            last_stats: BodyDrawStats::default(),
            light: None,
            display: BodyDisplay::Srgb,
        }
    }

    /// Light later draws and bakes by the scene's sun, or restore the fixed
    /// face shade with `None`.
    pub fn set_light(&mut self, light: Option<BodyLight>) {
        self.light = light;
    }

    /// The style for later bakes and for bodies drawn without a style.
    pub fn set_display(&mut self, display: BodyDisplay) {
        self.display = display;
    }

    pub fn last_stats(&self) -> BodyDrawStats {
        self.last_stats
    }

    pub(super) fn draw_inner(
        &mut self,
        device: &wgpu::Device,
        queue: Option<&wgpu::Queue>,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        clip_slab: Option<ClipSlab>,
        bodies: &[LiveBody<'_>],
        styles: &[BodyDisplay],
        sprite_coverage: &[u32],
    ) -> Result<BodyDrawStats, LiveBodyError> {
        if !clip_from_world
            .iter()
            .flatten()
            .all(|value| value.is_finite())
        {
            return Err(LiveBodyError::InvalidClip);
        }
        validate_clip(clip_slab)?;
        for body in bodies {
            validate_body(*body)?;
        }
        self.clock = self.clock.wrapping_add(1);
        let mut stats = BodyDrawStats::default();
        let mut instances: BTreeMap<([u8; 32], u32), Vec<Instance>> = BTreeMap::new();
        let mut tables: Vec<MaterialPalette<'_>> = Vec::new();
        let mut volumes: BTreeSet<[u8; 32]> = BTreeSet::new();
        for (index, body) in bodies.iter().enumerate() {
            let slot = palette_slot(body.palette, &mut tables);
            for placement in &body.mesh.placements {
                let key = (placement.volume.0, slot);
                volumes.insert(placement.volume.0);
                let model = model_matrix(*body, placement.yaw, placement.pivot, placement.pivot_at);
                let mut appearance = part_appearance(*body, placement.part);
                // The expression tail has one fraction and a spare lane for
                // the per-body ordered transition weight.
                appearance.expression_tail[1] =
                    sprite_coverage.get(index).copied().unwrap_or(0) as f32;
                appearance.expression_tail[2] =
                    styles.get(index).copied().unwrap_or(self.display).lane();
                instances.entry(key).or_default().push(Instance {
                    model: model.to_cols_array_2d(),
                    tint: appearance.tint,
                    expression: appearance.expression,
                    expression_tail: appearance.expression_tail,
                });
            }
        }
        // The cache holds geometry, so its capacity is counted in distinct
        // volumes rather than in batches a palette may have split.
        if volumes.len() > self.max_cached_meshes {
            self.last_stats = BodyDrawStats {
                cached_meshes: self.meshes.len(),
                ..stats
            };
            return Err(LiveBodyError::CacheOverflow {
                required_meshes: volumes.len(),
                capacity: self.max_cached_meshes,
            });
        }
        for body in bodies {
            for placement in &body.mesh.placements {
                let Some(part_mesh) = body.mesh.mesh_for(placement.volume) else {
                    continue;
                };
                self.cache_mesh(device, placement.volume.0, part_mesh, &mut stats);
            }
        }
        let frame = FrameUniform::new(clip_from_world, clip_slab, self.light);
        let mut write = |buffer: &wgpu::Buffer, offset, bytes: &[u8]| match queue {
            Some(queue) => queue.write_buffer(buffer, offset, bytes),
            None => upload_in_order(device, encoder, buffer, offset, bytes),
        };
        if self.last_frame.as_ref().map(bytemuck::bytes_of) != Some(bytemuck::bytes_of(&frame)) {
            write(&self.frame_buffer, 0, bytemuck::bytes_of(&frame));
            stats.frame_upload_bytes = size_of::<FrameUniform>();
            self.last_frame = Some(frame);
        }
        let mut blocks = vec![PaletteBlock::zeroed()];
        blocks.extend(tables.iter().map(|table| PaletteBlock::of(*table)));
        stats.palette_upload_bytes = self.palettes.upload(device, &blocks, &mut write);
        for (key, batch) in &instances {
            stats.instances += batch.len();
            stats.draw_parts += batch.len();
            self.update_instances(device, *key, batch, &mut stats, &mut write);
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("mesocosm live bodies"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: colour,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if queue.is_some() {
                        wgpu::LoadOp::Load
                    } else {
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: if queue.is_some() {
                        wgpu::LoadOp::Load
                    } else {
                        wgpu::LoadOp::Clear(1.0)
                    },
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.frame_bind_group, &[]);
        for (key, batch) in &instances {
            let cached = self.meshes.get(&key.0).expect("cached before batching");
            if cached.vertex_count == 0 {
                continue;
            }
            let instance_buffer = &self.instances.get(key).expect("updated before pass").buffer;
            pass.set_bind_group(
                1,
                self.palettes.bind_group(),
                &[PaletteTable::offset(key.1)],
            );
            pass.set_vertex_buffer(0, cached.vertices.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.draw(0..cached.vertex_count, 0..batch.len() as u32);
            stats.draws += 1;
        }
        drop(pass);
        stats.cached_meshes = self.meshes.len();
        self.last_stats = stats;
        Ok(stats)
    }
}

#[cfg(test)]
#[path = "live_body_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "live_body/palette_tests.rs"]
mod palette_tests;
