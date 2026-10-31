// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use super::decision::DecisionPhase;
use crate::rules::{Causation, Effect, Process, Query, Rules};
use crate::projects::DecisionPolicy;

pub(super) fn validate_producers(rules:&Rules) -> Result<()> {
    for (key, producer) in &rules.institution_producers {
        crate::validation::key(key)?;
        crate::validation::key(producer.step.proposal())?;
        crate::validation::key(&producer.process)?;
        let p=rules.processes.get(&producer.process).ok_or("institution producer process is undeclared")?;
        if p.causation!=Causation::Choice || !p.note || !p.period.is_some_and(|period|period>0) {
            return Err("institution producer needs a scheduled authored Choice with its real event".into());
        }
        let gates=p.requires.iter().filter(|q| matches!(q,Query::Institution {producer:held} if held==key)).count();
        let marker=|e:&Effect| matches!(e,Effect::Institution {producer:held} if held==key);
        let markers=p.commitments.iter().chain(&p.effects).filter(|e|marker(e)).count();
        if gates!=1 || markers!=1 || p.risk.is_some() && !p.commitments.iter().any(marker)
            || p.requires.iter().any(|q|
            matches!(q,Query::Institution {producer:held} if held!=key)) {
            return Err("institution producer requires exactly its own explicit query and effect".into());
        }
        if p.commitments.iter().chain(&p.effects).chain(p.risk.iter().flat_map(|r|&r.effects))
            .any(|e| e.branches().any(|e| matches!(e,Effect::Institution {..})))
            || p.risk.as_ref().is_some_and(|r| r.effects.iter().any(marker)) {
            return Err("institution producer marker cannot be conditional or a risk alternative".into());
        }
        if let BodyStep::Propose {request,..}=&producer.step {
            if let Request::Act {operation,..}=request {crate::validation::key(operation)?;}
            if request.cause().as_ref().is_some_and(|cause|crate::validation::key(cause).is_err()) {
                return Err("institution request causal key is invalid".into());
            }
        }
    }
    for p in rules.processes.values() {
        for key in p.requires.iter().filter_map(|q|match q {Query::Institution {producer}=>Some(producer),_=>None}) {
            if rules.institution_producers.get(key).is_none_or(|producer| producer.process!=p.id) {
                return Err("institution producer query names another or missing process declaration".into());
            }
        }
        for effect in p.commitments.iter().chain(&p.effects).chain(p.risk.iter().flat_map(|r|&r.effects)) {
            validate_effect(rules,p,effect)?;
        }
    }
    Ok(())
}

fn validate_effect(rules:&Rules,p:&Process,e:&Effect) -> Result<()> {
    if let Effect::Institution {producer}=e {
        if rules.institution_producers.get(producer).is_none_or(|held|held.process!=p.id)
            || !p.requires.contains(&Query::Institution {producer:producer.clone()}) {
            return Err("institution effect has no matching scoped producer query/process".into());
        }
    }
    for branch in e.branches() {validate_effect(rules,p,branch)?;}
    Ok(())
}

impl Simulation {
    pub(super) fn institution_records_at_start(&self) -> &Records {
        self.frame.as_ref().and_then(|f|f.institutions()).unwrap_or(&self.state.institutions)
    }
    pub(super) fn institution_polity_at_start(&self,id:Id) -> Option<&Polity> {
        match self.frame.as_ref().and_then(|f|f.polity(id)) {
            Some(was)=>was.as_ref(),None=>self.state.polities.get(&id),
        }
    }

    pub(super) fn producer_input(&self, actor:Id,target:Option<Id>,key:&str) -> Result<BodyWrite> {
        let producer=self.genesis.rules.institution_producers.get(key).ok_or("institution producer is undeclared")?;
        let p=self.genesis.rules.processes.get(&producer.process).ok_or("institution producer process is absent")?;
        if self.pass.is_none() || actor!=producer.actor || p.causation!=Causation::Choice
            || !p.period.is_some_and(|period|period>0 && self.state.tick%period==0) {
            return Err("institution producer requires its actual scheduled native body/process pass".into());
        }
        let body=self.body_at_start(actor).ok_or("institution producer body is absent")?;
        if !body.alive || body.method==Method::Inert || crate::directing::is_participant(body) {
            return Err("institution producer has no available acting native body".into());
        }
        let polity=self.institution_polity_at_start(producer.polity.0).filter(|p|p.ended_by.is_none())
            .ok_or("institution producer names an absent or ended polity")?;
        if !polity.constitution.members.contains(&actor) {
            return Err("institution producer is not an actual constitutional member".into());
        }
        let records=self.institution_records_at_start();
        let core=match &producer.step {
            BodyStep::Propose {proposal,request}=>{
                if records.proposals.contains_key(proposal)||records.body_proposals.contains_key(proposal)
                    || target!=request.target() {return Err("institution request already exists or names another native target".into());}
                let operation=self.institution_operation(polity,request)?;
                let (electorate,approvals)=self.institution_electorate(polity,&operation)?;
                if let Request::Amend {constitution,..}=request {self.check_constitution(&polity.constitution,constitution)?;}
                ProposalCore {key:proposal.clone(),actor,polity:producer.polity,constitution:polity.constitution.clone(),
                    request:request.clone(),operation,electorate,approvals,created:self.state.tick}
            },
            BodyStep::Approve {proposal,target:declared,..}=>{
                let core=records.core(proposal)?;
                self.institution_current(&core)?;
                if core.polity!=producer.polity || records.is_applied(proposal)? || !core.electorate.contains(&actor)
                    || records.voters(proposal)?.contains(&actor) || target!=*declared {
                    return Err("institution consent lacks its exact current native proposal, electorate, target or voter".into());
                }
                if let DecisionPolicy::Governance {process}=&core.operation.decision {
                    if process!=&producer.process {return Err("institution consent uses another authored deciding process".into());}
                }
                core
            },
            BodyStep::Apply {proposal}=>{
                let core=records.core(proposal)?;
                if core.polity!=producer.polity || core.actor!=actor || core.operation.process!=producer.process
                    || target!=core.request.target() || !self.institution_ready(&core) {
                    return Err("institution application lacks its exact native executor, operation or admitted decisions".into());
                }
                if let Request::Amend {constitution,..}=&core.request {self.check_constitution(&core.constitution,constitution)?;}
                core
            },
        };
        self.institution_support(&core.operation)?;
        let cause=match &producer.step {BodyStep::Approve {cause,..}=>cause.clone(),_=>core.request.cause()};
        if cause.as_ref().is_some_and(|cause|!self.state.events.contains_key(cause)) {
            return Err("institution producer causal event is absent".into());
        }
        Ok(BodyWrite {producer:key.into(),declaration:producer.clone(),core,polity_before:polity.clone(),
            target,cause,action:String::new(),act:None})
    }

