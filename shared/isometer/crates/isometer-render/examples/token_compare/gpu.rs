// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! GPU resources and timestamp/readback helpers for the representation probe.
use isometer_mesh::bake::Sheet;

pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct Gpu {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub info: wgpu::AdapterInfo,
}

impl Gpu {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            apply_limit_buckets: false,
            compatible_surface: None,
        }))
        .expect("a real GPU adapter is required");
        let info = adapter.get_info();
        let required_features =
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        assert!(
            adapter.features().contains(required_features),
            "GPU timestamps are required"
        );
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("token comparison"),
            required_features,
            ..Default::default()
        }))
        .expect("comparison device");
        Self {
            device,
            queue,
            info,
        }
    }

    pub fn encoder(&self) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("token probe"),
            })
    }

    pub fn wait(&self) {
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("GPU completion");
    }

    pub fn read(&self, texture: &wgpu::Texture, layer: u32) -> Sheet {
        let (width, height) = (texture.width(), texture.height());
        let stride = (width * 4).div_ceil(256) * 256;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("capture"),
            size: (stride * height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.encoder();
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: layer,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit([encoder.finish()]);
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.wait();
        let bytes = slice.get_mapped_range().expect("mapped capture");
        let rgba = bytes
            .chunks_exact(stride as usize)
            .flat_map(|r| r[..width as usize * 4].iter().copied())
            .collect();
        drop(bytes);
        buffer.unmap();
        Sheet {
            w: width as i32,
            h: height as i32,
            rgba,
        }
    }
}

pub struct Target {
    pub colour: wgpu::Texture,
    pub depth: wgpu::Texture,
    pub width: u32,
    pub height: u32,
}

impl Target {
    pub fn new(gpu: &Gpu, width: u32, height: u32, layers: u32) -> Self {
        let make = |format| {
            gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("comparison target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        Self {
            colour: make(FORMAT),
            depth: make(DEPTH),
            width,
            height,
        }
    }

    pub fn views(&self, layer: u32) -> (wgpu::TextureView, wgpu::TextureView) {
        let desc = wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: layer,
            array_layer_count: Some(1),
            ..Default::default()
        };
        (
            self.colour.create_view(&desc),
            self.depth.create_view(&desc),
        )
    }
}

pub fn clear(
    encoder: &mut wgpu::CommandEncoder,
    colour: &wgpu::TextureView,
    depth: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("comparison clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: colour,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth,
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
}

pub struct Timer {
    pub queries: wgpu::QuerySet,
    resolve: wgpu::Buffer,
    read: wgpu::Buffer,
    samples: u32,
}

impl Timer {
    pub fn new(gpu: &Gpu, samples: u32) -> Self {
        let queries = gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("frame timestamps"),
            ty: wgpu::QueryType::Timestamp,
            count: samples * 2,
        });
        let make = |usage| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("timestamp buffer"),
                size: samples as u64 * 16,
                usage,
                mapped_at_creation: false,
            })
        };
        Self {
            queries,
            resolve: make(wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC),
            read: make(wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST),
            samples,
        }
    }

    pub fn finish(&self, gpu: &Gpu) -> Vec<f64> {
        let mut encoder = gpu.encoder();
        encoder.resolve_query_set(&self.queries, 0..self.samples * 2, &self.resolve, 0);
        encoder.copy_buffer_to_buffer(&self.resolve, 0, &self.read, 0, self.samples as u64 * 16);
        gpu.queue.submit([encoder.finish()]);
        self.read.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        gpu.wait();
        let bytes = self
            .read
            .slice(..)
            .get_mapped_range()
            .expect("timestamp map");
        let ticks: &[u64] = bytemuck::cast_slice(&bytes);
        let us = ticks
            .chunks_exact(2)
            .map(|v| {
                v[1].saturating_sub(v[0]) as f64 * gpu.queue.get_timestamp_period() as f64 / 1000.0
            })
            .collect();
        drop(bytes);
        self.read.unmap();
        us
    }
}
