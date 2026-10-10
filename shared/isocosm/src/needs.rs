// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a body wants, read from its accounts (wing ruling 767): a need is
//! the world mind's [`Need`](crate::rules::Need), and how far a body is
//! from meeting it is read from what it holds, never kept. Eponym's hunger
//! reads the matter reserve and its fatigue `Energy`; any world may declare
//! others the same way.

use crate::{anatomy, rules::Rules, schema::*};
use serde::{Deserialize, Serialize};

/// One need as a body stands toward it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Want {
    pub account: Key,
    /// The level below which the need is felt.
    pub below: u64,
    pub held: u64,
    pub weight: i64,
}

impl Want {
    /// How far below the level the body holds, nought when met.
    pub fn deficit(&self) -> u64 {
        self.below.saturating_sub(self.held)
    }
    pub fn felt(&self) -> bool {
        self.held < self.below
    }
}

/// Every need of the world's mind that reads a level of `e`'s own account
/// and whose traits `e` carries, in declared order.
pub fn wants(rules: &Rules, e: &Entity) -> Vec<Want> {
    let Some(mind) = &rules.mind else {
        return vec![];
    };
    mind.needs
        .iter()
        .filter(|n| n.traits.is_subset(&e.traits))
        .filter_map(|n| match &n.query {
            crate::rules::Query::Below {
                who: crate::rules::Binding::Actor,
                key,
                amount,
            } => Some(Want {
                account: key.clone(),
                below: *amount,
                held: anatomy::held(e, rules, key),
                weight: n.weight,
            }),
            _ => None,
        })
        .collect()
}

/// The want reading `account`, if the world's mind declares one for `e`.
pub fn want(rules: &Rules, e: &Entity, account: &str) -> Option<Want> {
    wants(rules, e).into_iter().find(|w| w.account == account)
}
