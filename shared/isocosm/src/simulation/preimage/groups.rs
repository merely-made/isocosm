// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Every Character in a held storylet or forced pack uses its native
//! witness preimage, including those inside further nested groups (809).

use super::{Character, asserted};
use crate::schema::Key;
use serde::Serialize;
use std::collections::BTreeMap;

/// Variant order is the native Assertion order, hence the same postcard tags.
#[derive(Serialize)]
enum Assertion<'a> {
    Faction(&'a asserted::Faction),
    Fact(&'a asserted::Fact),
    Place(&'a asserted::Place),
    Route(&'a asserted::Route),
    Character(Character<'a>),
    Law(&'a asserted::Law),
    History(&'a asserted::HistoryLine),
    Edit(&'a asserted::MapEdit),
    Storylet(Applied<'a>),
    PackForced(Applied<'a>),
    StockOwnership(&'a crate::projects::StockLot),
    SpreadOwnership { identity: &'a crate::spread::Identity, lot: &'a crate::projects::StockLot },
}

impl<'a> From<&'a asserted::Assertion> for Assertion<'a> {
    fn from(value: &'a asserted::Assertion) -> Self {
        use asserted::Assertion as Native;
        match value {
            Native::Faction(v) => Self::Faction(v),
            Native::Fact(v) => Self::Fact(v),
            Native::Place(v) => Self::Place(v),
            Native::Route(v) => Self::Route(v),
            Native::Character(v) => Self::Character(v.into()),
            Native::Law(v) => Self::Law(v),
            Native::History(v) => Self::History(v),
            Native::Edit(v) => Self::Edit(v),
            Native::Storylet(v) => Self::Storylet(v.into()),
            Native::PackForced(v) => Self::PackForced(v.into()),
            Native::StockOwnership(v) => Self::StockOwnership(v),
            Native::SpreadOwnership { identity, lot } => Self::SpreadOwnership { identity, lot },
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Applied<'a> {
    key: &'a Key,
    source: &'a Key,
    asserts: Vec<Assertion<'a>>,
}

impl<'a> From<&'a asserted::Applied> for Applied<'a> {
    fn from(value: &'a asserted::Applied) -> Self {
        let asserted::Applied {
            key,
            source,
            asserts,
        } = value;
        Self {
            key,
            source,
            asserts: asserts.iter().map(Assertion::from).collect(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct Records<'a> {
    maps: &'a BTreeMap<Key, asserted::MapEdit>,
    storylets: BTreeMap<&'a Key, Applied<'a>>,
    packs: BTreeMap<&'a Key, Applied<'a>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    spread_ownership: &'a BTreeMap<Key, (crate::spread::Identity, crate::projects::StockLot)>,
}

impl<'a> From<&'a asserted::Records> for Records<'a> {
    fn from(value: &'a asserted::Records) -> Self {
        let asserted::Records {
            maps,
            storylets,
            packs,
            spread_ownership,
        } = value;
        Self {
            maps,
            storylets: storylets.iter().map(|(key, a)| (key, a.into())).collect(),
            packs: packs.iter().map(|(key, a)| (key, a.into())).collect(),
            spread_ownership,
        }
    }
}
