// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Resident local-pose images. The caller owns slot admission and revisions,
//! and places the held image at the current root on every draw.

use crate::live_body::upload_in_order;
use crate::{BodyDrawStats, LiveBody, LiveBodyError, LiveBodyRenderer};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

const COLOUR: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteProjection {
    local_from_clip: Mat4,
    resolution: u32,
}

impl SpriteProjection {
    /// Frame the root-relative, unscaled AABB using the scene's right, up and
    /// forward axes. Square pixels and one texel of transparent border keep
    /// the image's aspect and sampling stable.
    pub fn new(
        axes: [[f32; 3]; 3],
        (min, max): ([f32; 3], [f32; 3]),
        resolution: u32,
    ) -> Option<Self> {
        let [right, up, forward] = axes.map(Vec3::from_array);
        let min = Vec3::from_array(min);
        let max = Vec3::from_array(max);
        if resolution < 4
            || !min.is_finite()
            || !max.is_finite()
            || !min.cmple(max).all()
            || [right, up, forward]
                .iter()
                .any(|axis| !axis.is_finite() || (axis.length_squared() - 1.0).abs() > 1.0e-3)
            || [right.dot(up), right.dot(forward), up.dot(forward)]
                .iter()
                .any(|dot| dot.abs() > 1.0e-3)
        {
            return None;
        }
        let centre = (min + max) * 0.5;
        let half = (max - min) * 0.5;
        let side = half.dot(right.abs()).max(half.dot(up.abs())) / (1.0 - 2.0 / resolution as f32);
        let depth = half.dot(forward.abs()).max(side * 0.01) * 1.02;
        let local_from_clip = Mat4::from_cols(
            (right * side).extend(0.0),
            (up * side).extend(0.0),
            (forward * depth * 2.0).extend(0.0),
            (centre - forward * depth).extend(1.0),
        );
        local_from_clip.inverse().is_finite().then_some(Self {
            local_from_clip,
            resolution,
        })
    }

