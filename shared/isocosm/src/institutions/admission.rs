// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use super::decision::DecisionPhase;
use crate::{projects::{DebitSelection, DecisionPolicy, OwnedActed}, simulation::{Outcome as ActOutcome, Receipt}};

impl Simulation {
    pub(super) fn institution_ready(&self, proposal: &Proposal) -> bool {
        if proposal.applied.is_some() || self.institution_current(proposal).is_err() { return false; }
        let voters: BTreeSet<_> = proposal.votes.keys().copied().collect();
        voters.is_subset(&proposal.electorate) && voters.len() as u64 >= proposal.approvals
            && (!matches!(proposal.operation.decision,DecisionPolicy::Governance { .. })
                || proposal.votes.values().any(|vote| vote.act.is_some()))
    }

    /// Logged native input. This external command publishes one complete
    /// candidate and never grants a body, account, party or aggregate control.
    pub(crate) fn institution_change(&mut self, change: &Change) -> Result<String> {
        if self.frame.is_some() || self.pass.is_some() || self.institution_decision.is_some() {
            return Err("institution commands require their external native admission boundary".into());
        }
        let mut candidate = self.clone();
        let result = match change {
            Change::Propose { key,participant,actor,polity,request } =>
                candidate.propose_institution(key,*participant,*actor,*polity,request)?,
            Change::Approve { participant,actor,proposal,process,target,cause,selections } =>
                candidate.approve_institution(*participant,*actor,proposal,process.as_deref(),*target,cause.clone(),selections)?,
            Change::Apply { participant,actor,proposal,selections } =>
                candidate.apply_institution(*participant,*actor,proposal,selections)?,
        };
        if candidate.matter() != candidate.conserved { return Err("institution operation changed conserved native matter".into()); }
        let out = serde_json::to_string(&result).map_err(|e| e.to_string())?;
        *self = candidate;
        Ok(out)
    }

    fn propose_institution(&mut self, key:&str, participant:Id, actor:Id, id:PolityId, request:&Request) -> Result<Outcome> {
        crate::validation::key(key)?;
        self.institution_control(participant,actor)?;
        if self.state.institutions.proposals.contains_key(key) { return Err("institution proposal key already exists".into()); }
        let polity = self.state.polities.get(&id.0).filter(|p| p.ended_by.is_none()).ok_or("institution is absent or ended")?;
        if !polity.constitution.members.contains(&actor) { return Err("institution executor is not an actual constitutional member".into()); }
        let operation = self.institution_operation(polity,request)?;
        let (electorate,approvals) = self.institution_electorate(polity,&operation)?;
        self.institution_support(&operation)?;
        if let Request::Amend { constitution,.. } = request { self.check_constitution(&polity.constitution,constitution)?; }
        let proposal = Proposal { key:key.into(),participant,actor,polity:id,constitution:polity.constitution.clone(),
            request:request.clone(),operation,electorate,approvals,created:self.state.tick,votes:BTreeMap::new(),applied:None };
        self.state.institutions.proposals.insert(key.into(),proposal);
        Ok(Outcome { proposal:key.into(),ready:false,applied:None,decision:None })
    }

    fn approve_institution(&mut self, participant:Id, actor:Id, key:&str, process:Option<&str>, target:Option<Id>,
        cause:Option<Key>, selections:&[DebitSelection]) -> Result<Outcome> {
        self.institution_control(participant,actor)?;
        let mut proposal = self.state.institutions.proposals.get(key).ok_or("institution proposal is absent")?.clone();
        self.institution_current(&proposal)?;
        self.institution_support(&proposal.operation)?;
        if proposal.applied.is_some() || !proposal.electorate.contains(&actor) || proposal.votes.contains_key(&actor) {
            return Err("institution consent names an unavailable proposal or another/already-voted native body".into());
        }
        let act = match &proposal.operation.decision {
            DecisionPolicy::Governance { process: declared } => {
                if process != Some(declared.as_str()) { return Err("institution consent needs its exact authored deciding process".into()); }
                let context = self.institution_context(&proposal,actor,target,declared,DecisionPhase::Approval);
                let receipt = self.run_institution(participant,actor,target,declared,cause,selections,&context,true)?;
                self.institution_shape(&proposal)?;
                Some(self.institution_witness(proposal.polity,&receipt,ActKind::Decision)?)
            },
            _ => {
                if process.is_some() || target.is_some() || cause.is_some() || !selections.is_empty() {
                    return Err("individual constitutional consent has no inferred process, target or debit".into());
                }
                None
            },
        };
        proposal.votes.insert(actor,Approval { participant,actor,tick:self.state.tick,act:act.clone() });
        let ready = self.institution_ready(&proposal);
        self.state.institutions.proposals.insert(key.into(),proposal);
        Ok(Outcome { proposal:key.into(),ready,applied:None,decision:act })
    }

