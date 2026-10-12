// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use crate::{Result, schema::*, simulation::{Outcome, Receipt, Simulation}};
use super::ProjectId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Act {
        participant: Id,
        actor: Id,
        target: Option<Id>,
        process: Key,
        project: Option<ProjectId>,
        cause: Option<Key>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acted { pub receipt: Receipt, pub project: Option<ProjectId>, pub output: Option<Id> }

impl Simulation {
    /// Ordinary input selects work; only an accepted native act publishes it.
    pub(crate) fn project_change(&mut self, change: &Change) -> Result<String> {
        let Change::Act { participant, actor, target, process, project, cause } = change;
        if !self.state.population.get(*participant).is_some_and(crate::directing::is_participant)
            || self.plays(*participant) != Some(*actor) {
            return Err("project act requires native control of its actor".into());
        }
        let mut candidate = self.clone();
        if let Some(id) = project {
            let held = candidate.project_at_start(*id).ok_or("project is absent")?;
            candidate.project_eligible(*actor, held)?;
            candidate.state.projects.bindings.insert(*actor, *id);
        } else {
            candidate.state.projects.bindings.remove(actor);
        }
        candidate.project_process_readiness(*actor, *target, process)?;
        let receipt = candidate.execute(*actor, *target, process, cause.clone());
        if !matches!(receipt.outcome, Outcome::Accepted | Outcome::RiskOutcome) {
            return Err(format!("project act was not admitted: {:?}", receipt.outcome));
        }
        let bound = candidate.state.projects.bound(*actor);
        let out = Acted { project: bound.map(|(id, _)| id),
            output: bound.and_then(|(_, p)| p.output), receipt };
        let encoded = serde_json::to_string(&out).map_err(|why| why.to_string())?;
        *self = candidate;
        Ok(encoded)
    }
}
