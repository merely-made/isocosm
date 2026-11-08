// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The scene as a Cambium document leaf.
//!
//! [`SceneProducer`] carries the two contracts every vessel's viewport shares,
//! and nothing else:
//!
//! 1. **The unchanged-input skip.** A frame whose inputs have not moved is not
//!    re-encoded and advances no generation. What "inputs" means is a
//!    [`SceneSignature`] the source declares before anything is drawn, so the
//!    comparison costs no GPU work and no projection. [`SceneProducer::signature`]
//!    and [`SceneProducer::cached_bodies`] are that skip's evidence, read by
//!    tests in both products.
//! 2. **The output contract.** A scene's display texture holds encoded sRGB
//!    with straight alpha ([`SCENE_ENCODING`], [`SCENE_ALPHA`]). Rootstock's
//!    bridge converts alpha and performs no colour transfer, so declaring
//!    anything else here washes the scene out.
//!
//! A third contract rides on the second: the leaf is handed a texture of
//! exactly its physical size, whatever internal resolution the scene drew at.
//! [`FrameRequest::render_scale`] is the host's pixel grid — the scene draws at
//! the leaf's size divided by the scale, and this producer presents it back up
//! by that scale with a nearest sampler. Scale 1 takes no extra pass at all and
//! is the path that shipped.
//!
//! Everything above those two is the product's: [`SceneSource`] owns its own
//! [`Scene`](crate::Scene), builds its own [`SceneFrame`](crate::SceneFrame)
//! and decides what a frame's pixels are. See the extraction plan §3 and §5
//! step 7.

use cambium_rootstock::{
    ProducedTexture, ProducerContext, SourceAlpha, SourceEncoding, TextureProducer,
};

use crate::bodies::{Pose, SceneBody, SubjectKey};
use crate::camera::SlabCamera;

mod staging;
mod upscale;
use upscale::Upscale;

/// A scene's colour is not premultiplied: the tracer and the body pass both
/// write straight coverage.
pub const SCENE_ALPHA: SourceAlpha = SourceAlpha::Straight;
/// A scene's display twin holds encoded sRGB sampled without hardware decode.
pub const SCENE_ENCODING: SourceEncoding = SourceEncoding::Srgb;

/// One body's contribution to the skip signature.
///
/// `revision` is the host's own geometry revision rather than a projected
/// [`BodyDependencyRevision`](isometer_mesh::BodyDependencyRevision), so
/// comparing a frame costs no projection: a host that changes an anatomy
/// changes this number.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodySignature {
    pub subject: SubjectKey,
    pub revision: u64,
    pub pose: Pose,
    pub scale: f32,
    pub grounded: bool,
    pub tint: [f32; 3],
    pub always_visible: bool,
}

impl BodySignature {
    /// Everything about a scene body that reaches a pixel, with the host's
    /// revision for the document it points at.
    pub fn of(body: &SceneBody<'_>, revision: u64) -> Self {
        Self {
            subject: body.subject,
            revision,
            pose: body.pose,
            scale: body.scale,
            grounded: body.grounded,
            tint: body.tint,
            always_visible: body.always_visible,
        }
    }
}

/// Everything the next frame's pixels depend on. Equality here is the whole
/// skip rule, so an input a host leaves out is an input that will go stale.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneSignature {
    pub size: [u32; 2],
    /// The host's pixel grid, 1 for an unscaled leaf. The producer writes this
    /// from the request after the source answers, so a scale change is a real
    /// frame whether or not a source thought to declare it.
    pub render_scale: u32,
    /// `None` where the source has no camera yet, which never equals a frame
    /// that does.
    pub camera: Option<SlabCamera>,
    /// The terrain source's revision, or `None` for a bodies-only frame.
    pub terrain_revision: Option<u64>,
    pub bodies: Vec<BodySignature>,
    /// Inputs no scene can see: a host's own epoch, palette, tint or mode.
    /// Kept opaque so a product's skip stays exactly its own.
    pub host: Vec<u64>,
}

