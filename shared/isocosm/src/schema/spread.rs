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
    /// Authored route origin at departure; the logged change still names the
    /// actual source Site, PlaceId or Host ledger that was debited.
    pub from: Id,
    pub to: Id,
    pub departed: Tick,
    pub arrives: Tick,
    pub amount: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TransitId(pub u64);

/// The index is a checked queue location; only the native ID names the transfer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitMark {
    pub id: TransitId,
    pub index: usize,
    pub source: Substrate,
    pub route: usize,
}

/// The source of an explicit capacity value; it is not another matter stock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CapacitySource { World, Substrate(Substrate) }

/// A world-declared capacity effect, resolved by its named layer. Identity
/// selection is explicit; absence applies the value to every registered spread.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacityModifier {
    pub key: Key,
    pub source: CapacitySource,
    pub identity: Option<Identity>,
    pub layer: Key,
    pub milligrams: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spreads {
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub identities: BTreeSet<Identity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub places: Vec<PlaceLedger>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transit: Vec<Transfer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<CapacityModifier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transit_marks: Vec<TransitMark>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub next_transit: u64,
}

impl Spreads {
    pub fn is_empty(&self) -> bool {
        self.identities.is_empty() && self.places.is_empty() && self.transit.is_empty()
            && self.modifiers.is_empty() && self.transit_marks.is_empty() && self.next_transit == 0
    }
    pub fn owns(&self, key: &str) -> bool {
        self.identities.iter().any(|identity| identity.key() == key)
    }
}

fn is_zero(n: &u64) -> bool { *n == 0 }
