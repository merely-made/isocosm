// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The selected-recipe door for an asserted character (834). Identity and
//! authored fill survive native Arrival and Develop; no control is inferred.

use isocosm::{Command, Session, arrival::Arrival, schema::{Id, Key, Ledger, Method}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// All arrival inputs are authored. The selected lineage must already have
/// a native recipe; stocks are admitted by Arrival, never priced by the VTT.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterAdmission {
    pub character: Key,
    pub lineage: Key,
    pub seed: u64,
    pub site: Id,
    pub accounts: Ledger,
    pub method: Method,
    pub traits: BTreeSet<Key>,
    pub skills: BTreeMap<Key, u64>,
    pub disposition: [i16; 5],
}

/// Product binding and the historical recipe choice, not another saved body.
/// Native entry references retain the authoritative stocks/site and commands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecipeBinding {
    pub entity: Id,
    pub lineage: Key,
    pub recipe_revision: u64,
    pub recipe_digest: Key,
    pub seed: u64,
    pub arrival_entry: Key,
    pub develop_entry: Key,
}

impl RecipeBinding {
    pub fn validate(&self, session: &Session, character: &str) -> Result<(), String> {
        let held = session.sim.state().population.get(self.entity).ok_or("bound character is absent")?;
        if held.authored.as_ref().map(|fill| fill.key.as_str()) != Some(character) {
            return Err("VTT binding does not name this native authored character".into());
        }
        let arrival = session.entries.iter().find(|entry| entry.id == self.arrival_entry)
            .ok_or("VTT binding's native arrival entry is absent")?;
        match &arrival.command {
            Command::Arrive(value) if value.character.as_deref() == Some(character)
                && value.lineage == self.lineage && arrival.outcome == format!("entity:{}", self.entity) => {},
            _ => return Err("VTT binding's arrival differs from accepted native history".into()),
        }
        let develop = session.entries.iter().find(|entry| entry.id == self.develop_entry)
            .ok_or("VTT binding's native development entry is absent")?;
        if !matches!(&develop.command, Command::Develop { entity, seed }
            if *entity == self.entity && *seed == self.seed)
            || arrival.order >= develop.order || arrival.tick != develop.tick
        {
            return Err("VTT binding's development differs from accepted native history".into());
        }
        Ok(())
    }
}

/// The borrowed Session publishes only after both native operations accept.
/// Table owner/cell, patch placement, Join and Take are separate operations.
pub fn admit_character(session: &mut Session, input: &CharacterAdmission) -> Result<RecipeBinding, String> {
    let (entity, held) = session.sim.authored_entity(&input.character)
        .ok_or("character must be asserted before recipe admission")?;
    if !held.alive || held.place != isocosm::directing::PLACELESS || held.journey.is_some()
        || held.body.is_some() || !held.parts.is_empty() || held.accounts.values().any(|amount| *amount != 0)
    {
        return Err("recipe admission requires a living placeless character without a body or previous stocks".into());
    }
    let fill = held.authored.clone();
    let relations: BTreeSet<_> = session.sim.state().relations.iter()
        .filter(|relation| relation.subject == entity || relation.object == entity).cloned().collect();
    let lineage = session.sim.state().lineages.get(&input.lineage).ok_or("selected native lineage is absent")?;
    let recipe = lineage.development.as_ref().ok_or("selected native lineage has no recipe")?;
    let recipe_revision = lineage.revision;
    let recipe_digest = isocosm::digest(recipe);
    let arrival = Arrival { character:Some(input.character.clone()), lineage:input.lineage.clone(),
        site:input.site, accounts:input.accounts.clone(), method:input.method,
        traits:input.traits.clone(), skills:input.skills.clone(), disposition:input.disposition };
    let mut candidate = session.clone();
    let result = candidate.command(Command::Arrive(arrival))?;
    if result != format!("entity:{entity}") { return Err("native arrival changed the asserted character's ID".into()); }
    let arrival_entry = candidate.entries.last().ok_or("native arrival did not record its command")?.id.clone();
    candidate.command(Command::Develop { entity, seed:input.seed })?;
    let develop_entry = candidate.entries.last().ok_or("native development did not record its command")?.id.clone();
    let developed = candidate.sim.state().population.get(entity).ok_or("developed character is absent")?;
    let after_relations: BTreeSet<_> = candidate.sim.state().relations.iter()
        .filter(|relation| relation.subject == entity || relation.object == entity).cloned().collect();
    if developed.authored != fill || developed.body.is_none() || after_relations != relations {
        return Err("native character development did not retain its authored fill and faction relations".into());
    }
    let binding = RecipeBinding { entity, lineage:input.lineage.clone(), recipe_revision, recipe_digest,
        seed:input.seed, arrival_entry, develop_entry };
    binding.validate(&candidate, &input.character)?;
    *session = candidate;
    Ok(binding)
}
