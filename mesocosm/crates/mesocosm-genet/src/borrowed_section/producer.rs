// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::SceneHandle;
use crate::section::{self, BodyMode, Framing, Section, SectionFrame};
use cambium_rootstock::{ProducedTexture, ProducerContext, TextureProducer};
use isometer::{FrameRequest, SceneProducer, SceneSignature, SceneSource, SlabCamera};
use std::rc::Rc;

mod costs;
mod encoding;
mod preparation;
mod queries;
mod recording;
mod staging;
use staging::{Accepted, Staged, Target};
#[cfg(test)]
mod tests;
pub(crate) use costs::ProducerCosts;

/// Camera, part accents and body-only changes do not evict terrain residency.
/// Paged columns, materials and the actual ground/frame remain dependencies.
fn same_terrain(a: &Rc<section::SiteScene>, b: &Rc<section::SiteScene>) -> bool {
    Rc::ptr_eq(a, b)
        || (a.ground == b.ground
            && a.window.materials == b.window.materials
            && a.window.same_cells(&b.window))
}

pub struct SectionProducer {
    handle: SceneHandle,
    render_scale: u32,
    inner: Option<SceneProducer<Source>>,
    costs: costs::Costs,
}
impl SectionProducer {
    pub fn new(handle: SceneHandle) -> Self {
        Self {
            handle,
            render_scale: 1,
            inner: None,
            costs: Default::default(),
        }
    }
    pub(crate) fn preparation_costs(&self) -> preparation::PreparationCosts {
        self.costs.preparation()
    }
    pub(crate) fn costs(&self) -> ProducerCosts {
        self.costs.snapshot()
    }
    pub(crate) fn cost_totals(&self) -> (u64, u64, u64) {
        self.costs.totals()
    }
    pub(crate) fn cost_status(&self) -> (bool, bool) {
        let current = self.has_current_frame();
        let valid = self.handle.last_error().is_none()
            && self
                .inner
                .as_ref()
                .is_none_or(|inner| inner.last_error().is_none())
            && (self.handle.snapshot().is_none() || current);
        (
            valid,
            self.accepted().is_some_and(|(frame, _)| frame.populated),
        )
    }
    pub(crate) fn set_render_scale(&mut self, scale: u32) {
        self.render_scale = scale.clamp(1, 8);
        if let Some(inner) = &mut self.inner {
            inner.set_render_scale(self.render_scale);
        }
    }
    pub(crate) fn renders(&self) -> u64 {
        self.inner.as_ref().map_or(0, |inner| inner.renders())
    }
}

impl SectionProducer {
    fn ensure_inner(&mut self, cx: &ProducerContext<'_>) -> Result<bool, String> {
        // None from TextureProducer retains its prior image. Once a source
        // exists it must publish an empty frame to withdraw that image.
        if self.inner.is_none() && self.handle.snapshot().is_none() {
            return Ok(false);
        }
        if self.inner.is_none() {
            let host = section::host_device::from_renderer(cx.core.renderer());
            if &host.device != cx.device || &host.queue != cx.queue {
                *self.handle.error.borrow_mut() =
                    Some("viewport GPU differs from the present core".into());
                return Err("viewport GPU differs from the present core".into());
            }
            let mut inner = SceneProducer::new(Source::with_costs(
                host,
                self.handle.clone(),
                self.costs.clone(),
            ));
            inner.set_render_scale(self.render_scale);
            self.inner = Some(inner);
        }
        Ok(true)
    }
}
impl TextureProducer for SectionProducer {
    fn supports_native_staging(&self) -> bool {
        true
    }
    fn render_staged(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, cambium_rootstock::RasterizeFailure> {
        if !self
            .ensure_inner(cx)
            .map_err(cambium_rootstock::RasterizeFailure::SourceError)?
        {
            return Ok(None);
        }
        self.inner.as_mut().unwrap().render_staged(cx)
    }
    fn publish_staged(&mut self) {
        if let Some(inner) = &mut self.inner {
            inner.publish_staged();
        }
    }
    fn discard_staged(&mut self) {
        if let Some(inner) = &mut self.inner {
            inner.discard_staged();
        }
    }
    fn staged_is_live(&self) -> bool {
        self.inner
            .as_ref()
            .is_none_or(TextureProducer::staged_is_live)
    }
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        if !self.ensure_inner(cx).ok()? {
            return None;
        }
        self.inner.as_mut().unwrap().render(cx)
    }
    fn suspend(&mut self) {
        if let Some(inner) = &mut self.inner {
            inner.suspend();
        }
    }
    fn retire(&mut self) {
        if let Some(inner) = &mut self.inner {
            inner.retire();
        }
        self.inner = None;
    }
}

