// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dev lane's follow tile over a native critter (DT2): who it is, where,
//! what it holds, and its first few parts. The words are this crate's.

use isocosm::schema::Id;
use isocosm::simulation::Simulation;
use isometer_core::{Role, classify};

/// How many part rows the tile shows before it starts counting instead.
pub const MAX_PART_ROWS: usize = 3;
/// How many learned kinds the tile names before it counts.
pub const MAX_DISCOVERY_NAMES: usize = 2;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Follow {
    pub id: String,
    pub species: String,
    pub at: String,
    pub reserve: String,
    pub substance: String,
    pub flows: String,
    pub window: String,
    pub revision: String,
    pub discovered: String,
    pub parts: String,
    pub part_rows: Vec<(String, String)>,
    pub more_parts: usize,
}

/// The tile for `followed`, or `None` when it is gone.
pub fn follow_of(sim: &Simulation, followed: Id) -> Option<Follow> {
    let e = sim.state().population.get(followed).filter(|e| e.alive)?;
    let ledger: u64 = e.accounts.values().sum();
    let tissue: u64 = e.parts.values().flat_map(|p| p.matter.values()).sum();
    let living: Vec<_> = e.living().collect();
    let mut part_rows = Vec::new();
    for (id, part) in living.iter().take(MAX_PART_ROWS) {
        let role = e
            .body
            .as_ref()
            .and_then(|b| b.part(*id))
            .map(|p| role_word(classify(p.half_extent)))
            .unwrap_or("no geometry");
        let functions: Vec<&str> = part
            .functions
            .iter()
            .map(|f| f.trim_start_matches("function:"))
            .collect();
        part_rows.push((format!("part {}", id.0), format!("{role}: {}", functions.join(", "))));
    }
    let known: Vec<&str> = e.traits.iter().map(String::as_str).collect();
    Some(Follow {
        id: followed.to_string(),
        species: e.lineage.clone(),
        at: format!("site {}", e.place),
        reserve: format!("{ledger} mg"),
        substance: format!("{tissue} mg"),
        flows: String::new(),
        window: String::new(),
        revision: e.body_revision.to_string(),
        discovered: match known.len() {
            0 => "nothing yet".into(),
            n if n <= MAX_DISCOVERY_NAMES => known.join(", "),
            n => format!("{} and {} more", known[..MAX_DISCOVERY_NAMES].join(", "), n - MAX_DISCOVERY_NAMES),
        },
        parts: living.len().to_string(),
        part_rows,
        more_parts: living.len().saturating_sub(MAX_PART_ROWS),
    })
}

pub fn role_word(role: Role) -> &'static str {
    match role {
        Role::Mass => "mass",
        Role::Limb => "limb",
        Role::Plate => "plate",
        Role::Sensor => "sensor",
    }
}

/// A followed critter that stopped being one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Lost {
    pub id: Id,
    pub tick: u64,
}

pub fn lost_of(critter: Id, at: u64) -> Lost {
    Lost {
        id: critter,
        tick: at,
    }
}

pub fn lost_words(lost: Lost) -> String {
    format!("critter {} is gone (tick {})", lost.id, lost.tick)
}
