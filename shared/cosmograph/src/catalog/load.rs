// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Reads the embedded datasheets and assembles the catalog. Same format as
//! the other datasheets in this crate (TOML, `schema = 1`, embedded with
//! `include_str!`; `datasheet/mod.rs`, `social/ladder/tiers.rs`): pack
//! files people write are TOML (data formats brief F1, ruling 624).

use std::collections::BTreeSet;

use serde::Deserialize;

use super::refs::is_proposed;
use super::types::{EffectRow, PartRow, TraitRow};
use super::{Catalog, Effect, Mechanic, Origin, Part, ProposedProcess, Status, Trait, generate};

macro_rules! sheet {
    ($name:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/datasheets/catalog/", $name, ".toml"))
    };
}

const SHEETS: [(&str, &str); 11] = [
    ("mechanics", sheet!("mechanics")),
    ("traits_structure", sheet!("traits_structure")),
    ("traits_chemistry", sheet!("traits_chemistry")),
    ("traits_living", sheet!("traits_living")),
    ("traits_life", sheet!("traits_life")),
    ("effects_a", sheet!("effects_a")),
    ("effects_b", sheet!("effects_b")),
    ("parts_flora", sheet!("parts_flora")),
    ("parts_fauna", sheet!("parts_fauna")),
    ("parts_myco", sheet!("parts_myco")),
    ("parts_micro", sheet!("parts_micro")),
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Sheet {
    schema: u32,
    // Header fields every datasheet carries; read by people, not the code.
    #[allow(dead_code)]
    owner: String,
    #[allow(dead_code)]
    consumer: String,
    #[allow(dead_code)]
    status: String,
    #[serde(default, rename = "trait")]
    traits: Vec<TraitRow>,
    #[serde(default, rename = "effect")]
    effects: Vec<EffectRow>,
    #[serde(default, rename = "part")]
    parts: Vec<PartRow>,
    #[serde(default, rename = "mechanic")]
    mechanics: Vec<Mechanic>,
    #[serde(default, rename = "proposed_process")]
    proposed: Vec<ProposedProcess>,
}

fn trait_status(process: &str, via: &[String], needs: &Option<String>) -> Status {
    match needs {
        _ if is_proposed(process) => Status::Proposed { process: process.into() },
        Some(mechanic) => Status::NeedsRule { mechanic: mechanic.clone() },
        None => Status::Expressible { via: via.to_vec() },
    }
}

fn one_of(what: &str, key: &str, via: &[String], needs: &Option<String>) -> Result<(), String> {
    match (via.is_empty(), needs.is_none()) {
        (false, false) => Err(format!("{what} {key} names both via and needs")),
        (true, true) => Err(format!("{what} {key} names neither via nor needs")),
        _ => Ok(()),
    }
}

/// A part's status, and every mechanic it waits on, from what it carries.
/// Effects first in site order, then traits, so the status names the first
/// thing in the order a build would meet it.
pub(super) fn derive(part: &Part, traits: &[Trait], effects: &[Effect]) -> (Status, Vec<String>) {
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    let mut vias = BTreeSet::new();
    let mut note = |needs: &Option<String>, via: &[String]| {
        match needs {
            Some(m) if seen.insert(m.clone()) => ordered.push(m.clone()),
            Some(_) => {}
            None => vias.extend(via.iter().cloned()),
        }
    };
    for key in part.effect_keys() {
        if let Some(e) = effects.iter().find(|e| &e.key == key) {
            note(&e.needs, &e.via);
        }
    }
    for key in &part.traits {
        if let Some(t) = traits.iter().find(|t| &t.key == key) {
            note(&t.needs, &t.via);
        }
    }
    let mut waits = ordered.clone();
    waits.sort();
    let status = if is_proposed(&part.process) {
        Status::Proposed { process: part.process.clone() }
    } else if let Some(first) = ordered.first() {
        Status::NeedsRule { mechanic: first.clone() }
    } else {
        let name = super::refs::process_name(&part.process);
        vias.insert(format!("function:{name}"));
        Status::Expressible { via: vias.into_iter().collect() }
    };
    (status, waits)
}

pub(super) fn build() -> Result<Catalog, String> {
    let (mut trait_rows, mut effect_rows, mut part_rows) = (vec![], vec![], vec![]);
    let (mut mechanics, mut proposed) = (vec![], vec![]);
    for (name, text) in SHEETS {
        let sheet: Sheet = toml::from_str(text).map_err(|e| format!("catalog sheet {name}: {e}"))?;
        if sheet.schema != 1 {
            return Err(format!("catalog sheet {name}: unknown schema {}", sheet.schema));
        }
        trait_rows.extend(sheet.traits);
        effect_rows.extend(sheet.effects);
        part_rows.extend(sheet.parts);
        mechanics.extend(sheet.mechanics);
        proposed.extend(sheet.proposed);
    }
    let mut traits = Vec::new();
    for r in trait_rows {
        one_of("trait", &r.key, &r.via, &r.needs)?;
        let status = trait_status(&r.process, &r.via, &r.needs);
        traits.push(Trait {
            key: r.key,
            summary: r.summary,
            category: r.category,
            process: r.process,
            shape: r.shape,
            kingdoms: r.kingdoms,
            models: r.models,
            sites: r.sites,
            admits: r.admits,
            emits: r.emits,
            cost_mg: r.cost_mg,
            reads: r.reads,
            analogue: r.analogue,
            via: r.via,
            needs: r.needs,
            unlocks: r.unlocks,
            strengths: r.strengths,
            status,
        });
    }
    let mut effects = Vec::new();
    for r in effect_rows {
        one_of("effect", &r.key, &r.via, &r.needs)?;
        let status = match &r.needs {
            Some(mechanic) => Status::NeedsRule { mechanic: mechanic.clone() },
            None => Status::Expressible { via: r.via.clone() },
        };
        effects.push(Effect {
            key: r.key,
            summary: r.summary,
            site: r.site,
            class: r.class,
            models: r.models,
            accepts: r.accepts,
            processes: r.processes,
            only: r.only,
            kingdoms: r.kingdoms,
            admits: r.admits,
            emits: r.emits,
            cost_mg: r.cost_mg,
            analogue: r.analogue,
            via: r.via,
            needs: r.needs,
            gates: r.gates,
            timing: r.timing,
            status,
        });
    }
    let mut parts: Vec<Part> = part_rows
        .into_iter()
        .map(|r| Part {
            key: r.key,
            kingdom: r.kingdom,
            process: r.process,
            shape: r.shape,
            traits: r.traits,
            effects: r.effects.into_iter().filter(|(_, keys)| !keys.is_empty()).collect(),
            admits: r.admits,
            emits: r.emits,
            cost_mg: r.cost_mg.unwrap_or(0),
            models: r.models,
            analogue: r.analogue,
            status: Status::Proposed { process: String::new() },
            waits_on: Vec::new(),
            origin: Origin::Authored,
        })
        .collect();
    let (generated, dropped) = generate(&traits, &effects, &parts);
    parts.extend(generated);
    for i in 0..parts.len() {
        if parts[i].cost_mg == 0 {
            let price = |key: &String| {
                traits.iter().find(|t| &t.key == key).map(|t| t.cost_mg)
                    .or_else(|| effects.iter().find(|e| &e.key == key).map(|e| e.cost_mg))
                    .unwrap_or(0)
            };
            parts[i].cost_mg = parts[i].traits.iter().chain(parts[i].effect_keys()).map(price).sum();
        }
        let (status, waits) = derive(&parts[i], &traits, &effects);
        parts[i].status = status;
        parts[i].waits_on = waits;
    }
    Ok(Catalog { traits, effects, parts, mechanics, proposed, dropped })
}
