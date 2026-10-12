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
        lots: &[String],
    ) -> Result<Vec<GameEvent>, GameError> {
        self.living(subject)?;
        let actor = self.entity_of(subject)?;
        let selected = project.map(|id| self.project_position_for(subject, id)).transpose()?;
        let (site, at) = selected.unwrap_or((
            self.world.sim().state().population.get(actor)
                .and_then(|body| body.resident_site()).ok_or(WorldError::Refused)?,
            self.at(subject)?,
        ));
        if site != self.world.site() { return Err(WorldError::Refused.into()); }
        if !lots.is_empty() && let Some(target) = target {
            let here = if target == actor { true }
                else if let Some(item) = self.items.get(crate::ItemId(target)) {
                    matches!(item.location, crate::ItemLocation::At(position) if position == at)
                        || matches!(item.location,
                            crate::ItemLocation::Carried(owner) | crate::ItemLocation::Attached { subject:owner, .. }
                            if owner == subject)
                } else {
                    self.world.subjects().find_map(|(subject, entity)| (entity == target).then_some(subject))
                        .and_then(|subject| self.at(subject).ok()).is_some_and(|position| position == at)
                };
            if !here { return Err(WorldError::Refused.into()); }
        }
        let previous: std::collections::BTreeSet<_> =
            self.world.sim().state().projects.records.keys().copied().collect();
        let participant = self.participant.ok_or(WorldError::NotPlayed)?;
        let lowered = self.world.submit(Some(participant), &EponymIntent::Project(ProjectAct {
            subject: EntityHandle(actor),
            process: ActKey(process.into()),
            target: target.map(EntityHandle),
            project: project.map(|project| ProjectHandle(project.0)),
            lots: lots.to_vec(),
        }))?;
        let result = lowered.outcomes.last().ok_or(WorldError::Refused)?;
        let acted: isocosm::projects::Acted =
            serde_json::from_str(result).map_err(|_| WorldError::Decode)?;
        if let Some(id) = acted.project {
            if !self.project_places.contains_key(&id) {
                if previous.contains(&id) { return Err(WorldError::Refused.into()); }
                self.project_places.insert(id, super::project_places::ProjectPlace {
                    site, created_site:site, created_at:at, at,
                });
            } else if project != Some(id) && previous.contains(&id) {
                // A process cannot silently select work whose physical location
                // the caller never checked.
                return Err(WorldError::Refused.into());
            }
            if let Some(output) = acted.output {
                let body = self.world.sim().state().population.get(output).ok_or(WorldError::Refused)?;
                if body.resident_site() != Some(site) { return Err(WorldError::Refused.into()); }
                let place = self.project_places.get_mut(&id).ok_or(WorldError::Refused)?;
                place.site = site;
                place.at = at;
                self.places.at.insert(crate::ItemId(output), at);
            }
        } else if acted.output.is_some() { return Err(WorldError::Refused.into()); }
        Ok(vec![GameEvent::ProjectAct { tick, subject, process:process.into(),
            project:acted.project, output:acted.output.map(crate::ItemId), receipt:acted.receipt }])
    }
}
