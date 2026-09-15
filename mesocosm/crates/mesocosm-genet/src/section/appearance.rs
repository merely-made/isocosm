// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host appearance overrides. Organism and material facts stay in the world.

use isometer::render::live_body::LiveBodyError;
use mesocosm_core::{Organism, OrganismId};

use super::HostBodies;
use crate::section::Section;

impl Section {
    /// Overrides one organism's linear RGB instance multiplier. Process marks
    /// retain their existing material mix; focus and selection keep their accents.
    /// `None` restores the organism's ordinary presentation tint. Values must
    /// be finite and in [0, 1], with display-to-linear conversion owned by the host.
    ///
    /// Returns whether the override changed. An unchanged or invalid request
    /// preserves the completed frame receipt; a change requires a fresh draw.
    pub fn set_body_tint(
        &mut self,
        subject: OrganismId,
        tint: Option<[f32; 3]>,
    ) -> Result<bool, LiveBodyError> {
        if tint.is_some_and(|rgb| {
            rgb.into_iter()
                .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        }) {
            return Err(LiveBodyError::InvalidBody);
        }
        if self.body_tint(subject) == tint {
            return Ok(false);
        }
        match tint {
            Some(rgb) => {
                self.host_bodies.tints.insert(subject, rgb);
            },
            None => {
                self.host_bodies.tints.remove(&subject);
            },
        }
        self.invalidate_query();
        Ok(true)
    }

    /// The host override, or `None` while the ordinary organism tint is used.
    pub fn body_tint(&self, subject: OrganismId) -> Option<[f32; 3]> {
        self.host_bodies.tints.get(&subject).copied()
    }
}

impl HostBodies {
    pub(super) fn rendered_tint(&self, organism: &Organism) -> [f32; 3] {
        self.tints
            .get(&organism.id)
            .copied()
            .unwrap_or_else(|| crate::app::look_of(organism).0)
    }
}

#[cfg(test)]
#[path = "appearance_tests.rs"]
mod tests;
