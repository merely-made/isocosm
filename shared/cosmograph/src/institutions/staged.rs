// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projects::DecisionPolicy;
use super::decision::DecisionPhase;

#[derive(Clone, Debug)]
pub(crate) struct BodyWrite {
    pub(super) producer: Key, pub(super) declaration: BodyProducer, pub(super) core: ProposalCore,
    pub(super) polity_before: Polity, pub(super) target: Option<Id>, pub(super) cause: Option<Key>,
    pub(super) action: Key, pub(super) act: Option<ActWitness>,
}

impl BodyWrite {
    pub(crate) fn finish(&mut self,event:&Event,stamp:Option<&DecisionStamp>,risky:bool) -> Result<()> {
        if event.id!=self.action || event.subject!=self.declaration.actor || event.process!=self.declaration.process
            || event.object!=self.target || event.cause!=self.cause || event.authored.is_some() {
            return Err("institution producer differs from its actual selected body's event".into());
        }
        let deciding=match &self.declaration.step {
            BodyStep::Approve {..}=>matches!(self.core.operation.decision,DecisionPolicy::Governance {..}),
            BodyStep::Apply {..}=>!matches!(self.core.request,Request::Act {..}),
            _=>false,
        };
        let phase=match &self.declaration.step {
            BodyStep::Approve {..}=>DecisionPhase::Approval,_=>DecisionPhase::Application,
        };
        if deciding && (risky || stamp.is_none_or(|stamp| stamp.action!=self.action
            || stamp.decision.proposal!=self.core.key || stamp.decision.actor!=self.declaration.actor
            || stamp.decision.polity!=self.core.polity || stamp.decision.target!=self.target
            || stamp.decision.process!=self.declaration.process
            || stamp.decision.phase!=phase
            || stamp.decision.constitution!=crate::digest(&self.core.constitution))) {
            return Err("constitutional producer lacks its actual accepted scoped decision stamp".into());
        }
        let kind=match &self.declaration.step {
            BodyStep::Propose {..}=>ActKind::Proposal,
            BodyStep::Approve {..}=>ActKind::Decision,
            BodyStep::Apply {..}=>match self.core.request {
                Request::Act {..}=>ActKind::Operation,Request::Amend {..}=>ActKind::Amendment,Request::End {..}=>ActKind::Dissolution,
            },
        };
        self.act=Some(ActWitness {polity:self.core.polity,actor:event.subject,event:event.id.clone(),process:event.process.clone(),kind});
        Ok(())
    }

    pub(crate) fn fits(&self,sim:&Simulation) -> bool {
        if sim.genesis.rules.institution_producers.get(&self.producer)!=Some(&self.declaration)
            || sim.state.polities.get(&self.core.polity.0)!=Some(&self.polity_before) {return false;}
        let mut live=sim.clone();
        live.frame=None;
        live.clear_owned_capture();
        if live.institution_support(&self.core.operation).is_err() {return false;}
        let records=&sim.state.institutions;
        match &self.declaration.step {
            BodyStep::Propose {..}=>!records.proposals.contains_key(&self.core.key)&&!records.body_proposals.contains_key(&self.core.key),
            BodyStep::Approve {..}=>records.core(&self.core.key).is_ok_and(|core|core==self.core)
                && records.is_applied(&self.core.key)==Ok(false)
                && records.voters(&self.core.key).is_ok_and(|voters|!voters.contains(&self.declaration.actor)),
            BodyStep::Apply {..}=>{
                records.core(&self.core.key).is_ok_and(|core|core==self.core) && live.institution_ready(&self.core)
            },
        }
    }
}

impl Simulation {
    pub(crate) fn stage_institution_producer(&self,actor:Id,target:Option<Id>,key:&str,action:&str,planning:bool) -> Result<BodyWrite> {
        let mut input=self.producer_input(actor,target,key)?;
        if !planning && !self.institution_chosen(actor,&input.declaration.process) {
            return Err("institution producer has no actual selected native choice in this pass".into());
        }
        input.action=action.into();
        Ok(input)
    }

    pub(crate) fn publish_institution_producer(&mut self,input:BodyWrite) -> Result<()> {
        let act=input.act.ok_or("institution producer has no completed native event witness")?;
        let event=self.state.events.get(&act.event).ok_or("institution body's accepted event is absent")?;
        if event.subject!=act.actor || event.process!=act.process || event.authored.is_some()
            || event.tick!=self.state.tick || event.id!=input.action
            || self.state.polities.get(&input.core.polity.0)!=Some(&input.polity_before) {
            return Err("institution producer publication changes its actual body/event/constitution preimage".into());
        }
        if let Some(frame)=&mut self.frame {
            frame.keep_institutions(&self.state.institutions);
            frame.keep_polity(input.core.polity.0,Some(&input.polity_before));
        }
        let records=&mut self.state.institutions;
        match &input.declaration.step {
            BodyStep::Propose {..}=>{
                if records.proposals.contains_key(&input.core.key)||records.body_proposals.contains_key(&input.core.key) {
                    return Err("institution proposal identity was already accepted".into());
                }
                records.body_proposals.insert(input.core.key.clone(),BodyProposal {
                    core:input.core,act,votes:BTreeMap::new(),applied:None});
            },
            BodyStep::Approve {..}=>{
                if records.core(&input.core.key)?!=input.core || records.is_applied(&input.core.key)?
                    || records.voters(&input.core.key)?.contains(&act.actor) {
                    return Err("institution consent became stale before native publication".into());
                }
                records.body_votes.entry(input.core.key).or_default().insert(act.actor,
                    BodyVote {actor:act.actor,tick:self.state.tick,act});
            },
            BodyStep::Apply {..}=>{
                if records.core(&input.core.key)?!=input.core || records.is_applied(&input.core.key)? {
                    return Err("institution application became stale before native publication".into());
                }
                match &input.core.request {
                    Request::Amend {constitution,..}=>{
                        if let Some(j)=&mut self.journal {j.polity_before(input.core.polity.0,&input.polity_before);}
                        self.state.polities.get_mut(&input.core.polity.0).ok_or("institution is absent")?.constitution=constitution.clone();
                    },
                    Request::End {..}=>{
                        if let Some(j)=&mut self.journal {j.polity_before(input.core.polity.0,&input.polity_before);}
                        self.state.polities.get_mut(&input.core.polity.0).ok_or("institution is absent")?.ended_by=Some(act.event.clone());
                    },
                    _=>{},
                }
                records.body_applied.insert(input.core.key,BodyApplied {tick:self.state.tick,act});
            },
        }
        Ok(())
    }
}
