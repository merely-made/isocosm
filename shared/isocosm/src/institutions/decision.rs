// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projects::{DecisionPolicy, DebitPreview};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DecisionPhase { Approval, Application }

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionContext {
    pub proposal: Key, pub actor: Id, pub target: Option<Id>, pub polity: PolityId,
    pub constitution: Key, pub process: Key, pub phase: DecisionPhase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DecisionStamp { pub decision: DecisionContext, pub action: Key }

impl Simulation {
    pub(crate) fn institution_decision_ready(&self, actor: Id, target: Option<Id>) -> Result<String> {
        let input = self.institution_decision.as_ref().ok_or("no constitutional decision is proposed")?;
        self.institution_context_ready(input,actor,target)
    }

    pub(super) fn institution_context_ready(&self, input:&DecisionContext, actor:Id, target:Option<Id>) -> Result<String> {
        let proposal = self.institution_records_at_start().core(&input.proposal)?;
        self.institution_current(&proposal)?;
        self.institution_support(&proposal.operation)?;
        let polity = self.institution_polity_at_start(proposal.polity.0).ok_or("institution is absent")?;
        if input.actor != actor || input.target != target || input.polity != proposal.polity
            || crate::digest(&polity.constitution) != input.constitution
            || self.institution_records_at_start().is_applied(&proposal.key)? {
            return Err("constitutional decision names a stale institution, body or native target".into());
        }
        match input.phase {
            DecisionPhase::Approval => {
                let DecisionPolicy::Governance { process } = &proposal.operation.decision else {
                    return Err("individual constitutional consent executes no governance process".into());
                };
                if input.process != *process || !proposal.electorate.contains(&actor) {
                    return Err("decision has another authored process or constitutional member".into());
                }
            },
            DecisionPhase::Application if actor != proposal.actor || input.process != proposal.operation.process
                || target != proposal.request.target() || !self.institution_ready(&proposal) =>
                return Err("institution application lacks its admitted executor, process or decisions".into()),
            DecisionPhase::Application => {},
        }
        Ok(format!("constitutional decision ready: {}",proposal.polity.0))
    }

    pub(super) fn institution_context(&self, proposal: &ProposalCore, actor:Id, target:Option<Id>,
        process: &str, phase: DecisionPhase) -> DecisionContext {
        DecisionContext { proposal:proposal.key.clone(),actor,target,polity:proposal.polity,
            constitution:crate::digest(&proposal.constitution),process:process.into(),phase }
    }

    /// Quote the actual scoped native body process. Exact current stock lots
    /// remain selected by the caller and are checked by owned_act at admission.
    pub fn prepare_institution(&self, change: &Change) -> Result<DebitPreview> {
        let (participant,actor,key,process,target,cause,phase) = match change {
            Change::Approve { participant,actor,proposal,process:Some(process),target,cause,.. } =>
                (*participant,*actor,proposal,process.clone(),*target,cause.clone(),DecisionPhase::Approval),
            Change::Apply { participant,actor,proposal,.. } => {
                let held = self.state.institutions.core(proposal)?;
                (*participant,*actor,proposal,held.operation.process.clone(),held.request.target(),held.request.cause(),DecisionPhase::Application)
            },
            _ => return Err("this institution input creates no native body-stock operation".into()),
        };
        self.institution_control(participant,actor)?;
        let proposal = self.state.institutions.core(key)?;
        let mut candidate = self.clone();
        candidate.institution_decision = Some(self.institution_context(&proposal,actor,target,&process,phase));
        candidate.institution_decision_stamp = None;
        candidate.institution_decision_ready(actor,target)?;
        candidate.prepare_owned_act(actor,target,&process,cause)
    }
}
