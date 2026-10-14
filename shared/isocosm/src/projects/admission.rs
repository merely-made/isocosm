// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use crate::{Result, schema::*, simulation::{Outcome, Receipt, Simulation}};
use super::{Owner, ProjectId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Act {
        participant: Id,
        actor: Id,
        target: Option<Id>,
        process: Key,
        project: Option<ProjectId>,
        cause: Option<Key>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        lots: Vec<Key>,
    },
    Carry { participant:Id, actor:Id, project:ProjectId, carried:bool },
    Consent { participant:Id, actor:Id, project:ProjectId, owner:Owner, worker:Id,
        grant:bool, process:Option<Key>, cause:Option<Key> },
    ActOwned {
        participant: Id, actor: Id, target: Option<Id>, process: Key,
        project: Option<ProjectId>, cause: Option<Key>, lots: Vec<Key>,
        selections: Vec<super::DebitSelection>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Acted { pub receipt: Receipt, pub project: Option<ProjectId>, pub output: Option<Id> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnedProjectActed { pub acted: Acted, pub debits: Vec<super::DebitRequest>, pub claims: Vec<Key> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Custody { pub project:ProjectId, pub carrier:Option<Id>, pub site:Id, pub matter:u128 }

impl Simulation {
    /// Ordinary input selects work; only an accepted native act publishes it.
    pub(crate) fn project_change(&mut self, change: &Change) -> Result<String> {
        match change {
            Change::Act { participant,actor,target,process,project,cause,lots } =>
                self.act_project(*participant,*actor,*target,process,*project,cause.clone(),lots),
            Change::Carry { participant,actor,project,carried } => self.carry_project(*participant,*actor,*project,*carried),
            Change::Consent { participant,actor,project,owner,worker,grant,process,cause } =>
                self.consent_project(*participant,*actor,*project,*owner,*worker,*grant,process.as_deref(),cause.clone()),
            Change::ActOwned { participant,actor,target,process,project,cause,lots,selections } =>
                self.act_project_owned(*participant,*actor,*target,process,*project,cause.clone(),lots,selections),
        }
    }
    pub(crate) fn project_control(&self, participant: Id, actor: Id) -> Result<()> {
        if !self.state.population.get(participant).is_some_and(crate::directing::is_participant)
            || self.plays(participant) != Some(actor) {
            return Err("project act requires native control of its actor".into());
        }
        Ok(())
    }
    fn project_candidate(&self, participant:Id, actor:Id, target:Option<Id>, process:&str,
        project:Option<ProjectId>, lots:&[Key]) -> Result<Self> {
        self.project_control(participant, actor)?;
        if project.is_some() && !lots.is_empty() { return Err("work admission does not reserve new stock lots".into()); }
        let mut candidate = self.clone();
        if let Some(id) = project {
            let held = candidate.project_at_start(id).ok_or("project is absent")?;
            candidate.project_eligible(actor, held)?;
            candidate.state.projects.bindings.insert(actor, id);
        } else {
            candidate.state.projects.bindings.remove(&actor);
        }
        candidate.project_inputs = Some((actor,lots.to_vec()));
        candidate.project_process_readiness(actor, target, process)?;
        Ok(candidate)
    }
    fn project_result(&self, actor:Id, receipt:Receipt) -> Acted {
        let bound = self.state.projects.bound(actor);
        Acted { project: bound.map(|(id, _)| id), output: bound.and_then(|(_, p)| p.output), receipt }
    }
    fn act_project(&mut self, participant:Id, actor:Id, target:Option<Id>, process:&str,
        project:Option<ProjectId>, cause:Option<Key>, lots:&[Key]) -> Result<String> {
        let mut candidate = self.project_candidate(participant,actor,target,process,project,lots)?;
        let receipt = candidate.execute(actor, target, process, cause);
        candidate.project_inputs = None;
        if !matches!(receipt.outcome, Outcome::Accepted | Outcome::RiskOutcome) {
            return Err(format!("project act was not admitted: {:?}", receipt.outcome));
        }
        let out = candidate.project_result(actor,receipt);
        let encoded = serde_json::to_string(&out).map_err(|why| why.to_string())?;
        *self = candidate;
        Ok(encoded)
    }
    pub fn prepare_owned_project(&self, participant:Id, actor:Id, target:Option<Id>, process:&str,
        project:Option<ProjectId>, lots:&[Key], cause:Option<Key>) -> Result<super::DebitPreview> {
        self.project_candidate(participant,actor,target,process,project,lots)?
            .prepare_owned_act(actor,target,process,cause)
    }
    fn act_project_owned(&mut self, participant:Id, actor:Id, target:Option<Id>, process:&str,
        project:Option<ProjectId>, cause:Option<Key>, lots:&[Key], selections:&[super::DebitSelection]) -> Result<String> {
        let mut candidate = self.project_candidate(participant,actor,target,process,project,lots)?;
        let result = candidate.owned_act(participant,actor,target,process,cause,selections);
        candidate.project_inputs = None;
        let owned:super::OwnedActed = serde_json::from_str(&result?).map_err(|error|error.to_string())?;
        if !matches!(owned.receipt.outcome, Outcome::Accepted | Outcome::RiskOutcome) {
            return Err(format!("owned project act was not admitted: {:?}",owned.receipt.outcome));
        }
        let out = OwnedProjectActed { acted:candidate.project_result(actor,owned.receipt),
            debits:owned.debits,claims:owned.claims };
        let encoded = serde_json::to_string(&out).map_err(|why|why.to_string())?;
        *self = candidate;
        Ok(encoded)
    }
    /// The game pairs its actual position and burden check with this native cut.
    fn carry_project(&mut self, participant:Id, actor:Id, id:ProjectId, carried:bool) -> Result<String> {
        self.project_control(participant, actor)?;
        let mut project = self.state.projects.get(id).ok_or("project is absent")?.clone();
        self.project_eligible(actor,&project)?;
        let site = self.body_at_start(actor).and_then(Entity::resident_site).ok_or("project carrier is in transit")?;
        if carried {
            if project.carrier.is_some_and(|carrier| carrier != actor) { return Err("project already has a different carrier".into()); }
            project.carrier = Some(actor);
        } else {
            if project.carrier != Some(actor) { return Err("only the current native carrier drops this project".into()); }
            project.site = site;
            project.carrier = None;
        }
        let matter = self.project_matter(id)?;
        let out = serde_json::to_string(&Custody { project:id,carrier:project.carrier,site,matter }).map_err(|e| e.to_string())?;
        self.state.projects.records.insert(id,project);
        Ok(out)
    }
}
