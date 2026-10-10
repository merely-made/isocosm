// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The frozen developmental context. (PD4)
//!
//! Plan §4: *a frozen view containing only declared facts*. Every field here is
//! an owned copy of something the host decided to show — no borrowed world, no
//! handle, nothing a script could follow back to a live value. What is not in
//! this struct is not visible to an author, which is how "scripts cannot
//! inspect hidden world state" is enforced rather than asked for.

use isocosm::mosaic::{dims, path};
use isocosm::process::{Registry, RulesetDigest};
use isocosm::schema::Entity;
use isometer_core::{Role, classify};
use serde::{Deserialize, Serialize};

/// Why the host is asking. (Plan §4's bounded triggers.)
///
/// One today, because one is played. §4 lists founding and filial regrowth, a
/// chosen adaptation, assimilation or grafting, growth and repair, and
/// lifecycle change; each arrives with the gate that plays it, rather than as a
/// vocabulary written ahead of a consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trigger {
    /// A line came to a developmental option and is taking it up.
    Discovery,
}

impl Trigger {
    /// The word a script reads.
    pub fn word(self) -> &'static str {
        match self {
            Trigger::Discovery => "discovery",
        }
    }
}

/// One admitted definition, as an author sees it.
///
/// Identity, tract requirement and seeding — the same three things the digest
/// folds. A script is shown what a definition *rules*, never a native binding
/// or a label, because neither is rule-bearing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    /// `namespace:name`.
    pub id: String,
    /// The shape words a part must classify as: `mass`, `limb`, `plate`,
    /// `sensor`.
    pub expressed_by: Vec<String>,
    /// `geometry` or `acquired`.
    pub seeding: String,
}

/// One tract a part already expresses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TractView {
    /// `namespace:name`, or `unknown` when this world's ruleset no longer holds
    /// the definition the tract cites. Never the nearest local one.
    pub process: String,
    pub cells: u32,
}

/// One living part: a stable address, its shape, its tissue, and what it does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartView {
    /// The stable address, and what a proposal names.
    pub part: u32,
    pub role: String,
    pub cells: u32,
    pub free: u32,
    /// What one cell of this part's tissue is worth, in milligrams. The price
    /// a development is charged at, shown so an author can weigh it — and
    /// **not** so an author can set it: the host prices the accepted proposal
    /// itself (plan §4, "the proposal does not choose its own cost").
    pub cell_mg: u64,
    #[serde(alias = "sites")]
    pub tracts: Vec<TractView>,
}

/// One quantized world reading a script may branch on.
///
/// Named and integer, so a fixture can state the context it was recorded under
/// and a different one is a different declared context rather than a different
/// afternoon.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ambient {
    pub name: String,
    pub value: i64,
}

/// The whole frozen picture one expression call is given.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub trigger: Trigger,
    /// Which biology this is, as its content address. A fixture recorded under
    /// one ruleset is visibly a fixture for that ruleset.
    pub ruleset: RulesetDigest,
    /// The body revision this was frozen at.
    pub revision: u32,
    /// The phenotype digest a lowered proposal must expect. Staleness is
    /// refusable because this travels.
    pub expect: u64,
    /// Every definition this world admitted, sorted by id.
    pub definitions: Vec<Definition>,
    /// Every living part, in part order.
    pub parts: Vec<PartView>,
    /// What this request is about: the qualified ids the line has come to and
    /// may express here, sorted. A script that proposes anything else is
    /// making it up, and the validator will say so.
    ///
    /// Empty when this world's ruleset does not hold what the line came to,
    /// which is the missing-ruleset answer one door up (plan §6): the id is
    /// dropped rather than replaced with the nearest local definition.
    pub candidates: Vec<String>,
    /// The body's own reserve, in milligrams. An integer budget, per §4.
    pub material_mg: u64,
    /// Declared quantized world conditions, sorted by name.
    pub conditions: Vec<Ambient>,
}

impl Request {
    /// The context for `entity`'s body, frozen: its parts as the native
    /// mosaic lays them out (766), the registry's definitions, and what the
    /// host chose to show beside them.
    pub fn frozen(
        registry: &Registry,
        entity: &Entity,
        mut candidates: Vec<String>,
        material_mg: u64,
        mut conditions: Vec<Ambient>,
    ) -> Self {
        // Sorted here rather than trusted from the caller, so two fixtures that
        // declare the same context are the same context.
        candidates.sort();
        conditions.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            trigger: Trigger::Discovery,
            ruleset: registry.digest(),
            revision: u32::try_from(entity.body_revision).unwrap_or(u32::MAX),
            expect: isocosm::draw(entity.body_revision, "express", &[]),
            definitions: definitions_of(registry),
            parts: parts_of(entity),
            candidates,
            material_mg,
            conditions,
        }
    }
}

fn definitions_of(registry: &Registry) -> Vec<Definition> {
    registry
        .all()
        .map(|def| Definition {
            id: def.id.qualified(),
            expressed_by: def
                .expressed_by
                .iter()
                .map(|role| role_word(*role))
                .collect(),
            seeding: if def.seeded() { "geometry" } else { "acquired" }.to_owned(),
        })
        .collect()
}

/// A function key as the registry names its definition (773).
pub(crate) fn qualified(function: &str) -> String {
    format!("mesocosm:{}", function.trim_start_matches("function:"))
}

fn parts_of(entity: &Entity) -> Vec<PartView> {
    let Some(body) = entity.body.as_ref() else {
        return Vec::new();
    };
    entity
        .living()
        .filter_map(|(id, part)| {
            let half = body.part(id)?.half_extent;
            let total = path(dims(half)).len() as u32;
            let held: u32 = part.tracts.iter().map(|t| t.cells.len() as u32).sum();
            let lost = part.lost.len() as u32;
            let tissue: u64 = part.matter.values().sum();
            Some(PartView {
                part: id.0,
                role: role_word(classify(half)),
                cells: total.saturating_sub(lost),
                free: total.saturating_sub(lost).saturating_sub(held),
                cell_mg: tissue / u64::from(total.max(1)),
                tracts: part
                    .tracts
                    .iter()
                    .map(|t| TractView {
                        process: qualified(&t.function),
                        cells: t.cells.len() as u32,
                    })
                    .collect(),
            })
        })
        .collect()
}

pub(crate) fn role_word(role: Role) -> String {
    match role {
        Role::Mass => "mass",
        Role::Limb => "limb",
        Role::Plate => "plate",
        Role::Sensor => "sensor",
    }
    .to_owned()
}
