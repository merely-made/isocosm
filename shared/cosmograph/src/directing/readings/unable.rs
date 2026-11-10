// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Why not (case 1233): the acts a body cannot do now, and why. A pure
//! reading of the gates the chooser reads; nothing is kept.

use crate::{rules::Causation, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};

/// A Choice process the critter is not offered now, and why (case 1233).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unable {
    pub process: Key,
    pub target: Option<Id>,
    pub why: String,
}

/// Every Choice process `critter` would weigh but finds infeasible, in
/// process-key order, with the reason the act path gives where it is
/// attempted (case 1233). A process whose need is already met is no act the
/// critter is unable to do, so it is left out.
pub fn unable(sim: &Simulation, critter: Id) -> Vec<Unable> {
    let Some(place) = sim.body_at_start(critter).map(|e| e.place) else {
        return vec![];
    };
    let live = sim.live_nudges(critter, sim.state().tick);
    sim.genesis()
        .rules
        .processes
        .values()
        .filter(|p| p.causation == Causation::Choice && !sim.satisfied(critter, p))
        .filter_map(|p| {
            let (_, target) = sim.aim(critter, p, &live, place);
            let why = sim.infeasible(critter, target, p, place)?;
            Some(Unable { process: p.id.clone(), target, why: why.into_owned() })
        })
        .collect()
}