pub(crate) struct Source {
    host: tenant::HostDevice,
    handle: SceneHandle,
    section: Option<Section>,
    ground: Option<Rc<section::SiteScene>>,
    target: Option<Target>,
    accepted: Option<Accepted>,
    staged: Option<Staged>,
    transactional: bool,
    presented: Option<SlabCamera>,
    presented_input: Option<[u64; 4]>,
    animation_frame: u64,
    recording: recording::Recording,
    costs: costs::Costs,
    /// The hybrid lighting revision of the last encoded frame, to count changes.
    light_stamp: u64,
}

impl Source {
    #[cfg(test)]
    pub(crate) fn new(host: tenant::HostDevice, handle: SceneHandle) -> Self {
        Self::with_costs(host, handle, Default::default())
    }
    fn with_costs(host: tenant::HostDevice, handle: SceneHandle, costs: costs::Costs) -> Self {
        Self {
            host,
            handle,
            section: None,
            ground: None,
            target: None,
            accepted: None,
            staged: None,
            transactional: false,
            presented: None,
            presented_input: None,
            animation_frame: 0,
            recording: Default::default(),
            costs,
            light_stamp: 0,
        }
    }
    fn target(&mut self, size: [u32; 2]) {
        if self.target.as_ref().is_some_and(|t| t.3 == size) {
            return;
        }
        let texture = self.host.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Mesocosm admitted section"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let draw = texture.create_view(&Default::default());
        let sample = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });
        self.costs.count("target_textures_created", 1);
        self.costs.count("target_views_created", 2);
        self.costs.count(
            "target_texels_created",
            u64::from(size[0]) * u64::from(size[1]),
        );
        self.costs.target_extent(size);
        self.target = Some((texture, draw, sample, size));
    }
    fn clear(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesocosm section opaque background"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    #[cfg(test)]
    pub(crate) fn capture(&self) -> Option<(u32, u32, Vec<u8>)> {
        self.section
            .as_ref()?
            .capture_from(&self.target.as_ref()?.0, |_, _, _| {})
    }
    #[cfg(test)]
    pub(crate) fn stats(&self) -> section::BodyFrameStats {
        self.section
            .as_ref()
            .expect("actual Section was encoded")
            .body_stats()
    }
}

impl SceneSource for Source {
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String> {
        let Some(input) = self.handle.snapshot() else {
            return Ok(SceneSignature {
                size: request.size,
                host: vec![],
                ..Default::default()
            });
        };
        let mut host = input.fingerprint.to_vec();
        if self.section.as_ref().is_some_and(Section::animating) {
            self.animation_frame = self.animation_frame.wrapping_add(1);
        }
        host.push(self.animation_frame);
        host.extend([
            input.site(),
            input.played().unwrap_or(u64::MAX),
            u64::from(input.quarter_turn),
        ]);
        Ok(SceneSignature {
            size: request.size,
            camera: Some(input.camera(request.aspect)),
            terrain_revision: Some(input.scene.ground.revision()),
            host,
            ..Default::default()
        })
    }
    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String> {
        self.discard_staged();
        self.transactional = false;
        self.costs.count("legacy_frame_calls", 1);
        self.encode_frame(request, false)
    }
    fn supports_native_staging(&self) -> bool {
        true
    }
    fn stage_frame(
        &mut self,
        request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        self.stage_candidate(request)
    }
    fn stage_retained(&mut self, _request: &FrameRequest<'_>) -> Result<(), String> {
        self.stage_unchanged()
    }
    fn publish_staged(&mut self) {
        self.publish_candidate();
    }
    fn discard_staged(&mut self) {
        if self.staged.is_some() {
            self.costs.count("explicit_staged_discards", 1);
        }
        self.staged = None;
    }
    fn staged_is_live(&self) -> bool {
        self.staged
            .as_ref()
            .is_none_or(|stage| self.handle.is_lifetime(stage.lifetime))
    }
    fn presented_camera(&self) -> Option<SlabCamera> {
        self.presented
    }
    fn cached_bodies(&self) -> usize {
        self.section.as_ref().map_or(0, Section::cached_bodies)
    }
    fn suspend(&mut self) {
        self.staged = None;
        self.accepted = None;
        self.target = None;
        self.presented = None;
        self.presented_input = None;
    }
    fn retire(&mut self) {
        self.suspend();
        self.section = None;
        self.ground = None;
    }
}