    fn clip_from_local(self) -> [[f32; 4]; 4] {
        self.local_from_clip.inverse().to_cols_array_2d()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpriteDraw {
    pub slot: u32,
    pub projection: SpriteProjection,
    pub origin: [f32; 3],
    pub scale: f32,
    /// Number of pixels in each 4-by-4 ordered cell that select the sprite.
    /// Use 16 when the body draws exclusively as a sprite.
    pub coverage: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteError {
    InvalidAtlas,
    InvalidSlot,
    InvalidProjection,
    InvalidPlacement,
    InvalidCoverage,
    TooManyDraws,
    Bake(LiveBodyError),
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct Instance {
    clip_from_image: [[f32; 4]; 4],
    slot: u32,
    coverage: u32,
}

/// A fixed-size texture pair on the caller's device. Each refresh replaces one
/// layer, and the scene samples the same layers on held frames.
/// Submit recorded work before reusing the atlas with another encoder. If an
/// encoder is abandoned, discard the atlas with it because uploads are ordered
/// into the recording and may not have reached the device.
pub struct SpriteAtlas {
    colour: wgpu::Texture,
    depth: wgpu::Texture,
    resolution: u32,
    slots: u32,
    baker: LiveBodyRenderer,
    pipeline: wgpu::RenderPipeline,
    textures: wgpu::BindGroup,
    instances: wgpu::Buffer,
    previous: Vec<Instance>,
}

impl SpriteAtlas {
    /// Use the scene's display-encoded colour format for the draw target.
    pub fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        resolution: u32,
        slots: u32,
        mesh_capacity: usize,
    ) -> Result<Self, SpriteError> {
        let limits = device.limits();
        let instance_bytes = u64::from(slots) * size_of::<Instance>() as u64;
        if resolution < 4
            || resolution > limits.max_texture_dimension_2d
            || slots == 0
            || slots > limits.max_texture_array_layers
            || instance_bytes > limits.max_buffer_size
            || atlas_bytes(resolution, slots).is_none()
        {
            return Err(SpriteError::InvalidAtlas);
        }
        let size = wgpu::Extent3d {
            width: resolution,
            height: resolution,
            depth_or_array_layers: slots,
        };
        let texture = |label, format| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let colour = texture("sprite atlas colour", COLOUR);
        let depth = texture("sprite atlas depth", DEPTH);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite atlas texture layout"),
            entries: &[
                texture_entry(0, wgpu::TextureSampleType::Float { filterable: false }),
                texture_entry(1, wgpu::TextureSampleType::Depth),
            ],
        });
        let array = wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        };
        let textures = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite atlas textures"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&colour.create_view(&array)),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&depth.create_view(&array)),
                },
            ],
        });
        let pipeline = draw_pipeline(device, &layout, target_format);
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite draw instances"),
            size: instance_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Ok(Self {
            colour,
            depth,
            resolution,
            slots,
            baker: LiveBodyRenderer::new(device, COLOUR, mesh_capacity),
            pipeline,
            textures,
            instances,
            previous: Vec::new(),
        })
    }

    /// Nominal colour and depth image bytes. Instance, geometry, and transient
    /// upload buffers are separate from these images.
    pub fn retained_bytes(&self) -> u64 {
        atlas_bytes(self.resolution, self.slots).expect("checked on construction")
    }

    pub fn instance_buffer_bytes(&self) -> u64 {
        self.instances.size()
    }

    pub fn bake_mesh_bytes(&self) -> u64 {
        self.baker.mesh_buffer_bytes()
    }

    fn layers(
        &self,
        slot: u32,
        projection: SpriteProjection,
    ) -> Result<(wgpu::TextureView, wgpu::TextureView), SpriteError> {
        if slot >= self.slots {
            return Err(SpriteError::InvalidSlot);
        }
        if projection.resolution != self.resolution {
            return Err(SpriteError::InvalidProjection);
        }
        let layer = wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: slot,
            array_layer_count: Some(1),
            ..Default::default()
        };
        Ok((
            self.colour.create_view(&layer),
            self.depth.create_view(&layer),
        ))
    }

    /// Light later bakes by the scene's sun; images baked under another
    /// light must be invalidated by the caller's lighting revision.
    pub fn set_light(&mut self, light: Option<crate::live_body::BodyLight>) {
        self.baker.set_light(light);
    }

    /// The presentation style later bakes use; the caller keys images by it.
    pub fn set_display(&mut self, display: crate::live_body::BodyDisplay) {
        self.baker.set_display(display);
    }

    /// Record a posed voxel body into one atlas layer. Yaw, appearance and
    /// light are baked. Origin and uniform scale are applied later at display
    /// time. Refreshes and sprite draws can be recorded in one submitted encoder.
    pub fn bake_voxel(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        slot: u32,
        projection: SpriteProjection,
        mut body: LiveBody<'_>,
    ) -> Result<BodyDrawStats, SpriteError> {
        let (colour, depth) = self.layers(slot, projection)?;
        body.origin = [0.0; 3];
        body.scale = 1.0;
        self.baker
            .bake(
                device,
                encoder,
                &colour,
                &depth,
                projection.clip_from_local(),
                body,
            )
            .map_err(SpriteError::Bake)
    }

    /// Draw held images into the caller's loaded colour and standard-depth
    /// attachments. The quad is planar. Each opaque texel writes the depth of
    /// its baked local surface after reprojecting from the current root.
    /// Returns bytes uploaded for changed instances, separately from bakes.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        sprites: &[SpriteDraw],
    ) -> Result<u64, SpriteError> {
        if sprites.len() > self.slots as usize {
            return Err(SpriteError::TooManyDraws);
        }
        if sprites.is_empty() {
            return Ok(0);
        }
        let clip = Mat4::from_cols_array_2d(&clip_from_world);
        let mut instances = Vec::with_capacity(sprites.len());
        for sprite in sprites {
            if sprite.slot >= self.slots {
                return Err(SpriteError::InvalidSlot);
            }
            if sprite.projection.resolution != self.resolution {
                return Err(SpriteError::InvalidProjection);
            }
            if sprite.coverage > 16 {
                return Err(SpriteError::InvalidCoverage);
            }
            let transform = clip
                * Mat4::from_translation(Vec3::from_array(sprite.origin))
                * Mat4::from_scale(Vec3::splat(sprite.scale))
                * sprite.projection.local_from_clip;
            if !transform.is_finite() || sprite.scale <= 0.0 {
                return Err(SpriteError::InvalidPlacement);
            }
            instances.push(Instance {
                clip_from_image: transform.to_cols_array_2d(),
                slot: sprite.slot,
                coverage: sprite.coverage,
            });
        }
        let uploaded = if instances != self.previous {
            let bytes = bytemuck::cast_slice(&instances);
            upload_in_order(device, encoder, &self.instances, 0, bytes);
            let size = bytes.len() as u64;
            self.previous = instances;
            size
        } else {
            0
        };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("resident sprites"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: colour,
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
        pass.set_bind_group(0, &self.textures, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..sprites.len() as u32);
        Ok(uploaded)
    }
}

fn atlas_bytes(resolution: u32, slots: u32) -> Option<u64> {
    u64::from(resolution)
        .checked_pow(2)?
        .checked_mul(u64::from(slots))?
        .checked_mul(8)
}

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
        },
        count: None,
    }
}

fn draw_pipeline(
    device: &wgpu::Device,
    textures: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("resident sprite draw"),
        source: wgpu::ShaderSource::Wgsl(
            concat!(
                include_str!("ordered_dither.wgsl"),
                include_str!("sprites.wgsl")
            )
            .into(),
        ),
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("resident sprite draw"),
        bind_group_layouts: &[Some(textures)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("resident sprite draw"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs"),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<Instance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![
                    0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Uint32, 5 => Uint32
                ],
            })],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
mod tests;
