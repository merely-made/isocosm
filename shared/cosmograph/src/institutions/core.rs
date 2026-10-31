// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Constitutional inputs shared by participant and scheduled-body proposals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalCore {
    pub key: Key, pub actor: Id, pub polity: PolityId, pub constitution: Constitution,
    pub request: Request, pub operation: Operation, pub electorate: BTreeSet<Id>,
    pub approvals: u64, pub created: Tick,
}

impl From<&Proposal> for ProposalCore {
    fn from(p: &Proposal) -> Self {
        Self { key:p.key.clone(),actor:p.actor,polity:p.polity,constitution:p.constitution.clone(),
            request:p.request.clone(),operation:p.operation.clone(),electorate:p.electorate.clone(),
            approvals:p.approvals,created:p.created }
    }
}

/// Every request is supplied by the author; this catalogue generates no agenda.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyProducer { pub actor: Id, pub process: Key, pub polity: PolityId, pub step: BodyStep }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BodyStep {
    Propose { proposal: Key, request: Request },
    Approve { proposal: Key, target: Option<Id>, cause: Option<Key> },
    Apply { proposal: Key },
}

impl BodyStep {
    pub(super) fn proposal(&self) -> &str {
        match self { Self::Propose {proposal,..}|Self::Approve {proposal,..}|Self::Apply {proposal} => proposal }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyProposal {
    pub core: ProposalCore, pub act: ActWitness,
    /// Genuine participant approvals may join a body's native proposal.
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub votes: BTreeMap<Id, Approval>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub applied: Option<Applied>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyVote { pub actor: Id, pub tick: Tick, pub act: ActWitness }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BodyApplied { pub tick: Tick, pub act: ActWitness }

impl Records {
    pub(super) fn core(&self, key: &str) -> Result<ProposalCore> {
        match (self.proposals.get(key), self.body_proposals.get(key)) {
            (Some(p), None) => Ok(p.into()),
            (None, Some(p)) => Ok(p.core.clone()),
            (None, None) => Err("institution proposal is absent".into()),
            _ => Err("institution proposal identity has conflicting origins".into()),
        }
    }
    pub(super) fn participant_votes(&self, key: &str) -> Result<&BTreeMap<Id,Approval>> {
        self.core(key)?;
        Ok(match self.proposals.get(key) { Some(p) => &p.votes, None => &self.body_proposals[key].votes })
    }
    pub(super) fn voters(&self, key: &str) -> Result<BTreeSet<Id>> {
        let votes=self.participant_votes(key)?;
        let mut voters:BTreeSet<_>=votes.keys().copied().collect();
        for actor in self.body_votes.get(key).into_iter().flat_map(|votes| votes.keys()) {
            if !voters.insert(*actor) { return Err("institution voter has conflicting participant/body approvals".into()); }
        }
        Ok(voters)
    }
    pub(super) fn is_applied(&self, key: &str) -> Result<bool> {
        self.core(key)?;
        let participant=match self.proposals.get(key) { Some(p) => p.applied.as_ref(),None => self.body_proposals[key].applied.as_ref() };
        if participant.is_some() && self.body_applied.contains_key(key) {
            return Err("institution application has conflicting participant/body origins".into());
        }
        Ok(participant.is_some() || self.body_applied.contains_key(key))
    }
    pub(super) fn store_vote(&mut self,key:&str,vote:Approval) -> Result<()> {
        if self.is_applied(key)? || self.voters(key)?.contains(&vote.actor) { return Err("institution body has already voted or the operation is applied".into()); }
        match self.proposals.get_mut(key) { Some(p) => {p.votes.insert(vote.actor,vote);},
            None => {self.body_proposals.get_mut(key).expect("core checked above").votes.insert(vote.actor,vote);} }
        Ok(())
    }
    pub(super) fn store_applied(&mut self,key:&str,applied:Applied) -> Result<()> {
        if self.is_applied(key)? { return Err("institution operation is already applied".into()); }
        match self.proposals.get_mut(key) { Some(p) => p.applied=Some(applied),
            None => self.body_proposals.get_mut(key).expect("core checked above").applied=Some(applied) }
        Ok(())
    }
}
