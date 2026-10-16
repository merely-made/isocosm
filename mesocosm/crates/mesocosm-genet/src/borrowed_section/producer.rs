// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cambium_rootstock::{ProducedTexture, ProducerContext, TextureProducer};
use isometer::{FrameRequest, SceneProducer, SceneSignature, SceneSource, SlabCamera};
use crate::section::{self, BodyMode, Framing, Section, SectionFrame};
use super::SceneHandle;

#[cfg(test)]
mod tests;

pub struct SectionProducer { handle: SceneHandle, inner: Option<SceneProducer<Source>> }
impl SectionProducer {
    pub fn new(handle: SceneHandle) -> Self { Self { handle, inner: None } }
    pub(crate) fn renders(&self) -> u64 { self.inner.as_ref().map_or(0, |inner|inner.renders()) }
    pub(crate) fn pick(&mut self, local: (f64, f64), size: (f64, f64)) -> Result<(Option<isometer::Pick>, u64), String> {
        let source = self.inner.as_mut().ok_or("viewport has not rendered")?.source_mut();
        let input = self.handle.snapshot().ok_or("viewport has no current picture")?;
        if source.presented_input != Some(input.fingerprint) { return Err("the pointed picture changed before presentation".into()); }
        let revision = input.revision.ok_or("viewport has no world revision")?;
        if ![local.0,local.1,size.0,size.1].iter().all(|v|v.is_finite())
            || size.0 <= 0.0 || size.1 <= 0.0 || local.0 < 0.0 || local.1 < 0.0
            || local.0 >= size.0 || local.1 >= size.1 { return Err("pointer is outside the viewport".into()); }
        let target = source.target.as_ref().ok_or("viewport has no target")?.3;
        let pixel = [(local.0 / size.0 * target[0] as f64).floor() as u32,
            (local.1 / size.1 * target[1] as f64).floor() as u32];
        let section = source.section.as_mut().ok_or("viewport has no section")?;
        let pick = section.pick_at_pixel(pixel).map_err(|why|format!("{why:?}"))?;
        if let Some(isometer::Pick::Body(body)) = pick {
            if !section.validate_body_pick(body, &input.scene) { return Err("pointed body geometry is stale".into()); }
        }
        Ok((pick, revision))
    }
}
impl TextureProducer for SectionProducer {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        // None from TextureProducer retains its prior image. Once a source
        // exists it must publish an empty frame to withdraw that image.
        if self.inner.is_none() && self.handle.snapshot().is_none() { return None; }
        if self.inner.is_none() {
            let host = section::host_device::from_renderer(cx.core.renderer());
            if &host.device != cx.device || &host.queue != cx.queue {
                *self.handle.error.borrow_mut() = Some("viewport GPU differs from the present core".into());
                return None;
            }
            self.inner = Some(SceneProducer::new(Source::new(host, self.handle.clone())));
        }
        self.inner.as_mut().unwrap().render(cx)
    }
    fn suspend(&mut self) { if let Some(inner) = &mut self.inner { inner.suspend(); } }
    fn retire(&mut self) {
        if let Some(inner) = &mut self.inner { inner.retire(); }
        self.inner = None;
    }
}

pub(crate) struct Source {
    host: tenant::HostDevice, handle: SceneHandle, section: Option<Section>,
    ground: Option<[u64; 4]>, target: Option<(wgpu::Texture, wgpu::TextureView, wgpu::TextureView, [u32; 2])>,
    presented: Option<SlabCamera>,
    presented_input: Option<[u64;4]>,
}

impl Source {
    pub(crate) fn new(host: tenant::HostDevice, handle: SceneHandle) -> Self {
        Self { host, handle, section: None, ground: None, target: None, presented: None, presented_input: None }
    }
    fn target(&mut self, size: [u32; 2]) {
        if self.target.as_ref().is_some_and(|t| t.3 == size) { return; }
        let texture = self.host.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Mesocosm admitted section"), size: wgpu::Extent3d {
                width: size[0], height: size[1], depth_or_array_layers: 1 }, mip_level_count: 1,
            sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let draw = texture.create_view(&Default::default());
        let sample = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm), ..Default::default() });
        self.target = Some((texture, draw, sample, size));
    }
    fn clear(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesocosm section opaque background"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { view, depth_slice: None,
                resolve_target: None, ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store } })],
            depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
        });
    }
    #[cfg(test)]
    pub(crate) fn capture(&self) -> Option<(u32, u32, Vec<u8>)> {
        self.section.as_ref()?.capture_from(&self.target.as_ref()?.0, |_, _, _| {})
    }
    #[cfg(test)]
    pub(crate) fn stats(&self) -> section::BodyFrameStats {
        self.section.as_ref().expect("actual Section was encoded").body_stats()
    }
}

