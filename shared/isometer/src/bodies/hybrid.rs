// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::hybrid::SpriteRevisions;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Geometry prepared for the current draw and queries. The image cache owns
/// no body/query state and never substitutes old anatomy into this roster.
pub(crate) struct HybridBody<'a> {
    pub subject: SubjectKey,
    pub live: LiveBody<'a>,
    pub bounds: ([f32; 3], [f32; 3]),
    pub examined: bool,
    body: &'a PlacedBody,
}

impl HybridBody<'_> {
    pub(crate) fn revisions(&self, camera_facing: u64, lighting: u64) -> SpriteRevisions {
        let mut appearance = DefaultHasher::new();
        self.body.tint.map(f32::to_bits).hash(&mut appearance);
        for material in &self.body.materials {
            material.part.0.hash(&mut appearance);
            material.material.hash(&mut appearance);
            material.fraction.to_bits().hash(&mut appearance);
        }
        self.body.palette.hash(&mut appearance);
        let mut facing = DefaultHasher::new();
        camera_facing.hash(&mut facing);
        self.live.yaw_radians.to_bits().hash(&mut facing);
        SpriteRevisions {
            anatomy: self.body.revision,
            appearance: appearance.finish(),
            facing: facing.finish(),
            lighting,
        }
    }
}

impl BodyLayer {
    /// The sun later live draws take; `None` keeps the fixed face shade.
    pub(crate) fn set_light(&mut self, light: Option<BodyLight>) {
        self.renderer.set_light(light);
    }

    pub(crate) fn hybrid_bodies(&self) -> impl Iterator<Item = HybridBody<'_>> {
        self.placed.iter().map(|body| HybridBody {
            subject: body.subject,
            live: body.live(),
            bounds: body.bounds,
            examined: body.always_visible
                || self.focus_subject == Some(body.subject)
                || self
                    .selected
                    .is_some_and(|part| part.subject == body.subject),
            body,
        })
    }
}
