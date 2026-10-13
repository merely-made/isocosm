// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The existing paid limb-action reducer over borrowed current anatomy.
//! Its network/charge belongs to the product, never to another native runtime.

use super::*;
use crate::timed_action::{TimedAction,Contribution,ContributionState,LimbBinding,StrikeReceipt};
use wing_functions::{NodeKind,NodeId,PartRef,Operator,EvaluationRequest,EvaluationError,EffectReport};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChargeState {
    pub subject:SubjectId, pub network:FunctionalNetwork, pub action:Option<TimedAction>,
}

impl ChargeState {
    pub(super) fn validate(&self, policy:&CombatPolicy) -> Result<(),String> {
        self.network.validate().map_err(debug)?;
        // Charge and gate state may vary; node identity, capacity and edges may not.
        if self.network.edges != policy.network.edges || self.network.nodes.len() != policy.network.nodes.len() {
            return Err("Eponym charged network differs from its authored topology".into());
        }
        for (id,node) in &self.network.nodes {
            let authored = policy.network.nodes.get(id).ok_or("unknown charged Eponym node")?;
            let same = match (&node.kind,&authored.kind) {
                (NodeKind::Source{part,capacity,..},NodeKind::Source{part:p,capacity:c,..})
                | (NodeKind::Store{part,capacity,..},NodeKind::Store{part:p,capacity:c,..}) => part == p && capacity == c,
                (NodeKind::Gate{part,..},NodeKind::Gate{part:p,..})
                | (NodeKind::Effect{part},NodeKind::Effect{part:p}) => part == p,
                _ => false,
            };
            if !same { return Err("Eponym charged node differs from its authored source".into()); }
        }
        if let Some(action) = &self.action {
            if action.subject != self.subject || action.contributors.is_empty()
                || action.contributors.len() > policy.timed.max_contributors || action.last_tick < action.started_at
                || action.last_tick.0-action.started_at.0 > policy.timed.max_elapsed_ticks {
                return Err("invalid Eponym prepared action".into());
            }
            for (part,c) in &action.contributors {
                if *part != c.binding.part || c.charge > policy.timed.max_charge_per_limb
                    || !matches!(self.network.nodes.get(&c.node).map(|n|&n.kind),
                        Some(NodeKind::Effect{part:p}) if *p == PartRef{subject:self.subject.0,part:part.0}) {
                    return Err("Eponym prepared contribution has no authored current node".into());
                }
            }
        }
        Ok(())
    }

    pub(super) fn prepare(&mut self, record:&AnatomyRecord, direction:Direction, now:Tick) -> Result<String,String> {
        if self.action.is_some() { return Err("Eponym action already prepared".into()); }
        let (node,part) = self.network.nodes.iter().find_map(|(id,node)| match node.kind {
            NodeKind::Effect{part} if part.subject == self.subject.0 && record.document.part(PartId(part.part))
                .is_some_and(|p|!p.severed) => Some((*id,PartId(part.part))), _=>None,
        }).ok_or("Eponym action has no available authored limb")?;
        self.action = Some(TimedAction {subject:self.subject,direction,started_at:now,last_tick:now,
            contributors:[(part,Contribution {binding:LimbBinding {part,revision:record.revision},
                node,charge:0,state:ContributionState::Charging})].into()});
        Ok(format!("prepared {direction:?}"))
    }

    pub(super) fn join(&mut self, record:&AnatomyRecord, policy:&CombatPolicy, part:PartId) -> Result<String,String> {
        let action = self.action.as_mut().ok_or("Eponym action is not prepared")?;
        if action.contributors.contains_key(&part) || action.contributors.len() >= policy.timed.max_contributors
            || !record.document.part(part).is_some_and(|p|!p.severed) {
            return Err("Eponym contribution is duplicated, unavailable or at its limit".into());
        }
        let node = self.network.nodes.iter().find_map(|(id,node)|
            matches!(node.kind,NodeKind::Effect{part:p} if p == PartRef{subject:self.subject.0,part:part.0}).then_some(*id))
            .ok_or("Eponym selected limb lacks an authored effect node")?;
        action.contributors.insert(part,Contribution {binding:LimbBinding {part,revision:record.revision},
            node,charge:0,state:ContributionState::Charging});
        Ok(format!("joined part {}",part.0))
    }

