// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::projects::{Consented, Owner, ProjectId};
use isocosm_overlay::{ActKey, EntityHandle, FactionHandle,
    eponym::{EponymIntent, ProjectConsent, ProjectHandle, ProjectOwner}};
use crate::{GameError, GameEvent, WorldError};
use crate::identity::{SubjectId, Tick};
use super::GameState;

impl GameState {
    pub(super) fn project_consent(&mut self, tick:Tick, subject:SubjectId,
        project:ProjectId, owner:Owner, worker:SubjectId, grant:bool, process:Option<&str>)
        -> Result<Vec<GameEvent>, GameError> {
        self.living(subject)?;
        let actor = self.entity_of(subject)?;
        let worker_id = self.entity_of(worker)?;
        let participant = self.participant.ok_or(WorldError::NotPlayed)?;
        let owner = match owner {
            Owner::Entity(id) => ProjectOwner::Entity(EntityHandle(id)),
            Owner::Polity(id) => ProjectOwner::Polity(FactionHandle(id)),
        };
        let lowered = self.world.submit(Some(participant), &EponymIntent::ProjectConsent(ProjectConsent {
            subject:EntityHandle(actor), project:ProjectHandle(project.0), owner,
            worker:EntityHandle(worker_id), grant, process:process.map(|key| ActKey(key.into())),
        }))?;
        let result = lowered.outcomes.last().ok_or(WorldError::Refused)?;
        let consent:Consented = serde_json::from_str(result).map_err(|_| WorldError::Decode)?;
        if consent.project != project || consent.worker != worker_id {
            return Err(WorldError::Refused.into());
        }
        Ok(vec![GameEvent::ProjectConsented { tick, subject, project, worker,
            authorized:consent.authorized, round:consent.round }])
    }
}
