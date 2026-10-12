// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::simulation::Outcome;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerDecision {
    pub owner: Owner,
    pub electorate: Electorate,
    pub actors: BTreeSet<Id>,
    pub action: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission {
    pub granted: bool,
    pub round: u64,
    pub proof: Vec<OwnerDecision>,
    pub proposal: Option<bool>,
    pub decisions: Vec<OwnerDecision>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Consented { pub project: ProjectId, pub worker: Id, pub authorized: bool, pub round: u64 }

impl Simulation {
    fn decision_complete(&self, project: &Project, decision: &OwnerDecision) -> bool {
        if self.owner_electorate(project, decision.owner).ok().as_ref() != Some(&decision.electorate) { return false; }
        match &decision.electorate {
            Electorate::Individuals { actors, approvals, .. } =>
                decision.actors.is_subset(actors) && decision.actors.len() as u64 >= *approvals,
            Electorate::Governance { .. } => decision.action.is_some() && !decision.actors.is_empty(),
        }
    }
    fn jointly_decided(&self, project: &Project, decisions: &[OwnerDecision]) -> bool {
        !project.owners.is_empty() && decisions.len() == project.owners.len()
            && project.owners.iter().all(|owner| decisions.iter()
                .find(|decision| decision.owner == *owner).is_some_and(|d| self.decision_complete(project, d)))
    }
    pub fn project_authorized(&self, project: &Project, worker: Id) -> bool {
        project.permissions.get(&worker).is_some_and(|p| p.granted && self.jointly_decided(project, &p.proof))
    }
    pub(super) fn consent_project(&mut self, participant: Id, actor: Id, id: ProjectId,
        owner: Owner, worker: Id, grant: bool, process: Option<&str>, cause: Option<Key>) -> Result<String> {
        self.project_control(participant, actor)?;
        if !self.body_at_start(worker).is_some_and(|e| e.alive && !crate::directing::is_participant(e)) {
            return Err("project permission names no native living worker".into());
        }
        let project = self.state.projects.get(id).ok_or("project is absent")?.clone();
        if project.output.is_some() { return Err("completed project needs no work permission".into()); }
        let electorate = self.owner_electorate(&project, owner)?;
        let declared = self.project_consent_ready(&project,owner,actor)?;
        if process != declared.as_deref() { return Err("project consent names a different native owner decision".into()); }
        let mut candidate = self.clone();
        let action = match &electorate {
            Electorate::Individuals { actors, .. } => {
                if !actors.contains(&actor) || process.is_some() {
                    return Err("project consent needs this owner's actual native individual admission".into());
                }
                None
            },
            Electorate::Governance { polity,process: declared,constitution, .. } => {
                if process != Some(declared.as_str()) { return Err("collective consent needs its exact authored decision process".into()); }
                let context = DecisionContext { actor,project:id,polity:*polity,worker,grant,
                    process:declared.clone(),constitution:constitution.clone() };
                candidate.project_decision = Some(context.clone());
                candidate.project_decision_stamp = None;
                let receipt = candidate.execute(actor, Some(worker), declared, cause);
                let stamp = candidate.project_decision_stamp.take();
                candidate.project_decision = None;
                if receipt.outcome != Outcome::Accepted || candidate.state.projects.get(id) != Some(&project)
                    || stamp.as_ref().is_none_or(|s| s.decision != context || s.action != receipt.action) {
                    return Err("collective decision was not admitted or changed the work it decides".into());
                }
                Some(receipt.action)
            },
        };
        if candidate.owner_electorate(&project, owner)? != electorate {
            return Err("collective electorate changed during decision".into());
        }
        let mut permission = project.permissions.get(&worker).cloned().unwrap_or_default();
        if permission.proposal != Some(grant) {
            permission.round = permission.round.checked_add(1).ok_or("project consent round overflow")?;
            permission.proposal = Some(grant);
            permission.decisions.clear();
        }
        permission.decisions.retain(|d| d.owner != owner || d.electorate == electorate);
        let index = match permission.decisions.iter().position(|d| d.owner == owner) {
            Some(index) => index,
            None => {
                permission.decisions.push(OwnerDecision { owner, electorate, actors:BTreeSet::new(), action:None });
                permission.decisions.len() - 1
            },
        };
        permission.decisions[index].actors.insert(actor);
        permission.decisions[index].action = action;
        permission.decisions.sort_by_key(|d| d.owner);
        if candidate.jointly_decided(&project, &permission.decisions) {
            permission.granted = grant;
            permission.proof = permission.decisions.clone();
            permission.proposal = None;
            permission.decisions.clear();
        }
        let round = permission.round;
        let mut project = project;
        project.permissions.insert(worker, permission);
        let authorized = candidate.project_authorized(&project, worker);
        if authorized { project.authorized.insert(worker); } else { project.authorized.remove(&worker); }
        candidate.state.projects.records.insert(id, project);
        let out = serde_json::to_string(&Consented { project:id, worker, authorized, round }).map_err(|e| e.to_string())?;
        *self = candidate;
        Ok(out)
    }
}
