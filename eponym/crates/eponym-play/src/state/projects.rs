// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Controlled material work through the native project/process admission.

use super::GameState;
use crate::identity::{SubjectId, Tick};
use crate::{GameError, GameEvent, WorldError};
use isocosm::{projects::ProjectId, schema::Id};
use isocosm_overlay::{ActKey, EntityHandle};
use isocosm_overlay::eponym::{EponymIntent, ProjectAct, ProjectHandle};

impl GameState {
    pub(super) fn project_act(
        &mut self,
        tick: Tick,
        subject: SubjectId,
        process: &str,
        target: Option<Id>,
        project: Option<ProjectId>,
    ) -> Result<Vec<GameEvent>, GameError> {
        self.living(subject)?;
        let actor = self.entity_of(subject)?;
        let participant = self.participant.ok_or(WorldError::NotPlayed)?;
        let lowered = self.world.submit(Some(participant), &EponymIntent::Project(ProjectAct {
            subject: EntityHandle(actor),
            process: ActKey(process.into()),
            target: target.map(EntityHandle),
            project: project.map(|project| ProjectHandle(project.0)),
        }))?;
        let result = lowered.outcomes.last().ok_or(WorldError::Refused)?;
        let acted: isocosm::projects::Acted =
            serde_json::from_str(result).map_err(|_| WorldError::Decode)?;
        Ok(vec![GameEvent::ProjectAct { tick, subject, process:process.into(),
            project:acted.project, output:acted.output.map(crate::ItemId), receipt:acted.receipt }])
    }
}
