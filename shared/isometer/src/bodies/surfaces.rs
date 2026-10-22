// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isometer_render::{
    ClipSlab,
    implicit::Surface,
    live_body::{BodyHit, BodyQueryError},
};

impl BodyLayer {
    /// Publish only after the host encoded this exact surface successfully.
    /// Addresses and stale-frame validation still use the native mesh revision.
    pub fn set_surfaces(&mut self, surfaces: BTreeMap<SubjectKey, Surface>) -> Result<(), String> {
        for (subject, surface) in &surfaces {
            if !surface.valid() {
                return Err("invalid implicit surface".into());
            }
            let body = self
                .placed
                .iter()
                .find(|b| b.subject == *subject)
                .ok_or("implicit subject was not drawn")?;
            if surface
                .parts
                .iter()
                .any(|p| !body.mesh.placements.iter().any(|m| m.part.0 == p.meta[1]))
            {
                return Err("implicit surface names an absent part".into());
            }
        }
        self.surfaces = surfaces;
        Ok(())
    }
    pub(super) fn pick_surfaces(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
        clip: ClipSlab,
    ) -> Result<Option<(PartAddress, BodyHit)>, BodyQueryError> {
        let mut best: Option<(PartAddress, BodyHit)> = None;
        for (index, body) in self.placed.iter().enumerate() {
            let hit = if let Some(surface) = self.surfaces.get(&body.subject) {
                surface.pick(origin, direction, far, clip)?
            } else {
                isometer_render::live_body::pick_bodies(
                    &[body.live()],
                    origin,
                    direction,
                    far,
                    Some(clip),
                )?
            };
            if let Some(mut hit) = hit {
                hit.body_index = index;
                let address = PartAddress {
                    subject: body.subject,
                    part: hit.part,
                    revision: body.revision,
                };
                if best.as_ref().is_none_or(|(_, b)| hit.distance < b.distance) {
                    best = Some((address, hit));
                }
            }
        }
        Ok(best)
    }
}
