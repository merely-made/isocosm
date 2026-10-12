// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::{Causation, Rules};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionPolicy {
    Governance { process: Key },
    UnanimousMembers,
    Representatives { actors: BTreeSet<Id>, approvals: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Electorate {
    Individuals { actors: BTreeSet<Id>, approvals: u64, constitution: Option<Key> },
    Governance { polity: Id, process: Key, constitution: Key, members: BTreeSet<Id> },
}

impl DecisionPolicy {
    pub fn validate(&self, rules: &Rules) -> Result<()> {
        match self {
            Self::Governance { process } => {
                crate::validation::key(process)?;
                fn decision(e: &crate::rules::Effect) -> bool {
                    matches!(e, crate::rules::Effect::ProjectDecision) || e.branches().into_iter().any(decision)
                }
                if !rules.processes.get(process).is_some_and(|p| p.causation == Causation::Choice
                    && p.requires.contains(&crate::rules::Query::ProjectDecision)
                    && p.commitments.iter().chain(&p.effects).chain(p.risk.iter().flat_map(|r| &r.effects)).any(decision)) {
                    return Err("collective decision requires its authored native Choice process".into());
                }
            },
            Self::Representatives { actors, approvals } if actors.is_empty() || *approvals == 0 || *approvals > actors.len() as u64 =>
                return Err("representative decision needs explicit native actors and quorum".into()),
            _ => {},
        }
        Ok(())
    }
}

impl Simulation {
    pub(crate) fn owner_exists(&self, owner: Owner) -> Result<()> {
        let exists = match owner {
            Owner::Entity(id) => self.body_at_start(id).is_some_and(|e| e.alive && !crate::directing::is_participant(e)),
            Owner::Polity(id) => self.state.polities.get(&id).is_some_and(|p| p.ended_by.is_none()),
        };
        if exists { Ok(()) } else { Err("native input owner is unavailable".into()) }
    }
    pub(crate) fn owner_electorate(&self, project: &Project, owner: Owner) -> Result<Electorate> {
        if !project.owners.contains(&owner) { return Err("subject is not an owner of the reserved inputs".into()); }
        self.owner_exists(owner)?;
        let electorate = match owner {
            Owner::Entity(id) => Electorate::Individuals { actors:BTreeSet::from([id]), approvals:1, constitution:None },
            Owner::Polity(id) => match project.collectives.get(&id).ok_or("collective decision policy is absent")? {
                DecisionPolicy::Governance { process } => {
                    DecisionPolicy::Governance { process: process.clone() }.validate(&self.genesis.rules)?;
                    let constitution = &self.state.polities[&id].constitution;
                    return Ok(Electorate::Governance { polity:id, process:process.clone(),
                        constitution:crate::digest(constitution),members:constitution.members.clone() });
                },
                DecisionPolicy::UnanimousMembers => {
                    let actors = self.state.polities[&id].constitution.members.clone();
                    Electorate::Individuals { approvals:actors.len() as u64, actors,
                        constitution:Some(crate::digest(&self.state.polities[&id].constitution)) }
                },
                DecisionPolicy::Representatives { actors, approvals } =>
                    Electorate::Individuals { actors:actors.clone(), approvals:*approvals,
                        constitution:Some(crate::digest(&self.state.polities[&id].constitution)) },
            },
        };
        if let Electorate::Individuals { actors, approvals, .. } = &electorate {
            if *approvals == 0 || *approvals > actors.len() as u64 { return Err("native owner electorate is empty or invalid".into()); }
            for id in actors { self.owner_exists(Owner::Entity(*id))?; }
        }
        Ok(electorate)
    }
    pub fn project_consent_ready(&self, project: &Project, owner: Owner, actor: Id) -> Result<Option<Key>> {
        self.owner_exists(Owner::Entity(actor))?;
        match self.owner_electorate(project,owner)? {
            Electorate::Individuals { actors, .. } if actors.contains(&actor) => Ok(None),
            Electorate::Governance { process,members, .. } if members.contains(&actor) => Ok(Some(process)),
            _ => Err("actor is not admitted by this native owner's current electorate".into()),
        }
    }
}
