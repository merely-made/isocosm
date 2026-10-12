// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native reserved stocks and retained construction, independently resumed (837/838).

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::{schema::*, simulation::Simulation, Result};

mod definition;
pub use definition::{Catalogue, Definition, Step};
mod admission;
pub use admission::{Acted, Change, Custody};
mod readiness;
mod ownership;
pub use ownership::{LotAt, OwnedLot, Owner, StockLot, StockSource};
mod policy;
pub use policy::{DecisionPolicy, Electorate};
mod consent;
pub use consent::{Consented, OwnerDecision, Permission};
mod decision;
pub(crate) use decision::{DecisionContext, DecisionStamp};
mod debit;
pub use debit::guard_debit;

/// The accepted action that reserved the project, in its own identity domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProjectId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase { Reserve, Work }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub definition: Key,
    pub recipe: Key,
    pub maker: Id,
    /// Legacy issuer metadata; current access uses joint input-owner consent.
    pub authority: Id,
    pub site: Id,
    pub reserved: Ledger,
    pub construction: Ledger,
    pub progress: u64,
    pub steps: u64,
    pub authorized: BTreeSet<Id>,
    pub output: Option<Id>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carrier: Option<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<Key>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub owners: BTreeSet<Owner>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub collectives: BTreeMap<Id, DecisionPolicy>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub permissions: BTreeMap<Id, Permission>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projects {
    pub records: BTreeMap<ProjectId, Project>,
    pub bindings: BTreeMap<Id, ProjectId>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ownership: BTreeMap<Key, OwnedLot>,
}

impl Projects {
    pub fn is_empty(&self) -> bool { self.records.is_empty() && self.bindings.is_empty() && self.ownership.is_empty() }
    pub fn get(&self, id: ProjectId) -> Option<&Project> { self.records.get(&id) }
    pub fn bound(&self, actor: Id) -> Option<(ProjectId, &Project)> {
        let id = *self.bindings.get(&actor)?;
        Some((id, self.get(id)?))
    }
}

impl Simulation {
    pub(crate) fn project_at_start(&self, id: ProjectId) -> Option<&Project> {
        self.frame.as_ref().and_then(|frame| frame.project(id))
            .map_or_else(|| self.state.projects.get(id), |was| was.as_ref())
    }
    pub(crate) fn project_binding_at_start(&self, actor: Id) -> Option<ProjectId> {
        self.frame.as_ref().and_then(|frame| frame.project_binding(actor))
            .unwrap_or_else(|| self.state.projects.bindings.get(&actor).copied())
    }
    pub(crate) fn project_definition(&self, key: &str) -> Result<&Definition> {
        self.genesis.rules.projects.as_ref().and_then(|c| c.definitions.get(key))
            .ok_or_else(|| "project definition is absent".into())
    }
    /// Fixed work stays at creation; custody follows an actual native residence.
    pub fn project_site(&self, project: &Project) -> Option<Id> {
        project.carrier.map_or(Some(project.site), |carrier|
            self.body_at_start(carrier).and_then(Entity::resident_site))
    }
    pub fn project_matter(&self, id: ProjectId) -> Result<u128> {
        let project = self.state.projects.get(id).ok_or("project is absent")?;
        project.reserved.iter().chain(&project.construction).try_fold(0u128, |total, (key, amount)| {
            if !matches!(self.genesis.rules.accounts.get(key), Some(crate::rules::AccountKind::Matter { .. })) {
                return Err("project stock is not declared matter".into());
            }
            total.checked_add(u128::from(*amount)).ok_or_else(|| "project matter overflow".into())
        })
    }
    pub(crate) fn project_eligible(&self, actor: Id, project: &Project) -> Result<()> {
        let body = self.body_at_start(actor).ok_or("project worker is absent")?;
        let definition = self.project_definition(&project.definition)?;
        if !body.alive || !body.resident_site().is_some_and(|site| self.project_site(project) == Some(site))
            || project.output.is_some() || !self.project_authorized(project, actor)
            || !crate::social::Confidence::read(body, &definition.work).sufficient()
        { return Err("project worker lacks current craft, residence or native authorization".into()); }
        Ok(())
    }

}

pub(crate) fn validate_effect(rules: &crate::rules::Rules,
    process: &crate::rules::Process, effect: &crate::rules::Effect) -> Result<()> {
    use crate::rules::{Binding, Causation, Effect, Query};
    let phase = match effect {
        Effect::ProjectDecision => {
            if process.causation != Causation::Choice || !process.requires.contains(&Query::ProjectDecision) {
                return Err("project decision needs an actual authored Choice with its native decision gate".into());
            }
            return Ok(());
        },
        Effect::ReserveProject { definition, from, .. } => {
            if matches!(from, Binding::Part)
                || !rules.projects.as_ref().is_some_and(|c| c.definitions.contains_key(definition))
                || !process.requires.contains(&Query::Project { definition: definition.clone(),
                    phase: Phase::Reserve, process: process.id.clone() })
            { return Err("project reservation lacks declared source or native readiness".into()); }
            reserve_bindings(process, definition)?;
            Phase::Reserve
        },
        Effect::WorkProject => {
            if !process.requires.iter().any(|q| matches!(q,
                Query::Project { phase: Phase::Work, process: key, .. } if key == &process.id)) {
                return Err("project work lacks an authored native readiness gate".into());
            }
            Phase::Work
        },
        _ => return Ok(()),
    };
    if process.causation != Causation::Choice {
        return Err(format!("project {phase:?} needs a chosen native process"));
    }
    Ok(())
}

pub(crate) fn reserve_bindings(process: &crate::rules::Process, key: &str)
    -> Result<(crate::rules::Binding, Vec<Key>)> {
    use crate::rules::Effect;
    fn found(effect: &Effect, key: &str, out: &mut Vec<(crate::rules::Binding, Vec<Key>)>) {
        if let Effect::ReserveProject { definition, from, lots, .. } = effect
            && definition == key && !out.contains(&(*from, lots.clone())) { out.push((*from, lots.clone())); }
        for inner in effect.branches() { found(inner, key, out); }
    }
    let mut out = Vec::new();
    for effect in process.commitments.iter().chain(&process.effects)
        .chain(process.risk.iter().flat_map(|r| &r.effects)) { found(effect, key, &mut out); }
    match out.as_slice() {
        [only] => Ok(only.clone()),
        _ => Err("project reservation needs one declared source and lot mapping".into()),
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Writes {
    pub before: BTreeMap<ProjectId, Option<Project>>,
    pub records: BTreeMap<ProjectId, Project>,
    pub binding_before: BTreeMap<Id, Option<ProjectId>>,
    pub bindings: BTreeMap<Id, ProjectId>,
    pub ownership_before: BTreeMap<Key, Option<OwnedLot>>,
    pub ownership: BTreeMap<Key, OwnedLot>,
}

impl Writes {
    pub(crate) fn fits(&self, projects: &Projects) -> bool {
        self.before.iter().all(|(id, was)| projects.records.get(id) == was.as_ref())
            && self.binding_before.iter().all(|(actor, was)| projects.bindings.get(actor).copied() == *was)
            && self.ownership_before.iter().all(|(key, was)| projects.ownership.get(key) == was.as_ref())
    }
}
