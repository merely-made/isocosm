// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a body wants, read from its accounts (wing ruling 767): a need is
//! the world mind's [`Need`](crate::rules::Need), and how far a body is
//! from meeting it is read from what it holds, never kept. Eponym's hunger
//! reads the matter reserve and its fatigue `Energy`; any world may declare
//! others the same way.

use crate::{anatomy, rules::{Binding, Expr, Query, Reading, Rules}, schema::*};
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

/// The actor account read by a fixed level or an unfilled-capacity need.
pub fn account(query: &Query) -> Option<&Key> {
    match query {
        Query::Below { who: Binding::Actor, key, .. } => Some(key),
        Query::Computed(Expr::AtLeast(left, right)) if **right == Expr::Const(1) => {
            match left.as_ref() {
                Expr::Read(Reading::Room { who: Binding::Actor, key }) => Some(key),
                _ => None,
            }
        }
        _ => None,
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
        .filter_map(|n| {
            let key = account(&n.query)?;
            let held = anatomy::held(e, rules, key);
            let below = match &n.query {
                Query::Below { amount, .. } => *amount,
                _ => held.saturating_add(anatomy::room(e, rules, key)),
            };
            Some(Want { account: key.clone(), below, held, weight: n.weight })
        })
        .collect()
}

/// The want reading `account`, if the world's mind declares one for `e`.
pub fn want(rules: &Rules, e: &Entity, account: &str) -> Option<Want> {
    wants(rules, e).into_iter().find(|w| w.account == account)
}
