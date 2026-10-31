// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Accepted in-site locations supplement native project custody (840).

use std::collections::BTreeMap;
use cosmograph::{projects::ProjectId, schema::Id};
use serde::{Deserialize, Serialize};
use crate::{GameError, ItemError, WorldError};
use crate::identity::SubjectId;
use super::GameState;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ProjectPlace {
    pub site: Id,
    pub created_site: Id,
    pub created_at: [i32; 3],
    pub at: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectLocation {
    At { site: Id, at: [i32; 3] },
    Carried { site: Id, at: [i32; 3], subject: SubjectId },
}

impl ProjectLocation {
    pub fn position(self) -> (Id, [i32; 3]) {
        match self { Self::At { site, at } | Self::Carried { site, at, .. } => (site, at) }
    }
}

pub(super) type ProjectPlaces = BTreeMap<ProjectId, ProjectPlace>;

impl GameState {
    /// An absent game binding or travelling/unbound carrier supplies no pose.
    pub fn project_location(&self, id: ProjectId) -> Result<ProjectLocation, GameError> {
        let sim = self.world.sim();
        let project = sim.state().projects.get(id).ok_or(WorldError::Refused)?;
        let place = self.project_places.get(&id).ok_or(WorldError::Refused)?;
        if project.output.is_some() { return Err(WorldError::Refused.into()); }
        let site = sim.project_site(project).ok_or(WorldError::Refused)?;
        if site != self.world.site() { return Err(WorldError::Refused.into()); }
        if let Some(carrier) = project.carrier {
            let subject = self.world.subjects().find_map(|(s, entity)|
                (entity == carrier).then_some(s)).ok_or(WorldError::Refused)?;
            Ok(ProjectLocation::Carried { site, at:self.at(subject)?, subject })
        } else if place.site == site {
            Ok(ProjectLocation::At { site, at:place.at })
        } else { Err(WorldError::Refused.into()) }
    }

    pub(super) fn project_position_for(&self, subject: SubjectId, id: ProjectId)
        -> Result<(Id, [i32; 3]), GameError> {
        let position = self.project_location(id)?.position();
        let actor = self.entity_of(subject)?;
        if self.world.sim().state().population.get(actor).and_then(|body| body.resident_site())
            != Some(position.0) || self.at(subject)? != position.1
        { return Err(WorldError::Refused.into()); }
        Ok(position)
    }

    pub(super) fn project_burden(&self, subject: SubjectId) -> Result<u128, GameError> {
        let actor = self.entity_of(subject)?;
        let sim = self.world.sim();
        sim.state().projects.records.iter().filter(|(_, project)|
            project.output.is_none() && project.carrier == Some(actor))
            .try_fold(0u128, |total, (&id, _)| total.checked_add(
                sim.project_matter(id).map_err(|_| WorldError::Refused)?)
                .ok_or_else(|| GameError::World(WorldError::Refused)))
    }

    /// Whole current burden, with every native project stock counted once.
    pub(super) fn actual_burden(&self, subject: SubjectId) -> Result<u128, GameError> {
        self.items.carried_by(subject).try_fold(self.project_burden(subject)?, |total, item| {
            let mass = self.item_reading(item.id).map_err(|_| WorldError::Refused)?.matter_mass_mg;
            total.checked_add(mass).ok_or_else(|| GameError::World(WorldError::Refused))
        })
    }

    pub(super) fn check_project_capacity(&self, subject: SubjectId, id: ProjectId)
        -> Result<(), GameError> {
        let body = self.bodies.get(subject).ok_or(crate::BodyError::MissingSubject(subject))?;
        let project = self.world.sim().state().projects.get(id).ok_or(WorldError::Refused)?;
        if project.carrier.is_some() { return Err(WorldError::Refused.into()); }
        let attempted_mg = self.actual_burden(subject)?.checked_add(
            self.world.sim().project_matter(id).map_err(|_| WorldError::Refused)?)
            .ok_or(WorldError::Refused)?;
        if attempted_mg > u128::from(body.profile.carry_capacity_mg) {
            return Err(ItemError::NativeOverCapacity {
                capacity_mg:body.profile.carry_capacity_mg, attempted_mg,
            }.into());
        }
        Ok(())
    }
}