/// One producer invocation, before a source decides whether to draw it.
#[derive(Clone, Copy)]
pub struct FrameRequest<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    /// The size to draw at. On the request a host or `render` builds this is
    /// the leaf's physical size; on the one a [`SceneSource`] is handed it is
    /// already divided by [`Self::render_scale`], because sizing the scene is
    /// exactly what a source uses it for. See [`Self::for_scene`].
    pub size: [u32; 2],
    /// The leaf's aspect, taken from its physical size at both scales: the
    /// scaled scene frames the same world, and the upscale never stretches it.
    pub aspect: f32,
    /// The leaf's resolved CSS `color`, encoded sRGB with straight alpha, or
    /// `None` where the document declared none. A host tint is its own
    /// business: this crate never interprets it.
    pub color: Option<[f32; 4]>,
    /// No prior image is usable. Set after resize, suspension or a new device,
    /// and it defeats the skip.
    pub needs_frame: bool,
    /// The integer pixel grid the host wants: the scene draws at `size /
    /// render_scale` and the producer presents it back up nearest-neighbour
    /// into a texture of the leaf's physical size. 1 is today's path, and
    /// takes no presentation pass.
    pub render_scale: u32,
}

impl<'a> FrameRequest<'a> {
    /// The context as a request, with aspect taken from the physical size so a
    /// resized leaf stretches nothing. Unscaled: the producer's own
    /// [`SceneProducer::set_render_scale`] is what puts a grid on it.
    pub fn of(cx: &'a ProducerContext<'a>) -> FrameRequest<'a> {
        let size = cx.frame.physical_size;
        FrameRequest {
            device: cx.device,
            queue: cx.queue,
            size,
            aspect: size[0] as f32 / size[1].max(1) as f32,
            color: cx.frame.appearance.color(),
            needs_frame: cx.frame.needs_frame,
            render_scale: 1,
        }
    }

    /// The scale, never zero.
    pub fn scale(&self) -> u32 {
        self.render_scale.max(1)
    }

    /// The same request at the size the scene draws: the leaf's size divided
    /// by the scale, rounded down and never below one pixel. This is what a
    /// [`SceneSource`] sees in `inputs` and `frame`; the aspect it keeps is
    /// still the leaf's, so a floor that loses a row costs no framing.
    pub fn for_scene(&self) -> FrameRequest<'a> {
        let scale = self.scale();
        FrameRequest {
            size: [(self.size[0] / scale).max(1), (self.size[1] / scale).max(1)],
            ..*self
        }
    }
}

/// What a product supplies each frame.
///
/// The source owns its own [`Scene`](crate::Scene) and the policy that fills a
/// [`SceneFrame`](crate::SceneFrame): which bodies are alive, where the camera
/// sits, what the terrain is. What it does not own is the skip — it declares
/// its inputs and is called only when they moved.
pub trait SceneSource {
    /// Presentation still needs another frame even if authoritative inputs
    /// and camera are unchanged, for example during an ordered transition.
    fn needs_animation_frame(&self) -> bool {
        false
    }

    /// Everything the frame this request would draw depends on. Called before
    /// every frame, including the ones that are skipped, so it must not encode
    /// and must not project.
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String>;

    /// Encodes and submits one frame, answering the view to present. `None` is
    /// the source's own finer skip, and neither advances the generation nor
    /// banks the signature.
    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String>;

    /// Deferred hosts require explicit transactional publication support.
    fn supports_native_staging(&self) -> bool {
        false
    }

    /// Encode a candidate without replacing accepted query or image metadata.
    fn stage_frame(
        &mut self,
        _request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        Err("scene source does not support native staging".into())
    }

    /// Freeze metadata for an unchanged image, without encoding it again.
    fn stage_retained(&mut self, _request: &FrameRequest<'_>) -> Result<(), String> {
        Ok(())
    }
    fn publish_staged(&mut self) {}
    fn discard_staged(&mut self) {}
    fn staged_is_live(&self) -> bool {
        true
    }

