// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Directing's readings (D3), each a pure function of the world and its
//! log: suggestions (685), standing orders (690), regions (687, 689) and
//! the survival filter (691). None is kept.

use crate::{schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};

pub mod habitable;
pub mod orders;
pub mod regions;
pub mod survival;

pub use habitable::habitable;
pub use orders::{Orders, orders};
pub use regions::{Region, region_of, regions};
pub use survival::{Mode, View, view};

/// An act the critter considered and could take beside its choice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub process: Key,
    pub target: Option<Id>,
    pub score: i64,
}

/// What `critter` could do now beside what it would choose, as a receipt's
/// `foregone` names it, filtered through what it knows (685), best first.
pub fn suggestions(sim: &Simulation, critter: Id) -> Vec<Suggestion> {
    let considered = sim.consider(critter, false);
    let mut all: Vec<Suggestion> = considered
        .options
        .into_iter()
        .filter(|(p, _)| considered.process.as_ref() != Some(p))
        .map(|(process, (score, target))| Suggestion {
            process,
            target,
            score,
        })
        .collect();
    all.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.process.cmp(&b.process))
    });
    view(sim, critter, Mode::Survival).filter(all)
}

/// The suggestions a receipt's `foregone` names, as the critter knows them.
pub fn from_receipt(sim: &Simulation, receipt: &crate::simulation::Receipt) -> Vec<Suggestion> {
    let all = suggestions(sim, receipt.actor);
    let named = |s: &Suggestion| receipt.foregone.contains(&s.process);
    all.into_iter().filter(named).collect()
}

#[cfg(test)]
mod tests;
