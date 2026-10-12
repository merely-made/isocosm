// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native spread identity and place-owned stocks (488, 497–500).

use super::{Id, Key, Ledger, Tick};
use isometer_space::places::PlaceId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Identity {
    Territory(Id),
    Surface(Key),
}

impl Identity {
    /// Territory stock belongs to its individual identity; surface stock to
    /// the exact native lineage. Neither aliases an ordinary rule account.
    pub fn key(&self) -> Key {
        match self { Self::Territory(id) => format!("entity:{id}"), Self::Surface(key) => key.clone() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Substrate {
    Site(Id),
    Place(PlaceId),
    Host(Id),
}

/// A genuinely finer place owns this ledger. A site ID is not a PlaceId.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceLedger {
    pub place: PlaceId,
    pub accounts: Ledger,
}

/// Matter leaves the source only when this conserved pending transfer is
/// published. It is counted once until native arrival credits the destination.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transfer {
    pub identity: Identity,
    pub from: Id,
    pub to: Id,
    pub departed: Tick,
    pub arrives: Tick,
    pub amount: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spreads {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub identities: BTreeSet<Identity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub places: Vec<PlaceLedger>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transit: Vec<Transfer>,
}

impl Spreads {
    pub fn is_empty(&self) -> bool {
        self.identities.is_empty() && self.places.is_empty() && self.transit.is_empty()
    }
    pub fn owns(&self, key: &str) -> bool {
        self.identities.iter().any(|identity| identity.key() == key)
    }
}
