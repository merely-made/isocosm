// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

pub(super) type Target = (
    wgpu::Texture,
    wgpu::TextureView,
    wgpu::TextureView,
    [u32; 2],
);

#[derive(Clone)]
pub(super) struct Accepted {
    pub input: Rc<super::super::Snapshot>,
    pub queries: Option<isometer::QuerySnapshot>,
    pub body: section::BodyFrameStats,
    pub populated: bool,
    pub terrain: isometer::ResidencyStats,
    pub lifetime: u64,
}
impl Accepted {
    pub(super) fn capture(
        section: &Section,
        input: Rc<super::super::Snapshot>,
        lifetime: u64,
        populated: bool,
    ) -> Self {
        let queries = if input
            .presentation
            .as_ref()
            .is_some_and(|p| p.bodies == BodyMode::Capsules)
        {
            None
        } else {
            section.query_snapshot()
        };
        Self {
            input,
            queries,
            body: section.body_stats(),
            populated,
            terrain: section.terrain_stats(),
            lifetime,
        }
    }
}
pub(super) struct Staged {
    accepted: Option<Accepted>,
    target: Option<Target>,
    camera: Option<SlabCamera>,
    fingerprint: Option<[u64; 4]>,
    pub lifetime: u64,
    was_animating: bool,
}

impl Source {
    pub(super) fn stage_candidate(
        &mut self,
        request: &FrameRequest<'_>,
    ) -> Result<Option<wgpu::TextureView>, String> {
        self.costs.count("native_stage_calls", 1);
        self.staged = None;
        self.transactional = true;
        let lifetime = self.handle.lifetime();
        let accepted = self.accepted.take();
        let target = self.target.take();
        let camera = self.presented.take();
        let fingerprint = self.presented_input.take();
        // Fresh candidate target: no writes to the image still displayed.
        let was_animating = self.section.as_ref().is_some_and(Section::animating);
        let result = self.encode_frame(request, true);
        let candidate = Staged {
            accepted: self.accepted.take(),
            target: self.target.take(),
            camera: self.presented.take(),
            fingerprint: self.presented_input.take(),
            lifetime,
            was_animating,
        };
        self.accepted = accepted;
        self.target = target;
        self.presented = camera;
        self.presented_input = fingerprint;
        if result.is_ok() {
            self.costs.count("staged_candidates_ready", 1);
            self.staged = Some(candidate);
        } else {
            self.costs.count("staged_candidates_refused", 1);
        }
        result
    }
    pub(super) fn stage_unchanged(&mut self) -> Result<(), String> {
        self.transactional = true;
        self.costs.count("retained_stage_calls", 1);
        let mut accepted = self
            .costs
            .measure("retained_query_clone", || self.accepted.clone());
        if accepted
            .as_ref()
            .is_some_and(|frame| frame.queries.is_some())
        {
            self.costs.count("retained_query_arc_clones", 1);
        }
        if let Some(frame) = &mut accepted {
            let input = self
                .handle
                .snapshot()
                .ok_or("retained scene input was withdrawn")?;
            if input.fingerprint != frame.input.fingerprint
                || !self.handle.is_lifetime(frame.lifetime)
            {
                return Err("retained scene source changed".into());
            }
            frame.input = input;
        } else if self.handle.snapshot().is_some() {
            return Err("retained scene has no accepted queries".into());
        }
        self.staged = Some(Staged {
            accepted,
            target: None,
            camera: self.presented,
            fingerprint: self.presented_input,
            lifetime: self.handle.lifetime(),
            was_animating: false,
        });
        Ok(())
    }
    pub(super) fn publish_candidate(&mut self) {
        let Some(stage) = self.staged.take() else {
            return;
        };
        if !self.handle.is_lifetime(stage.lifetime) {
            return;
        }
        if let (Some(section), Some(target), Some(frame)) =
            (&self.section, &stage.target, &stage.accepted)
        {
            if let Err(why) =
                self.recording
                    .frame(section, &target.0, &frame.input, stage.was_animating)
            {
                *self.handle.error.borrow_mut() = Some(why);
            }
        }
        self.costs.count("staged_candidates_published", 1);
        self.accepted = stage.accepted;
        if let Some(target) = stage.target {
            self.target = Some(target);
        }
        self.presented = stage.camera;
        self.presented_input = stage.fingerprint;
    }
}
