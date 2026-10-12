// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::projects::{Custody, ProjectId};
use isocosm_overlay::{EntityHandle, eponym::{EponymIntent, ProjectCarry, ProjectHandle}};
use crate::{GameError, GameEvent, WorldError};
use crate::identity::{SubjectId, Tick};
use super::GameState;

impl GameState {
    pub(super) fn project_carry(&mut self, tick:Tick, subject:SubjectId,
        project:ProjectId, carried:bool) -> Result<Vec<GameEvent>, GameError> {
        self.living(subject)?;
        let (site, at) = self.project_position_for(subject, project)?;
        if carried { self.check_project_capacity(subject, project)?; }
        let actor = self.entity_of(subject)?;
        let participant = self.participant.ok_or(WorldError::NotPlayed)?;
        let lowered = self.world.submit(Some(participant), &EponymIntent::ProjectCarry(ProjectCarry {
            subject:EntityHandle(actor), project:ProjectHandle(project.0), carried,
        }))?;
        let result = lowered.outcomes.last().ok_or(WorldError::Refused)?;
        let custody:Custody = serde_json::from_str(result).map_err(|_| WorldError::Decode)?;
        if custody.project != project || custody.site != site
            || custody.carrier != carried.then_some(actor)
        { return Err(WorldError::Refused.into()); }
        if !carried {
            let place = self.project_places.get_mut(&project).ok_or(WorldError::Refused)?;
            place.site = site;
            place.at = at;
        }
        Ok(vec![GameEvent::ProjectCarried { tick, subject, project, carried, site, at }])
    }
}
