// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::projects::{DebitRequest, DebitSelection, LotQuantity, StockSource};

/// Actual stock quantities approved by the player. Native transient request
/// identities are obtained again at Apply, never supplied or waived here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DebitTerm {
    pub operation: u64, pub ordinal: u64, pub source: StockSource,
    pub account: Key, pub amount: u64, pub lots: Vec<LotQuantity>, pub unowned: u64,
}

impl DebitTerm {
    pub(super) fn matches(&self, request: &DebitRequest) -> bool {
        self.operation == request.operation && self.ordinal == request.ordinal
            && self.source == request.source && self.account == request.account
            && self.amount == request.amount
    }
    pub(super) fn selection(&self, request: DebitRequest) -> Result<DebitSelection, String> {
        if !self.matches(&request) { return Err("journey stock quote changed since approval".into()); }
        Ok(DebitSelection { request, lots:self.lots.clone(), unowned:self.unowned })
    }
}

/// Every identity and choice is explicit. No faction position, generated
/// process, guessed cost or default stock allocation supplies a departure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepartureInput {
    pub participant: Id, pub body: Id, pub from: Id, pub to: Id,
    pub process: Key, pub profile: Key, pub target: Option<Id>, pub debits: Vec<DebitTerm>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum JourneyInput {
    Propose { party: Key, departures: Vec<DepartureInput> },
    Approve { proposal: EventHandle },
    Apply { proposal: EventHandle },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JourneyEnvelope { pub participant: Id, pub tick: Tick, pub intent: JourneyInput }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness { pub entry: Key, pub event_key: Key, pub event: EventHandle, pub tick: Tick }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Departure {
    pub input: DepartureInput, pub kind: DepartureKind,
    /// A native route/profile quote. For Goal this is not a promised departure.
    pub journey: isocosm::schema::Journey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepartureKind { Normative, Goal }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub proposer: Id, pub tick: Tick, pub party: Key,
    pub roster: super::super::time::Roster, pub departures: Vec<Departure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval { pub participant: Id, pub body: Id, pub witness: Witness }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DepartureResult {
    Departed { participant: Id, body: Id, entry: Key, journey: isocosm::schema::Journey },
    Goal { participant: Id, body: Id, entry: Key },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion { pub participant: Id, pub witness: Witness, pub departures: Vec<DepartureResult> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub declaration: Declaration, pub witness: Witness,
    pub approvals: Vec<Approval>, pub completed: Option<Completion>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) enum Record {
    Proposed(Declaration),
    Approved { participant: Id, body: Id, proposal: Key, event: EventHandle },
    Applied { participant: Id, proposal: Key, event: EventHandle, departures: Vec<DepartureResult> },
}