    pub(crate) fn institution_producer_ready(&self,actor:Id,target:Option<Id>,key:&str) -> Result<String> {
        let input=self.producer_input(actor,target,key)?;
        Ok(format!("scheduled constitutional body ready: {}",input.core.polity.0))
    }

    pub(crate) fn institution_process_scope(&self,actor:Id,target:Option<Id>,p:&Process) -> Result<Option<DecisionContext>> {
        let Some(key)=p.requires.iter().find_map(|q|match q {Query::Institution {producer}=>Some(producer),_=>None}) else {return Ok(None);};
        let input=self.producer_input(actor,target,key)?;
        let phase=match &input.declaration.step {
            BodyStep::Approve {..} if matches!(input.core.operation.decision,DecisionPolicy::Governance {..})=>DecisionPhase::Approval,
            BodyStep::Apply {..}=>DecisionPhase::Application,
            _=>return Ok(None),
        };
        Ok(Some(self.institution_context(&input.core,actor,target,&p.id,phase)))
    }

    pub(crate) fn process_query(&self,actor:Id,target:Option<Id>,place:Id,part:Option<PartId>,p:&Process,q:&Query) -> Result<String> {
        if matches!(q,Query::InstitutionDecision)
            && p.requires.iter().any(|gate|matches!(gate,Query::Institution {..})) {
            let input=self.institution_process_scope(actor,target,p)?.ok_or("no explicit constitutional deciding producer")?;
            return self.institution_context_ready(&input,actor,target);
        }
        self.query(actor,target,place,part,q)
    }

    pub(crate) fn institution_process_cause(&self,actor:Id,target:Option<Id>,p:&Process) -> Result<Option<Key>> {
        let Some(key)=p.requires.iter().find_map(|q|match q {Query::Institution {producer}=>Some(producer),_=>None}) else {return Ok(None);};
        Ok(self.producer_input(actor,target,key)?.cause)
    }

    pub(crate) fn institution_target(&self,actor:Id,p:&Process) -> Option<Result<Option<Id>>> {
        let key=p.requires.iter().find_map(|q|match q {Query::Institution {producer}=>Some(producer),_=>None})?;
        Some((|| {
            let producer=self.genesis.rules.institution_producers.get(key).ok_or("institution producer is absent")?;
            if producer.actor!=actor {return Err("institution target belongs to another native actor".into());}
            let target=match &producer.step {
                BodyStep::Propose {request,..}=>request.target(),
                BodyStep::Approve {target,..}=>*target,
                BodyStep::Apply {proposal}=>self.institution_records_at_start().core(proposal)?.request.target(),
            };
            self.producer_input(actor,target,key)?;
            if p.target.is_some()!=target.is_some() || !self.target_matches(actor,target,p) {
                return Err("institution request target does not satisfy its actual declared native scope".into());
            }
            let body=self.body_at_start(actor).ok_or("institution actor is absent")?;
            let part=self.bind_part(actor,p);
            for query in &p.requires {self.process_query(actor,target,body.place,part,p,query)?;}
            Ok(target)
        })())
    }

    pub(crate) fn institution_identity_roots(&self) -> BTreeSet<Id> {
        let mut roots=BTreeSet::new();
        for producer in self.genesis.rules.institution_producers.values() {
            roots.insert(producer.actor);
            match &producer.step {
                BodyStep::Propose {request,..}=>roots.extend(request.target()),
                BodyStep::Approve {target,..}=>roots.extend(target),
                _=>{},
            }
        }
        let records=&self.state.institutions;
        for key in records.proposals.keys().chain(records.body_proposals.keys()) {
            if let Ok(core)=records.core(key) {roots.insert(core.actor);roots.extend(core.request.target());roots.extend(core.electorate);}
            if let Ok(voters)=records.voters(key) {roots.extend(voters);}
        }
        roots.retain(|id|self.state.population.get(*id).is_some());
        roots
    }
}
