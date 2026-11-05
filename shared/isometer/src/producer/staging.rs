// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) struct StagedFrame {
    signature: Option<SceneSignature>,
    pub(super) generation: u64,
    display: Option<(wgpu::Texture, wgpu::TextureView, [u32; 2])>,
}

impl<S: SceneSource> SceneProducer<S> {
    /// Prepare a candidate. The host must either publish or discard it; all
    /// public generation/signature/camera reads remain the accepted frame.
    pub fn render_scene_staged(
        &mut self,
        request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        self.discard_candidate();
        if !self.source.supports_native_staging() {
            return Err("scene source does not support native staging".into());
        }
        let scene = request.for_scene();
        let mut current = self.source.inputs(&scene)?;
        current.render_scale = request.scale();
        if !request.needs_frame && self.produced && self.signature == current {
            if let Err(why) = self.source.stage_retained(&scene) {
                self.source.discard_staged();
                return Err(why);
            }
            self.staged = Some(StagedFrame {
                signature: None,
                generation: self.renders,
                display: None,
            });
            return Ok(None);
        }
        let result = (|| {
            let Some(mut view) = self.source.stage_frame(&scene)? else {
                self.source.discard_staged();
                return Ok(None);
            };
            // An in-flight upscale must not overwrite the accepted texture.
            let mut display = None;
            if request.scale() > 1 {
                let accepted = self.display.take();
                let scaled = self.present(request, &view);
                display = self.display.take();
                self.display = accepted;
                view = scaled?;
            }
            let generation = self
                .renders
                .checked_add(1)
                .ok_or("scene frame generation exhausted")?;
            self.staged = Some(StagedFrame {
                signature: Some(current),
                generation,
                display,
            });
            Ok(Some(view))
        })();
        if result.is_err() {
            self.discard_candidate();
        }
        result
    }

    pub(super) fn publish_candidate(&mut self) {
        let Some(staged) = self.staged.take() else {
            return;
        };
        if !self.source.staged_is_live() {
            self.source.discard_staged();
            return;
        }
        self.source.publish_staged();
        if let Some(signature) = staged.signature {
            self.signature = signature;
            self.produced = true;
            self.renders = staged.generation;
            self.display = staged.display;
        }
        self.presented = self.source.presented_camera();
        self.error = None;
    }

    pub(super) fn discard_candidate(&mut self) {
        self.staged = None;
        self.source.discard_staged();
    }
}