    fn apply_institution(&mut self, participant:Id, actor:Id, key:&str, selections:&[DebitSelection]) -> Result<Outcome> {
        self.institution_control(participant,actor)?;
        let mut proposal = self.state.institutions.proposals.get(key).ok_or("institution proposal is absent")?.clone();
        if proposal.actor != actor || !self.institution_ready(&proposal) {
            return Err("institution application requires its actual executor and current admitted decisions".into());
        }
        self.institution_support(&proposal.operation)?;
        let deciding = !matches!(proposal.request,Request::Act { .. });
        let context = self.institution_context(&proposal,actor,proposal.request.target(),&proposal.operation.process,DecisionPhase::Application);
        let receipt = self.run_institution(participant,actor,proposal.request.target(),&proposal.operation.process,
            proposal.request.cause(),selections,&context,deciding)?;
        self.institution_shape(&proposal)?;
        let kind = match &proposal.request {
            Request::Act { .. } => ActKind::Operation,
            Request::Amend { constitution,.. } => {
                self.check_constitution(&proposal.constitution,constitution)?;
                if let Some(journal) = &mut self.journal { journal.polity_before(proposal.polity.0,&self.state.polities[&proposal.polity.0]); }
                self.state.polities.get_mut(&proposal.polity.0).ok_or("institution disappeared")?.constitution = constitution.clone();
                ActKind::Amendment
            },
            Request::End { .. } => {
                if let Some(journal) = &mut self.journal { journal.polity_before(proposal.polity.0,&self.state.polities[&proposal.polity.0]); }
                self.state.polities.get_mut(&proposal.polity.0).ok_or("institution disappeared")?.ended_by = Some(receipt.id.clone());
                ActKind::Dissolution
            },
        };
        let act = self.institution_witness(proposal.polity,&receipt,kind)?;
        let applied = Applied { participant,tick:self.state.tick,act };
        proposal.applied = Some(applied.clone());
        self.state.institutions.proposals.insert(key.into(),proposal);
        Ok(Outcome { proposal:key.into(),ready:false,applied:Some(applied),decision:None })
    }

    fn run_institution(&mut self, participant:Id, actor:Id, target:Option<Id>, process:&str, cause:Option<Key>,
        selections:&[DebitSelection], context:&DecisionContext, deciding:bool) -> Result<Receipt> {
        self.institution_decision = Some(context.clone());
        self.institution_decision_stamp = None;
        let result = self.owned_act(participant,actor,target,process,cause,selections);
        let stamp = self.institution_decision_stamp.take();
        self.institution_decision = None;
        let acted: OwnedActed = serde_json::from_str(&result?).map_err(|e| e.to_string())?;
        if !acted.receipt.accepted() || deciding && (acted.receipt.outcome != ActOutcome::Accepted
            || stamp.as_ref().is_none_or(|stamp| stamp.decision != *context || stamp.action != acted.receipt.id)) {
            return Err("institution process was not admitted with its actual constitutional decision witness".into());
        }
        Ok(acted.receipt)
    }

    fn institution_witness(&self, polity:PolityId, receipt:&Receipt, kind:ActKind) -> Result<ActWitness> {
        let event = self.state.events.get(&receipt.id).ok_or("institution process has no actual native event")?;
        if event.authored.is_some() || event.subject != receipt.actor || event.process != receipt.chosen
            || event.tick != receipt.tick || event.object != receipt.target || event.cause != receipt.cause {
            return Err("institution attribution differs from the actual body's admitted event".into());
        }
        Ok(ActWitness { polity,actor:receipt.actor,event:event.id.clone(),process:event.process.clone(),kind })
    }

    fn check_constitution(&self, old:&Constitution, offered:&Constitution) -> Result<()> {
        if offered.founded_by != old.founded_by { return Err("amendment cannot rewrite the institution's founding cause".into()); }
        crate::validation::key(&offered.governance)?;
        crate::validation::key(&offered.support_account)?;
        if !self.genesis.rules.institutions.contains_key(&offered.governance)
            || !self.genesis.rules.accounts.contains_key(&offered.support_account) {
            return Err("amendment has no declared deciding method or support account".into());
        }
        for focus in &offered.focus { crate::validation::key(focus)?; }
        for member in &offered.members {
            if !self.state.population.get(*member).is_some_and(|body| !crate::directing::is_participant(body)) {
                return Err("amendment names no actual native member body".into());
            }
        }
        if offered.host != old.host {
            return Err("institution hosting changes require a typed native host admission".into());
        }
        Ok(())
    }
}
