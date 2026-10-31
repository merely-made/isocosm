// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::projects::{DecisionPolicy, DebitSelection, Owner};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PolityId(pub Id);

/// Each source is read independently. No aggregation or issuance supplies support.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Support { pub source: Owner, pub account: Key, pub minimum: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation { pub process: Key, pub decision: DecisionPolicy, pub support: Vec<Support> }

/// The constitution names this authored method catalogue by its governance key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub operations: BTreeMap<Key, Operation>, pub amend: Option<Operation>, pub end: Option<Operation>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Request {
    Act { operation: Key, target: Option<Id>, cause: Option<Key> },
    Amend { constitution: Constitution, target: Option<Id>, cause: Option<Key> },
    End { target: Option<Id>, cause: Option<Key> },
}

impl Request {
    pub(super) fn target(&self) -> Option<Id> {
        match self { Self::Act { target,.. } | Self::Amend { target,.. } | Self::End { target,.. } => *target }
    }
    pub(super) fn cause(&self) -> Option<Key> {
        match self { Self::Act { cause,.. } | Self::Amend { cause,.. } | Self::End { cause,.. } => cause.clone() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Change {
    Propose { key: Key, participant: Id, actor: Id, polity: PolityId, request: Request },
    Approve { participant: Id, actor: Id, proposal: Key, process: Option<Key>, target: Option<Id>,
        cause: Option<Key>, selections: Vec<DebitSelection> },
    Apply { participant: Id, actor: Id, proposal: Key, selections: Vec<DebitSelection> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ActKind { Decision, Operation, Amendment, Dissolution, Foundation, Proposal }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActWitness {
    pub polity: PolityId, pub actor: Id, pub event: Key, pub process: Key, pub kind: ActKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval { pub participant: Id, pub actor: Id, pub tick: Tick, pub act: Option<ActWitness> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applied { pub participant: Id, pub tick: Tick, pub act: ActWitness }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub key: Key, pub participant: Id, pub actor: Id, pub polity: PolityId,
    pub constitution: Constitution, pub request: Request, pub operation: Operation,
    pub electorate: BTreeSet<Id>, pub approvals: u64, pub created: Tick,
    pub votes: BTreeMap<Id, Approval>, pub applied: Option<Applied>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Records {
    pub proposals: BTreeMap<Key, Proposal>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub body_proposals: BTreeMap<Key, BodyProposal>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub body_votes: BTreeMap<Key, BTreeMap<Id, BodyVote>>,
    #[serde(default, skip_serializing_if="BTreeMap::is_empty")]
    pub body_applied: BTreeMap<Key, BodyApplied>,
}

impl Records { pub fn is_empty(&self) -> bool {
    self.proposals.is_empty() && self.body_proposals.is_empty() && self.body_votes.is_empty() && self.body_applied.is_empty()
} }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome { pub proposal: Key, pub ready: bool, pub applied: Option<Applied>, pub decision: Option<ActWitness> }