    /// The camera the last completed frame used, for a caller naming a pixel.
    fn presented_camera(&self) -> Option<SlabCamera> {
        None
    }

    /// Distinct body geometries the source is retaining. The skip test's other
    /// half: a suspension releases targets and keeps this.
    fn cached_bodies(&self) -> usize {
        0
    }

    /// Entering a non-painted or surface-suspended state. Retained geometry
    /// may stay cached.
    fn suspend(&mut self) {}

    /// The registration was removed. Release image and renderer resources.
    fn retire(&mut self) {
        self.suspend();
    }
}

/// A [`SceneSource`] as a Cambium texture producer: the unchanged-input skip
/// and the sRGB / straight-alpha output contract, over any product's scene.
pub struct SceneProducer<S: SceneSource> {
    source: S,
    staged: Option<staging::StagedFrame>,
    signature: SceneSignature,
    /// Whether `signature` describes a frame that actually reached the screen.
    /// A source that has never drawn, or one that was suspended, can never
    /// skip.
    produced: bool,
    presented: Option<SlabCamera>,
    renders: u64,
    error: Option<String>,
    /// The host's pixel grid. 1 presents the source's own view untouched.
    render_scale: u32,
    /// The nearest-neighbour presentation pass and the leaf-sized texture it
    /// writes, built on the first scaled frame and never on an unscaled one.
    upscale: Option<Upscale>,
    display: Option<(wgpu::Texture, wgpu::TextureView, [u32; 2])>,
}

impl<S: SceneSource> SceneProducer<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            staged: None,
            signature: SceneSignature::default(),
            produced: false,
            presented: None,
            renders: 0,
            error: None,
            render_scale: 1,
            upscale: None,
            display: None,
        }
    }

    /// The integer pixel grid this leaf draws on: the scene renders at the
    /// leaf's physical size divided by the scale and is presented back up
    /// nearest-neighbour, so the produced texture is always the leaf's own
    /// size. Zero is read as one, and a change is a real frame.
    pub fn set_render_scale(&mut self, scale: u32) {
        self.render_scale = scale.max(1);
    }

    pub fn render_scale(&self) -> u32 {
        self.render_scale
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Frames produced. This is the `generation` the document sees, and a skip
    /// never advances it.
    pub fn renders(&self) -> u64 {
        self.renders
    }

    /// The camera of the last produced frame. Survives query invalidation, so
    /// a caller can still name the pixel it drew.
    pub fn presented_camera(&self) -> Option<SlabCamera> {
        self.presented
    }

    /// The banked inputs of the last produced frame: the skip's evidence.
    pub fn signature(&self) -> &SceneSignature {
        &self.signature
    }

    /// Distinct body geometries the source retains.
    pub fn cached_bodies(&self) -> usize {
        self.source.cached_bodies()
    }

    pub fn last_error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Draws one frame, or answers `None` when nothing the picture depends on
    /// has moved.
    pub fn render_scene(
        &mut self,
        request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        self.discard_candidate();
        let scene = request.for_scene();
        let mut current = self.source.inputs(&scene)?;
        // The source declared its own inputs; the grid it drew them on is the
        // producer's, so a host that never mentions the scale still redraws
        // when the scale moves under it.
        current.render_scale = request.scale();
        if !request.needs_frame && !self.source.needs_animation_frame()
            && self.produced && self.signature == current {
            return Ok(None);
        }
        let mut view = self.source.frame(&scene)?;
        if request.scale() > 1 {
            if let Some(drawn) = view.clone() {
                view = Some(self.present(request, &drawn)?);
            }
        }
        if view.is_some() {
            self.signature = current;
            self.produced = true;
            self.presented = self.source.presented_camera();
            self.renders = self
                .renders
                .checked_add(1)
                .expect("scene frame generation exhausted");
        }
        Ok(view)
    }

    /// Drops the banked frame without dropping the count, so the next request
    /// draws whatever the source rebuilt.
    fn forget(&mut self) {
        self.signature = SceneSignature::default();
        self.produced = false;
        self.presented = None;
    }

    /// Blows the scene's image up by the request's scale into a texture of the
    /// leaf's physical size, with a nearest sampler, so the leaf still gets the
    /// exact size rootstock's contract demands and no host sampler of any
    /// filter can soften the pixels.
    ///
    /// `scene` is the view the source just submitted: the scene's display twin,
    /// encoded sRGB in an sRGB-tagged format, which this pass decodes on sample
    /// and re-encodes on write, so the bytes that arrive are the bytes that
    /// leave.
    fn present(
        &mut self,
        request: &FrameRequest<'_>,
        scene: &wgpu::TextureView,
    ) -> Result<wgpu::TextureView, String> {
        let size = [request.size[0].max(1), request.size[1].max(1)];
        let upscale = self
            .upscale
            .get_or_insert_with(|| Upscale::new(request.device));
        let display = match self.display.take() {
            Some(display) if display.2 == size => display,
            _ => {
                let (texture, view) = crate::scene::target(
                    request.device,
                    size[0],
                    size[1],
                    crate::scene::CAPTURE_FORMAT,
                    "scene presentation",
                );
                (texture, view, size)
            },
        };
        let mut encoder = request
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("scene presentation"),
            });
        upscale.draw(request.device, &mut encoder, scene, &display.1);
        request.queue.submit([encoder.finish()]);
        let view = display.1.clone();
        self.display = Some(display);
        Ok(view)
    }
}

