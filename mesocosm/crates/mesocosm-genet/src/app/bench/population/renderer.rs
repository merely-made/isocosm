// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::model::Workload;
use isometer_render::{
    Camera,
    live_body::{LiveBody, LiveBodyRenderer},
};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct RenderStats {
    pub mesh_builds: usize,
    pub mesh_upload_bytes: usize,
    pub instance_upload_bytes: usize,
    pub frame_upload_bytes: usize,
    pub draw_parts: usize,
    pub body_count: usize,
    pub draws: usize,
    pub evictions: usize,
    pub cached_meshes: usize,
    /// Bodies whose conservative posed bounds cross the viewport/clip depth.
    pub clipped_bodies: usize,
    /// Conservative bounds completely beyond a single viewport/depth plane.
    pub fully_outside_bodies: usize,
    pub texture_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub timestamp_queries_enabled: bool,
}

pub struct Renderer {
    renderer: LiveBodyRenderer,
    _colour: wgpu::Texture,
    output: wgpu::TextureView,
    attachment: wgpu::TextureView,
    _depth: wgpu::Texture,
    depth: wgpu::TextureView,
    size: (u32, u32),
    camera: Camera,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        size: (u32, u32),
        workload: &Workload,
    ) -> Result<Self, String> {
        if size.0 == 0 || size.1 == 0 {
            return Err("population viewport must be nonempty".into());
        }
        let limit = device.limits().max_texture_dimension_2d;
        if size.0 > limit || size.1 > limit {
            return Err("population viewport exceeds texture limits".into());
        }
        let colour = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bench population colour"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });
        let output = colour.create_view(&Default::default());
        let attachment = colour.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bench population depth"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let depth = depth_texture.create_view(&Default::default());
        Ok(Self {
            renderer: LiveBodyRenderer::new(
                device,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                workload.config.mesh_capacity,
            ),
            _colour: colour,
            output,
            attachment,
            _depth: depth_texture,
            depth,
            size,
            camera: Camera {
                target: [0.0, 7.0, 0.0],
                extent: workload.config.camera_extent as f32,
                aspect: size.0 as f32 / size.1 as f32,
                ..Default::default()
            },
        })
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.output
    }

    #[cfg(test)]
    pub(super) fn oracle_texture(&self) -> &wgpu::Texture {
        &self._colour
    }
    #[cfg(test)]
    pub(super) fn oracle_depth(&self) -> &wgpu::Texture {
        &self._depth
    }
    #[cfg(test)]
    pub(super) fn oracle_extent(&mut self, extent: f32) {
        self.camera.extent = extent;
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        workload: &Workload,
        yaw: f32,
        tint: [f32; 3],
    ) -> Result<RenderStats, String> {
        if !yaw.is_finite() || tint.iter().any(|v| !v.is_finite() || *v < 0.0) {
            return Err("invalid population appearance".into());
        }
        let mut bodies = Vec::with_capacity(workload.stats.part_instances);
        let ordered: Box<dyn Iterator<Item = _>> = if workload.config.reverse_instances {
            Box::new(workload.instances.iter().rev())
        } else {
            Box::new(workload.instances.iter())
        };
        for instance in ordered {
            for mesh in &workload.designs[instance.design] {
                let mut body = LiveBody::new(mesh, instance.origin);
                body.yaw_radians = yaw;
                body.tint = [0, 1, 2].map(|i| instance.tint[i] * tint[i]);
                bodies.push(body);
            }
        }
        let matrix = self.camera.view_proj();
        let mut clipped_bodies = 0;
        let mut fully_outside_bodies = 0;
        let (s, c) = yaw.sin_cos();
        for instance in &workload.instances {
            let mut clipped = false;
            let mut outside = [true; 6];
            let (min, max) = workload.bounds[instance.design];
            for bits in 0..8 {
                let p = [0, 1, 2].map(|i| if bits & (1 << i) == 0 { min[i] } else { max[i] });
                let rotated = [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c];
                let world = [0, 1, 2].map(|i| rotated[i] + instance.origin[i]);
                let clip = matrix.transform_point3(world.into());
                clipped |=
                    clip.x.abs() > 1.0 || clip.y.abs() > 1.0 || !(0.0..=1.0).contains(&clip.z);
                let planes = [
                    clip.x < -1.,
                    clip.x > 1.,
                    clip.y < -1.,
                    clip.y > 1.,
                    clip.z < 0.,
                    clip.z > 1.,
                ];
                for i in 0..6 {
                    outside[i] &= planes[i];
                }
            }
            clipped_bodies += usize::from(clipped);
            fully_outside_bodies += usize::from(outside.into_iter().any(|v| v));
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("bench population"),
        });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bench population clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.attachment,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.012,
                            g: 0.02,
                            b: 0.028,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth,
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
        let stats = self
            .renderer
            .draw(
                device,
                queue,
                &mut encoder,
                &self.attachment,
                &self.depth,
                matrix.to_cols_array_2d(),
                None,
                &bodies,
            )
            .map_err(|e| format!("population draw: {e:?}"))?;
        queue.submit([encoder.finish()]);
        Ok(RenderStats {
            mesh_builds: stats.mesh_builds,
            mesh_upload_bytes: stats.mesh_upload_bytes,
            instance_upload_bytes: stats.instance_upload_bytes,
            frame_upload_bytes: stats.frame_upload_bytes,
            draw_parts: stats.draw_parts,
            body_count: workload.stats.body_count,
            draws: stats.draws,
            evictions: stats.evictions,
            cached_meshes: stats.cached_meshes,
            clipped_bodies,
            fully_outside_bodies,
            texture_bytes: u64::from(self.size.0) * u64::from(self.size.1) * 8,
            width: self.size.0,
            height: self.size.1,
            timestamp_queries_enabled: device.features().contains(wgpu::Features::TIMESTAMP_QUERY),
        })
    }
}
