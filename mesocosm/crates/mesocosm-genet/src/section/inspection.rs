// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's half of the presentation queries.
//!
//! The queries themselves (the frame receipt and the part walk) are
//! [`isometer::Scene`]'s. What stays here is the product's address, a
//! [`BodySelection`] keyed on the native entity id, and the scene lookup an
//! adapter does before the scene sees a body.

use isometer::core::PartId;
use isometer::mesh::BodyDependencyRevision;
use isometer::{PartAddress, SceneVolumes, SubjectKey};

use super::{Section, SiteScene};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodySelection {
    pub organism: u64,
    pub part: PartId,
    pub revision: BodyDependencyRevision,
}

impl BodySelection {
    fn address(self) -> PartAddress {
        PartAddress {
            subject: SubjectKey(self.organism),
            part: self.part,
            revision: self.revision,
        }
    }

    fn from_address(address: PartAddress) -> Self {
        Self {
            organism: address.subject.0,
            part: address.part,
            revision: address.revision,
        }
    }
}

impl Section {
    pub fn pick_at_pixel(&self, pixel: [u32; 2]) -> Result<Option<isometer::Pick>, isometer::BodyPickError> {
        self.scene.pick_at_pixel(pixel)
    }

    pub fn query_generation(&self) -> Option<u64> { self.scene.query_generation() }

    pub fn validate_body_pick(&mut self, pick: isometer::BodyPick, scene: &SiteScene) -> bool {
        self.query_generation() == Some(pick.frame)
            && self.validate_selection(BodySelection::from_address(pick.address), scene)
    }

    pub(super) fn invalidate_query(&mut self) {
        self.scene.invalidate_query();
    }

    pub fn select_part(
        &self,
        subject: u64,
        current: Option<BodySelection>,
        backwards: bool,
    ) -> Option<BodySelection> {
        if self.body_mode == super::BodyMode::Capsules {
            return None;
        }
        self.scene
            .select_part(
                SubjectKey(subject),
                current.map(BodySelection::address),
                backwards,
            )
            .map(BodySelection::from_address)
    }

    pub fn validate_selection(&mut self, selection: BodySelection, scene: &SiteScene) -> bool {
        if self.body_mode == super::BodyMode::Capsules {
            return false;
        }
        let Some(body) = scene.body(selection.organism) else {
            return false;
        };
        let body = self.host_bodies.scene_body(body, None);
        self.scene.bodies_mut().validate_address(
            selection.address(),
            &body,
            SceneVolumes::DeclaredSolid(&scene.volumes),
        )
    }

    pub fn set_body_focus(&mut self, subject: Option<u64>, selected: Option<BodySelection>) {
        self.scene.set_body_focus(
            subject.map(SubjectKey),
            selected.map(BodySelection::address),
        );
    }
}
