// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Scene, SceneFrame, SceneHost};

mod hybrid;
#[cfg(test)]
mod tests;
pub(super) use hybrid::HybridFrame;

impl Scene {
    pub fn hybrid_stats(&self) -> crate::hybrid::HybridStats {
        self.hybrid.stats
    }

    pub fn hybrid_needs_frame(&self) -> bool {
        self.hybrid.needs_frame
    }

    /// Projects, culls and rasters the frame's bodies, clearing the shared
    /// attachments first. Returns the `clip_from_world` the terrain join
    /// reuses, which is the camera's whether the draw succeeded or not.
    pub(super) fn draw_bodies(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: &SceneFrame<'_>,
        host: &mut dyn SceneHost,
    ) -> Result<[[f32; 4]; 4], String> {
        let camera = frame.camera;
        let window = camera.window();
        host.begin();
        let Self { bodies, .. } = &mut *self;
        bodies.budget = frame.body_budget.max(1);
        bodies.prepare(frame.bodies, frame.volumes, window, |body, stats| {
            host.fallback(body, stats)
        });
        host.prepared(&mut self.bodies);
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear shared scene attachments"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.traced_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.bodies.depth_view,
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
        let matrix = camera.clip_from_world();
        if host.uses_body_tenant() {
            self.hybrid.inactive(&self.queue);
            return Ok(matrix);
        }
        let Self {
            device,
            queue,
            bodies,
            traced_view,
            hybrid,
            width,
            height,
            ..
        } = &mut *self;
        let settings = host.hybrid_settings();
        // One light for the live draw and every bake this frame; quantized
        // under hybrid so images rebake only on a visible change (1170).
        let light = host
            .body_light()
            .map(|light| settings.map_or(light, |s| light.quantized(s.light_step)));
        bodies.set_light(light);
        let styles = bodies
            .drawn()
            .map(|subject| (subject, host.body_display(subject)))
            .collect();
        bodies.set_styles(styles);
        let result = if let Some(settings) = settings {
            let coverage = hybrid.draw(
                device,
                queue,
                encoder,
                traced_view,
                bodies,
                camera,
                [*width, *height],
                host,
                settings,
                light,
            );
            bodies.draw_hybrid(device, queue, encoder, traced_view, camera, &coverage)
        } else {
            hybrid.inactive(queue);
            bodies.draw(device, queue, encoder, traced_view, camera)
        };
        if let Err(error) = result {
            if settings.is_some() {
                // The encoder may already contain sprite draws. Withdraw the
                // entire frame rather than publish a mismatched query roster.
                return Err(error);
            }
            eprintln!("{error}; using capsule fallback");
            bodies.stats.last_error = Some(error);
            for subject in bodies.take_drawn() {
                if let Some(body) = frame.bodies.iter().find(|body| body.subject == subject) {
                    host.fallback(body, &mut bodies.stats);
                }
            }
        }
        Ok(matrix)
    }
}
