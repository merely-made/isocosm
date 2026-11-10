// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a holder sees of a lineage it holds, and what it could hold (cases
//! 1216, 1217, 1221, 1223): a pure reading of the world, never kept. A
//! surface is matter under its lineage's key, so its extent is counted
//! where that matter lies.

use crate::{
    schema::*,
    simulation::{Simulation, State},
    spread::{self, Identity, Substrate},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// How many substrates of one kind hold a surface's matter, and how much.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub count: u64,
    pub total: u64,
}

/// A held surface lineage as its holder sees it (cases 1223, 1224).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    pub lineage: Key,
    pub traits: BTreeSet<Key>,
    pub revision: u64,
    pub sites: Tally,
    pub places: Tally,
    pub hosts: Tally,
    /// Matter in transit between substrates.
    pub transit: u64,
    /// It holds no matter on any substrate and none in transit.
    pub ended: bool,
}

/// Whether the surface `lineage` holds no matter anywhere nor in transit.
pub(crate) fn ended(state: &State, lineage: &str) -> bool {
    let identity = Identity::Surface(lineage.into());
    !state.spreads.transit.iter().any(|t| t.identity == identity && t.amount > 0)
        && !spread::stocks(state).iter().any(|(_, held)| *held == identity)
}

/// The surface `lineage` as it stands; none if it is not a registered surface.
pub fn held(sim: &Simulation, lineage: &str) -> Option<Held> {
    let state = sim.state();
    let identity = Identity::Surface(lineage.into());
    let line = state.lineages.get(lineage).filter(|_| state.spreads.identities.contains(&identity))?;
    let (mut sites, mut places, mut hosts) = (Tally::default(), Tally::default(), Tally::default());
    for (at, _) in spread::stocks(state).into_iter().filter(|(_, held)| *held == identity) {
        let total = spread::held(state, at, &identity).unwrap_or(0);
        let tally = match at {
            Substrate::Site(_) => &mut sites,
            Substrate::Place(_) => &mut places,
            Substrate::Host(_) => &mut hosts,
        };
        tally.count += 1;
        tally.total = tally.total.saturating_add(total);
    }
    let transit = state.spreads.transit.iter().filter(|t| t.identity == identity).map(|t| t.amount).sum();
    Some(Held {
        lineage: lineage.into(),
        traits: line.traits.clone(),
        revision: line.revision,
        sites,
        places,
        hosts,
        transit,
        ended: ended(state, lineage),
    })
}

/// The lineages a participant could hold, in key order: the registered
/// surfaces that hold matter, and the world-class lineages.
pub fn holdable(sim: &Simulation) -> Vec<Key> {
    let state = sim.state();
    let mut out: BTreeSet<Key> = state.lineages.iter()
        .filter(|(_, l)| l.kingdom == crate::directing::WORLD).map(|(key, _)| key.clone()).collect();
    out.extend(state.spreads.identities.iter().filter_map(|identity| match identity {
        Identity::Surface(key) if state.lineages.contains_key(key) && !ended(state, key) => Some(key.clone()),
        _ => None,
    }));
    out.into_iter().collect()
}
