// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a script says, and how it becomes something the validator can read.
//! (PD4)
//!
//! # The smallest proposal plan §4 describes
//!
//! *Which admitted process should express on which existing part, at what
//! bounded capacity.* Three fields, and no fourth: not a cost (the host prices
//! the accepted result), not a cell address (the host lays tissue out), not a
//! revision (the host froze one into the request). Everything a script could
//! get wrong that the game would then have to live with is simply not
//! expressible.
//!
//! # Lowering is deterministic, and the script does not choose it
//!
//! A part's requested tracts take tissue from the high end of its lattice
//! downward, in the order the script listed them, each run contiguous — the
//! same suffix rule
//! legacy `Candidate::propose` relied on, and for
//! the same reason: a suffix of the row-major order is a connected region and
//! so is the prefix left behind. What the script did not claim keeps doing what
//! it did. The result is a **complete desired state** for the parts named,
//! which is the only shape the validator accepts.

use isocosm::mosaic::{CellId, dims, path, propose};
use isocosm::process::{ProcessId, Registry};
use isocosm::schema::{Entity, Key, PartId};
use serde::{Deserialize, Serialize};

use super::Refused;

/// One thing a script asks a part to express.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expression {
    /// The stable part address, from the request.
    pub part: u32,
    /// `namespace:name`. Resolved against the world's admitted ruleset, and
    /// refused when it does not hold it.
    pub process: String,
    /// How much tissue, in cells. Bounded capacity, per plan §4.
    pub cells: u32,
}

/// What one expression call returned.
///
/// **A proposal, never a change.** Nothing here has happened; it is what an
/// author would like to have happen, on its way to the one validator.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    #[serde(alias = "sites")]
    pub tracts: Vec<Expression>,
}

/// A lowered proposal: each named part's complete desired tracts, by
/// function key, ready for the native mosaic (766).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Allocation {
    pub parts: Vec<(PartId, Vec<(Key, Vec<CellId>)>)>,
}

impl Allocation {
    /// Places the cells on `entity`'s parts, all or nothing; returns how many
    /// cells changed what they do.
    pub fn commit(&self, entity: &mut Entity) -> Result<u32, Refused> {
        let body = entity.body.clone().ok_or(Refused::Validator("no body".into()))?;
        let mut parts = entity.parts.clone();
        let mut changed = 0;
        for (id, tracts) in &self.parts {
            let half = body.part(*id).ok_or(Refused::UnknownPart { part: *id })?.half_extent;
            let part = parts.get_mut(id).ok_or(Refused::UnknownPart { part: *id })?;
            changed += propose(part, half, tracts).map_err(Refused::Validator)?;
        }
        entity.parts = parts;
        Ok(changed)
    }
}

/// Lowers a script's proposal onto `entity`'s living parts: each requested
/// tract takes cells from the high end of the part's lattice path, in the
/// script's order; what the script did not claim keeps doing what it did.
pub fn lower(
    registry: &Registry,
    entity: &Entity,
    proposal: &Proposal,
) -> Result<Allocation, Refused> {
    let body = entity.body.as_ref().ok_or(Refused::Validator("no body".into()))?;
    let mut named: Vec<PartId> = proposal.tracts.iter().map(|t| PartId(t.part)).collect();
    named.sort_unstable();
    named.dedup();
    let mut parts = Vec::new();
    for id in named {
        let (Some(part), Some(geometry)) = (entity.parts.get(&id), body.part(id)) else {
            return Err(Refused::UnknownPart { part: id });
        };
        let lost = &part.lost;
        let living: Vec<CellId> = path(dims(geometry.half_extent))
            .into_iter()
            .filter(|c| !lost.contains(c))
            .collect();
        let mine = proposal.tracts.iter().filter(|t| t.part == id.0);
        let asked = mine.clone().map(|t| t.cells).fold(0u32, u32::saturating_add);
        if asked as usize > living.len() {
            return Err(Refused::TooMuchTissue {
                part: id,
                asked,
                living: living.len() as u32,
            });
        }
        let mut taken: Vec<CellId> = Vec::new();
        let mut requested: Vec<(Key, Vec<CellId>)> = Vec::new();
        for tract in mine {
            let function = resolve(registry, &tract.process)?;
            let remaining = &living[..living.len() - taken.len()];
            let run = remaining[remaining.len() - tract.cells as usize..].to_vec();
            taken.extend(run.iter().copied());
            match requested.iter_mut().find(|(held, _)| *held == function) {
                Some((_, cells)) => cells.extend(run),
                None => requested.push((function, run)),
            }
        }
        // What the part already does keeps its place, the script's after.
        let mut claimed: Vec<(Key, Vec<CellId>)> = part
            .tracts
            .iter()
            .filter_map(|t| {
                let kept: Vec<CellId> =
                    t.cells.iter().copied().filter(|c| !taken.contains(c)).collect();
                (!kept.is_empty()).then(|| (t.function.clone(), kept))
            })
            .collect();
        for (function, run) in requested {
            match claimed.iter_mut().find(|(held, _)| *held == function) {
                Some((_, cells)) => cells.extend(run),
                None => claimed.push((function, run)),
            }
        }
        for (_, cells) in &mut claimed {
            cells.sort_unstable();
            cells.dedup();
        }
        parts.push((id, claimed));
    }
    Ok(Allocation { parts })
}

/// A qualified id, resolved against the world's own ruleset.
///
/// `None` is a real answer (plan §6, missing packs): an id this world did not
/// admit is refused by name and never replaced with the nearest local
/// definition.
fn resolve(registry: &Registry, qualified: &str) -> Result<Key, Refused> {
    let unknown = || Refused::UnknownProcess {
        id: qualified.to_owned(),
    };
    let (namespace, name) = qualified.split_once(':').ok_or_else(unknown)?;
    registry
        .get(&ProcessId::new(namespace, name))
        .map(|def| format!("function:{}", def.id.name))
        .ok_or_else(unknown)
}