impl SceneSource for Source {
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        let Some(input) = self.handle.snapshot() else {
            return Ok(SceneSignature { size:request.size, host:vec![], ..Default::default() });
        };
        let mut host = input.fingerprint.to_vec();
        host.extend([input.site(), input.played().unwrap_or(u64::MAX), u64::from(input.quarter_turn)]);
        Ok(SceneSignature { size: request.size, camera: Some(input.camera(request.aspect)),
            terrain_revision: Some(input.fingerprint[0]), host, ..Default::default() })
    }
    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        if request.device != &self.host.device || request.queue != &self.host.queue {
            return Err("Mesocosm section cannot migrate onto a foreign GPU".into());
        }
        let size = [request.size[0].max(1), request.size[1].max(1)];
        self.target(size);
        let Some(input) = self.handle.snapshot() else {
            self.section = None; self.ground = None; self.presented = None; self.presented_input = None;
            let mut encoder = self.host.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesocosm withdrawn close-up clears previous image") });
            Self::clear(&mut encoder, &self.target.as_ref().unwrap().1);
            self.host.queue.submit([encoder.finish()]);
            return Ok(Some(self.target.as_ref().unwrap().2.clone()));
        };
        let camera = input.camera(request.aspect);
        let result = (|| {
            if self.section.is_none() {
                self.section = Some(Section::new(&self.host, size[0], size[1], wgpu::TextureFormat::Rgba8UnormSrgb,
                    &input.scene.ground, Framing::default())?);
            }
            let section = self.section.as_mut().unwrap();
            section.resize(size[0], size[1]);
            if self.ground != Some(input.fingerprint) {
                section.set_ground(&input.scene.ground)?;
                self.ground = Some(input.fingerprint);
            }
            section.configure_bodies(BodyMode::Voxels, section::DEFAULT_BODY_BUDGET);
            if let Some(view) = &input.presentation {
                section.configure_bodies(view.bodies, view.budget);
                section.set_half_height(view.half_height);
                section.set_body_light(view.light);
                section.configure_terrain(view.terrain, view.centre, view.centre[1]);
                section.set_body_focus(view.focus, view.selection);
            }
            section.set_forward(Some(camera.forward))?;
            let mut encoder = self.host.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesocosm section on the present device") });
            let draw = &self.target.as_ref().unwrap().1;
            Self::clear(&mut encoder, draw);
            let pose = section::pose_of(&input.scene, 1.0).map(|(pose, _)|pose);
            let roster = section::roster_of(&input.scene, section.slab_window(camera.centre), 1.0);
            section.draw(&mut encoder, draw, SectionFrame { scene: &input.scene, dirty: &[],
                centre: camera.centre, pose: pose.as_ref(), roster: &roster })?;
            self.host.queue.submit([encoder.finish()]);
            self.presented = Some(camera);
            self.presented_input = Some(input.fingerprint);
            self.handle.error.borrow_mut().take();
            Ok::<(), String>(())
        })();
        if let Err(why) = result {
            // A failed encode cannot keep pixels from an earlier native cut.
            self.section = None; self.ground = None; self.presented = None; self.presented_input = None;
            *self.handle.error.borrow_mut() = Some(why);
            let mut encoder = self.host.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Mesocosm refused frame clears previous image") });
            Self::clear(&mut encoder, &self.target.as_ref().unwrap().1);
            self.host.queue.submit([encoder.finish()]);
        }
        Ok(Some(self.target.as_ref().unwrap().2.clone()))
    }
    fn presented_camera(&self) -> Option<SlabCamera> { self.presented }
    fn cached_bodies(&self) -> usize { self.section.as_ref().map_or(0, Section::cached_bodies) }
    fn suspend(&mut self) { self.target = None; self.presented = None; self.presented_input = None; }
    fn retire(&mut self) { self.suspend(); self.section = None; self.ground = None; }
}
