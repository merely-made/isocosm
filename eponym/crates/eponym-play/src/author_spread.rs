// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Author-only references resolve accepted native actors, never future IDs.

use crate::identity::SubjectId;
use isocosm::{schema::{Id, Key}, projects::{DebitSelection, DecisionPolicy, Owner},
    spread::{TransitId, Identity, Placement, Supply, Substrate}};
use isometer_space::places::PlaceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorRef { Native(Id), Founder(Key), Subject(SubjectId) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolderRef { Site(Id), Place(PlaceId), Host(ActorRef), Transit(TransitId) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityRef { Territory(ActorRef), Surface(Key) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AuthorSupply {
    Matter { from: HolderRef, account: Key },
    Spread { from: HolderRef, identity: IdentityRef },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorPlacement {
    pub source: AuthorSupply, pub identity: IdentityRef, pub at: HolderRef, pub amount: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum AuthorSpreadStep {
    Register { identity: IdentityRef },
    Place { placement: AuthorPlacement, selections: Vec<DebitSelection> },
    Overflow { identity: IdentityRef, at: HolderRef, selections: Vec<DebitSelection> },
    ForkStrain { parent: Key, at: HolderRef, amount: u64, traits: BTreeSet<Key>, selections: Vec<DebitSelection> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorSpreadLot {
    pub key: Key,
    pub source: HolderRef,
    pub amount: u64,
    pub owners: BTreeSet<Owner>,
    pub collectives: BTreeMap<Id, DecisionPolicy>,
}

pub(crate) fn substrate(source: isocosm::projects::StockSource) -> Result<Substrate, crate::WorldError> {
    use isocosm::projects::StockSource;
    match source {
        StockSource::Entity(id) => Ok(Substrate::Host(id)),
        StockSource::Site(id) => Ok(Substrate::Site(id)),
        StockSource::Place(id) => Ok(Substrate::Place(id)),
        StockSource::Transit(_) => Err(crate::WorldError::Refused),
    }
}

pub(crate) fn placement(source: Supply, identity: Identity, at: Substrate, amount: u64) -> Placement {
    Placement { source, identity, at, amount }
}
