// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Standing orders (rulings 216, 59, 690): desire paths read from the
//! logged nudges and how each was answered, never kept or set. Where the
//! player's attention keeps leading and the critter thrives becomes its
//! range and its home; what the player keeps pointing at rises among its
//! priorities; places that went badly are avoided (215). *Reading, not
//! ruled:* stances grow the same way, by the lineage of what it acted on.

use super::super::{Aim, Nudge, Toward};
use crate::{schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Orders {
    /// Sites attention led to, each with how it went there.
    pub range: BTreeMap<Id, i64>,
    pub home: Option<Id>,
    /// Acts the player pointed at, by how they served.
    pub priorities: BTreeMap<Key, i64>,
    /// Lineages acted on, by how it went.
    pub stances: BTreeMap<Key, i64>,
    pub avoid: BTreeSet<Id>,
}

/// The standing orders `participant`'s nudges have grown for `critter`.
pub fn orders(sim: &Simulation, participant: Id, critter: Id) -> Orders {
    let s = sim.state();
    let mine = |n: &&Nudge| n.participant == participant && n.critter == critter;
    let mut attention: BTreeMap<Id, i64> = BTreeMap::new();
    let mut thrived: BTreeMap<Id, i64> = BTreeMap::new();
    let mut o = Orders::default();
    for n in s.nudges.iter().filter(mine) {
        let named = match n.toward {
            Toward::Site(site) => Some(site),
            Toward::Thing(_) => n.answer.as_ref().map(|a| a.site),
        };
        if let Some(site) = named {
            *attention.entry(site).or_default() += 1;
        }
        let Some(a) = &n.answer else {
            continue;
        };
        let went = if a.served { 1 } else { -1 };
        *thrived.entry(a.site).or_default() += went;
        if n.aim == Aim::Act {
            *o.priorities.entry(a.process.clone()).or_default() += went;
            if let Toward::Thing(t) = n.toward
                && let Some(e) = s.population.get(t)
            {
                *o.stances.entry(e.lineage.clone()).or_default() += went;
            }
        }
    }
    for (&site, &went) in &thrived {
        if went < 0 {
            o.avoid.insert(site);
        }
    }
    for (&site, &count) in &attention {
        let went = thrived.get(&site).copied().unwrap_or(0);
        if went >= 0 {
            o.range.insert(site, count + went);
        }
    }
    // Home is where attention and thriving meet most, the lowest site
    // among equals.
    let top = o
        .range
        .iter()
        .filter(|(_, v)| **v > 0)
        .max_by_key(|(s, v)| (**v, std::cmp::Reverse(**s)));
    o.home = top.map(|(s, _)| *s);
    o
}
