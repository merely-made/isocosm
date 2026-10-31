// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{projects::{DecisionPolicy, Owner}, rules::{Causation, Effect, Query, Rules}};

fn process(rules: &Rules, key: &str, deciding: bool) -> Result<()> {
    crate::validation::key(key)?;
    let process = rules.processes.get(key).ok_or("institution process is undeclared")?;
    if process.causation != Causation::Choice || !process.note {
        return Err("institution process needs an authored native Choice and actual event witness".into());
    }
    if deciding {
        fn marker(effect: &Effect) -> bool {
            matches!(effect,Effect::InstitutionDecision) || effect.branches().into_iter().any(marker)
        }
        if !process.requires.contains(&Query::InstitutionDecision)
            || !process.commitments.iter().chain(&process.effects).chain(process.risk.iter().flat_map(|risk| &risk.effects)).any(marker) {
            return Err("institution decision requires its scoped authored Query and Effect".into());
        }
    }
    Ok(())
}

pub fn validate(rules: &Rules) -> Result<()> {
    for (key,definition) in &rules.institutions {
        crate::validation::key(key)?;
        for (name,operation) in &definition.operations { crate::validation::key(name)?; operation.validate(rules,false)?; }
        for operation in [&definition.amend,&definition.end].into_iter().flatten() { operation.validate(rules,true)?; }
    }
    super::producer::validate_producers(rules)?;
    Ok(())
}

impl Operation {
    fn validate(&self, rules: &Rules, deciding: bool) -> Result<()> {
        process(rules,&self.process,deciding)?;
        match &self.decision {
            DecisionPolicy::Governance { process: key } => process(rules,key,true)?,
            other => other.validate(rules)?,
        }
        for support in &self.support {
            crate::validation::key(&support.account)?;
            if !rules.accounts.contains_key(&support.account) { return Err("institution support account is undeclared".into()); }
        }
        Ok(())
    }
}

impl Simulation {
    pub(super) fn institution_operation(&self, polity: &Polity, request: &Request) -> Result<Operation> {
        let definition = self.genesis.rules.institutions.get(&polity.constitution.governance)
            .ok_or("constitution has no explicitly authored institutional method")?;
        let operation = match request {
            Request::Act { operation,.. } => definition.operations.get(operation),
            Request::Amend { .. } => definition.amend.as_ref(),
            Request::End { .. } => definition.end.as_ref(),
        }.ok_or("constitution declares no method for this institutional operation")?;
        operation.validate(&self.genesis.rules,!matches!(request,Request::Act { .. }))?;
        Ok(operation.clone())
    }

    pub(super) fn institution_electorate(&self, polity: &Polity, operation: &Operation) -> Result<(BTreeSet<Id>,u64)> {
        let members = &polity.constitution.members;
        let (actors,approvals) = match &operation.decision {
            DecisionPolicy::Governance { .. } => (members.clone(),1),
            DecisionPolicy::UnanimousMembers => (members.clone(),members.len() as u64),
            DecisionPolicy::Representatives { actors,approvals } => (actors.clone(),*approvals),
        };
        if actors.is_empty() || approvals == 0 || approvals > actors.len() as u64
            || !actors.is_subset(members) { return Err("institution has no valid authored current-member electorate".into()); }
        for actor in actors.iter().filter(|_| !matches!(operation.decision,DecisionPolicy::Governance { .. })) {
            if !self.body_at_start(*actor).is_some_and(|body| body.alive && !crate::directing::is_participant(body)) {
                return Err("institution electorate includes an unavailable native body".into());
            }
        }
        Ok((actors,approvals))
    }

    pub(super) fn institution_current(&self, proposal: &ProposalCore) -> Result<()> {
        self.institution_shape(proposal)?;
        let polity = self.institution_polity_at_start(proposal.polity.0).ok_or("institution is absent")?;
        if self.institution_electorate(polity,&proposal.operation)? != (proposal.electorate.clone(),proposal.approvals) {
            return Err("institution electorate changed; propose again".into());
        }
        Ok(())
    }

    pub(super) fn institution_shape(&self, proposal: &ProposalCore) -> Result<()> {
        let polity = self.institution_polity_at_start(proposal.polity.0).filter(|p| p.ended_by.is_none())
            .ok_or("institution is absent or ended")?;
        if polity.constitution != proposal.constitution
            || self.institution_operation(polity,&proposal.request)? != proposal.operation {
            return Err("institution constitution, method or electorate changed; propose again".into());
        }
        Ok(())
    }

    pub(super) fn institution_support(&self, operation: &Operation) -> Result<()> {
        for support in &operation.support {
            let held = match support.source {
                Owner::Entity(id) => crate::anatomy::held(self.body_at_start(id)
                    .ok_or("institution support body is absent")?,&self.genesis.rules,&support.account),
                Owner::Polity(id) => self.institution_polity_at_start(id).filter(|p| p.ended_by.is_none())
                    .ok_or("institution support polity is absent or ended")?.accounts.get(&support.account).copied().unwrap_or(0),
            };
            if held < support.minimum { return Err("institution lacks the explicitly required native support".into()); }
        }
        Ok(())
    }
}
