// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use isometer_render::implicit::Surface;

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
}
