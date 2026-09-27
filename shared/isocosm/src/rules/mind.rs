// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What the core reads of minds: needs and the mind (rulings 158, 159, 163,
//! 164 and 227).

use super::Query;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A need, as the core has needs now (ruling 227): members carrying every
/// one of `traits` for whom `query` holds have their mood moved by `weight`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Need {
    pub traits: BTreeSet<Key>,
    pub query: Query,
    pub weight: i64,
}

/// What the core reads of a mind now. Mood is read from needs and never
/// kept; strain is kept in its account (rulings 158, 159 and 227). A mind
/// bears strain up to its bearing, set by its traits (ruling 164); past it,
/// it breaks, up with the chance its traits and the moment give (ruling 163).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mind {
    pub strain: Key,
    pub needs: Vec<Need>,
    pub bearing: i64,
    pub bearing_traits: BTreeMap<Key, i64>,
    /// Per mille chance that a break goes up.
    pub rise: i64,
    pub rise_traits: BTreeMap<Key, i64>,
    /// The moment: per mille added to the rise for each point of mood.
    pub stake: i64,
}
