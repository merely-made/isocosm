// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What happened to a surface between two states (case 1224): a pure
//! reading of the two states, never kept. Territories are entities and have
//! their own events, so only surfaces are covered here.

use super::*;
use crate::{rules::Rules, schema::{Id, Key}, simulation::{Simulation, State}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Bound::{Excluded, Unbounded};

/// Something that happened to a surface between two states (case 1224).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Surfaced {
    /// It holds matter at a substrate where it held none.
    Reached { identity: Identity, at: Substrate },
    /// It left a substrate it held matter at.
    Left { identity: Identity, at: Substrate },
    /// Its matter at a substrate met or passed its capacity there.
    Brimmed { identity: Identity, at: Substrate },
    /// The host it holds matter in died.
    HostDied { identity: Identity, host: Id },
    /// It shares a substrate with another surface it did not share it with before.
    Met { identity: Identity, other: Identity, at: Substrate },
}

impl Surfaced {
    /// Identity key, substrate, kind, then the other party: the report order.
    fn order(&self) -> (Key, Substrate, u8, Key) {
        match self {
            Self::Reached { identity, at } => (identity.key(), *at, 0, Key::new()),
            Self::Left { identity, at } => (identity.key(), *at, 1, Key::new()),
            Self::Brimmed { identity, at } => (identity.key(), *at, 2, Key::new()),
            Self::HostDied { identity, host } => (identity.key(), Substrate::Host(*host), 3, Key::new()),
            Self::Met { identity, other, at } => (identity.key(), *at, 4, other.key()),
        }
    }
}

type Holdings = BTreeSet<(Substrate, Identity)>;

/// Every surface stock a state holds: which identity holds matter where.
fn holdings(state: &State) -> Holdings {
    transport::stocks(state).into_iter()
        .filter(|(_, identity)| matches!(identity, Identity::Surface(_))).collect()
}

fn together(held: &Holdings) -> BTreeMap<Substrate, BTreeSet<Identity>> {
    let mut at: BTreeMap<Substrate, BTreeSet<Identity>> = BTreeMap::new();
    for (substrate, identity) in held { at.entry(*substrate).or_default().insert(identity.clone()); }
    at
}

fn amount(state: &State, at: Substrate, identity: &Identity) -> u64 {
    stock::ledger(state, at).ok().and_then(|ledger| ledger.get(&identity.key())).copied().unwrap_or(0)
}

fn gone(state: &State, host: Id) -> bool {
    state.population.get(host).is_none_or(|entity| !entity.alive)
}

/// What happened to every surface between `before` and `after`, in order of
/// identity key, then substrate, then kind (case 1224). A host that died is
/// reported as `HostDied`, whether its body is kept or gone, never as `Left`.
pub fn happened(rules: &Rules, before: &State, after: &State) -> Vec<Surfaced> {
    let (was, is) = (holdings(before), holdings(after));
    let mut found = Vec::new();
    for (at, identity) in is.difference(&was) {
        found.push(Surfaced::Reached { identity: identity.clone(), at: *at });
    }
    for (at, identity) in &was {
        match at {
            Substrate::Host(host) if gone(after, *host) && !gone(before, *host) =>
                found.push(Surfaced::HostDied { identity: identity.clone(), host: *host }),
            _ if !is.contains(&(*at, identity.clone())) =>
                found.push(Surfaced::Left { identity: identity.clone(), at: *at }),
            _ => {},
        }
    }
    if rules.spread_capacity.is_some() {
        for (at, identity) in &is {
            let Ok(limit) = capacity(after, rules, *at, identity) else { continue; };
            if amount(after, *at, identity) < limit { continue; }
            let below = capacity(before, rules, *at, identity)
                .map_or(true, |prior| amount(before, *at, identity) < prior);
            if below { found.push(Surfaced::Brimmed { identity: identity.clone(), at: *at }); }
        }
    }
    let (met, shared) = (together(&was), together(&is));
    for (at, present) in &shared {
        let prior = met.get(at);
        for first in present {
            for second in present.range::<Identity, _>((Excluded(first), Unbounded)) {
                if prior.is_some_and(|p| p.contains(first) && p.contains(second)) { continue; }
                found.push(Surfaced::Met { identity: first.clone(), other: second.clone(), at: *at });
                found.push(Surfaced::Met { identity: second.clone(), other: first.clone(), at: *at });
            }
        }
    }
    found.sort_by_key(Surfaced::order);
    found
}

/// `happened` over two simulations' current states (case 1224).
pub fn surfaced(before: &Simulation, after: &Simulation) -> Vec<Surfaced> {
    happened(&after.genesis().rules, before.state(), after.state())
}
