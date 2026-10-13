// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Consuming transfer into the shared host, without another native runtime.

use super::GameState;
use crate::contract::mode::{self, SpatialBindingInput};

pub struct NativeHandoff {
    pub native: isocosm::Session,
    pub game: Vec<u8>,
}

impl GameState {
    pub(crate) fn handoff_products(&self) -> crate::contract::mode::actions::handoff::ProductRecords {
        use crate::contract::mode::actions::handoff::{ProductRecords, ImportedProjectPose,
            MovementRoles, AnatomyLabel};
        let projects = self.project_places.iter().map(|(project, place)| ImportedProjectPose {
            project: *project, site: place.site, created_site: place.created_site,
            created_at: place.created_at, at: place.at,
        }).collect();
        let movement = self.world.subjects().filter_map(|(subject,_)|
            self.movement_profile(subject).map(|profile| MovementRoles {
                subject, profile: profile.clone(),
            })).collect();
        let anatomies = self.world.subjects().filter_map(|(subject,_)|
            self.anatomies.get(subject).map(|record| AnatomyLabel {
                subject, revision: record.revision, species: record.document.species(),
                native_source: record.native_source.clone(),
            })).collect();
        ProductRecords { items: self.places.clone(), projects, movement, anatomies }
    }

    /// Failed admission returns the original owner, including its solver.
    pub fn into_native_mode(self, input: &SpatialBindingInput)
        -> Result<NativeHandoff, (Self, String)> {
        let prior = match self.save() {
            Ok(bytes) => bytes,
            Err(why) => return Err((self, format!("Eponym handoff save: {why:?}"))),
        };
        self.into_native_mode_with_prior(input, prior)
    }

    pub(crate) fn into_native_mode_with_prior(self, input: &SpatialBindingInput, prior: Vec<u8>)
        -> Result<NativeHandoff, (Self, String)> {
        let prepared = self.prepare_native_mode(input, &prior);
        match prepared {
            Err(why) => Err((self, why)),
            Ok(game) => Ok(NativeHandoff { native: self.world.into_native(), game }),
        }
    }

    fn prepare_native_mode(&self, input: &SpatialBindingInput, prior: &[u8]) -> Result<Vec<u8>, String> {
        let subjects: std::collections::BTreeMap<_,_> = self.world.subjects().collect();
        if input.bindings.len() != subjects.len() {
            return Err("handoff requires every existing Eponym subject mapping".into());
        }
        for binding in &input.bindings {
            if subjects.get(&binding.subject) != Some(&binding.entity)
                || self.pose(binding.subject) != Some(binding.pose) {
                return Err("handoff cannot replace an existing subject, entity or accepted pose".into());
            }
            let native = self.world.session();
            let volume = native.sim.volume(binding.site, binding.window.min,
                binding.window.max, binding.window.depth)?;
            let old = self.world.volume();
            if volume.site != old.site || volume.min != old.min || volume.max != old.max
                || volume.datum != old.datum {
                return Err("handoff pose must retain its existing native volume frame".into());
            }
        }
        mode::bind(self.world.session(), prior, input)
    }
}