impl<S: SceneSource> TextureProducer for SceneProducer<S> {
    fn supports_native_staging(&self) -> bool {
        self.source.supports_native_staging()
    }
    fn render_staged(
        &mut self,
        cx: &ProducerContext<'_>,
    ) -> Result<Option<ProducedTexture>, cambium_rootstock::RasterizeFailure> {
        let request = FrameRequest {
            render_scale: self.render_scale,
            ..FrameRequest::of(cx)
        };
        self.render_scene_staged(&request)
            .map(|view| {
                view.map(|view| ProducedTexture {
                    view,
                    generation: self.staged.as_ref().map_or(self.renders, |s| s.generation),
                    alpha: SCENE_ALPHA,
                    encoding: SCENE_ENCODING,
                })
            })
            .map_err(cambium_rootstock::RasterizeFailure::SourceError)
    }
    fn publish_staged(&mut self) {
        self.publish_candidate();
    }
    fn discard_staged(&mut self) {
        self.discard_candidate();
    }
    fn staged_is_live(&self) -> bool {
        self.source.staged_is_live()
    }

    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        let request = FrameRequest {
            render_scale: self.render_scale,
            ..FrameRequest::of(cx)
        };
        match self.render_scene(&request) {
            Ok(view) => {
                self.error = None;
                view.map(|view| ProducedTexture {
                    view,
                    generation: self.renders,
                    alpha: SCENE_ALPHA,
                    encoding: SCENE_ENCODING,
                })
            },
            Err(why) => {
                self.error = Some(why);
                None
            },
        }
    }

    fn suspend(&mut self) {
        self.discard_candidate();
        self.source.suspend();
        self.forget();
    }

    fn retire(&mut self) {
        self.discard_candidate();
        self.source.retire();
        self.forget();
        // The presentation target is this producer's own image resource.
        self.display = None;
        self.upscale = None;
    }
}

/// The wrapper is the source with a skip in front of it: a host reads its own
/// receipts through the producer it registered, without a second handle.
impl<S: SceneSource> std::ops::Deref for SceneProducer<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.source
    }
}

impl<S: SceneSource> std::ops::DerefMut for SceneProducer<S> {
    fn deref_mut(&mut self) -> &mut S {
        &mut self.source
    }
}

#[cfg(test)]
mod tests;
