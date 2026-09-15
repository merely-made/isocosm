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
//! Everything above those two is the product's: [`SceneSource`] owns its own
//! [`Scene`](crate::Scene), builds its own [`SceneFrame`](crate::SceneFrame)
//! and decides what a frame's pixels are. See the extraction plan §3 and §5
//! step 7.

use cambium_rootstock::{
    ProducedTexture, ProducerContext, SourceAlpha, SourceEncoding, TextureProducer,
};

use crate::bodies::{Pose, SceneBody, SubjectKey};
use crate::camera::SlabCamera;

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
pub struct FrameRequest<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub size: [u32; 2],
    pub aspect: f32,
    /// The leaf's resolved CSS `color`, encoded sRGB with straight alpha, or
    /// `None` where the document declared none. A host tint is its own
    /// business: this crate never interprets it.
    pub color: Option<[f32; 4]>,
    /// No prior image is usable. Set after resize, suspension or a new device,
    /// and it defeats the skip.
    pub needs_frame: bool,
}

impl FrameRequest<'_> {
    /// The context as a request, with aspect taken from the physical size so a
    /// resized leaf stretches nothing.
    pub fn of<'a>(cx: &'a ProducerContext<'a>) -> FrameRequest<'a> {
        let size = cx.frame.physical_size;
        FrameRequest {
            device: cx.device,
            queue: cx.queue,
            size,
            aspect: size[0] as f32 / size[1].max(1) as f32,
            color: cx.frame.appearance.color(),
            needs_frame: cx.frame.needs_frame,
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
    /// Everything the frame this request would draw depends on. Called before
    /// every frame, including the ones that are skipped, so it must not encode
    /// and must not project.
    fn inputs(&mut self, request: &FrameRequest<'_>) -> Result<SceneSignature, String>;

    /// Encodes and submits one frame, answering the view to present. `None` is
    /// the source's own finer skip, and neither advances the generation nor
    /// banks the signature.
    fn frame(&mut self, request: &FrameRequest<'_>) -> Result<Option<wgpu::TextureView>, String>;

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
    signature: SceneSignature,
    /// Whether `signature` describes a frame that actually reached the screen.
    /// A source that has never drawn, or one that was suspended, can never
    /// skip.
    produced: bool,
    presented: Option<SlabCamera>,
    renders: u64,
    error: Option<String>,
}

impl<S: SceneSource> SceneProducer<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            signature: SceneSignature::default(),
            produced: false,
            presented: None,
            renders: 0,
            error: None,
        }
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
        let current = self.source.inputs(request)?;
        if !request.needs_frame && self.produced && self.signature == current {
            return Ok(None);
        }
        let view = self.source.frame(request)?;
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
}

impl<S: SceneSource> TextureProducer for SceneProducer<S> {
    fn render(&mut self, cx: &ProducerContext<'_>) -> Option<ProducedTexture> {
        match self.render_scene(&FrameRequest::of(cx)) {
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
        self.source.suspend();
        self.forget();
    }

    fn retire(&mut self) {
        self.source.retire();
        self.forget();
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