    pub(super) fn reconcile(&mut self, record:&AnatomyRecord) {
        if let Some(action) = &mut self.action {
            for contribution in action.contributors.values_mut() {
                if record.document.part(contribution.binding.part).is_some_and(|p|!p.severed) {
                    contribution.binding.revision = record.revision;
                } else { contribution.state = ContributionState::Cancelled; }
            }
        }
    }

    pub(super) fn charge(&mut self, record:&AnatomyRecord, policy:&CombatPolicy, tick:Tick) -> Result<String,String> {
        self.reconcile(record);
        let action = self.action.as_mut().ok_or("Eponym action is not prepared")?;
        if action.last_tick.0.checked_add(1) != Some(tick.0)
            || tick.0-action.started_at.0 > policy.timed.max_elapsed_ticks {
            return Err("wrong Eponym charge tick or elapsed limit".into());
        }
        let live = live(record);
        let mut outcomes = vec![];
        for contribution in action.contributors.values_mut() {
            if contribution.state == ContributionState::Cancelled || contribution.charge == policy.timed.max_charge_per_limb {
                continue;
            }
            let amount = policy.timed.charge_per_tick.min(policy.timed.max_charge_per_limb-contribution.charge);
            let request = EvaluationRequest {operator:Operator::Strengthen,target:contribution.node,cost:amount,
                range:0,max_hops:policy.timed.evaluation.max_hops};
            match self.network.evaluate(&request,&live,&policy.timed.evaluation) {
                Ok(evaluation) => match evaluation.effect {
                    EffectReport::Strengthened{amount,..} => {
                        contribution.charge += amount; contribution.state = ContributionState::Charging;
                        outcomes.push(format!("part {} charged {amount}: {}",contribution.binding.part.0,contribution.charge));
                    }, _ => return Err("Eponym strengthen returned another effect".into()),
                },
                Err(EvaluationError::InsufficientCharge{..}|EvaluationError::ClosedGate{..}|EvaluationError::MissingPart(_)) => {
                    contribution.state = ContributionState::Paused;
                    outcomes.push(format!("part {} paused",contribution.binding.part.0));
                }, Err(error)=>return Err(debug(error)),
            }
        }
        action.last_tick = tick;
        Ok(outcomes.join("\n"))
    }

    pub(super) fn release(&mut self, record:&AnatomyRecord, policy:&CombatPolicy, tick:Tick) -> Result<Vec<StrikeReceipt>,String> {
        self.reconcile(record);
        let action = self.action.take().ok_or("Eponym action is not prepared")?;
        if tick < action.last_tick || tick.0-action.started_at.0 > policy.timed.max_elapsed_ticks {
            return Err("Eponym release has a stale tick or exceeds elapsed limit".into());
        }
        let live = live(record);
        Ok(action.contributors.into_values().filter(|c|c.state != ContributionState::Cancelled && c.charge > 0
            && route_open(&self.network,c.node,&live,policy.timed.evaluation.max_hops))
            .map(|c|StrikeReceipt {binding:c.binding,direction:action.direction,charge:c.charge}).collect())
    }

    pub(super) fn gate(&mut self, node:NodeId, open:bool) -> Result<String,String> {
        match self.network.nodes.get_mut(&node).map(|n|&mut n.kind) {
            Some(NodeKind::Gate{open:state,..}) => {*state = open;Ok(format!("gate {}: {open}",node.0))},
            _=>Err("Eponym gate input does not name an authored gate".into()),
        }
    }
}

fn live(record:&AnatomyRecord) -> BTreeSet<PartRef> {
    record.document.parts.iter().filter(|p|!p.severed)
        .map(|p|PartRef {subject:record.subject.0,part:p.id.0}).collect()
}
fn route_open(network:&FunctionalNetwork,target:NodeId,live:&BTreeSet<PartRef>,max_hops:u32) -> bool {
    let mut queue:std::collections::VecDeque<_> = network.nodes.iter().filter_map(|(id,node)|
        matches!(node.kind,NodeKind::Source{..}|NodeKind::Store{..}).then_some((*id,0))).collect();
    let mut seen = BTreeSet::new();
    while let Some((id,hops)) = queue.pop_front() {
        if !seen.insert(id) { continue; }
        let Some(node) = network.nodes.get(&id) else { continue; };
        if !live.contains(&node.kind.part()) || matches!(node.kind,NodeKind::Gate{open:false,..}) { continue; }
        if id == target { return true; }
        if hops == max_hops { continue; }
        queue.extend(network.edges.iter().filter(|e|e.from == id).map(|e|(e.to,hops+1)));
    }
    false
}
