// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Lossless native v1/v2 wire facts, before geometry moved out of Part.
//! Hashes always use these typed fields, never the input JSON's spelling/order.

use super::*;
use crate::{population, rules::Rules};
use serde::{de::DeserializeOwned, Deserializer};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyPart {
    pub parent: Option<Id>,
    pub traits: BTreeSet<Key>,
    pub severed: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub shape: Key,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub functions: BTreeSet<Key>,
    #[serde(default, skip_serializing_if = "zero")]
    pub half_extent: [i32; 3],
    #[serde(default, skip_serializing_if = "zero")]
    pub offset: [i32; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub situs: Option<[u8; 3]>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cells: BTreeMap<Key, u32>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub matter: Ledger,
}

fn zero(value: &[i32; 3]) -> bool { *value == [0; 3] }

impl LegacyPart {
    pub(super) fn virtual_part(&self) -> bool {
        self.parent.is_none() && !self.severed && self.shape.is_empty()
            && self.functions.is_empty() && zero(&self.half_extent) && zero(&self.offset)
            && self.situs.is_none() && self.cells.is_empty() && self.matter.is_empty()
    }
    pub(super) fn native(&self) -> Part {
        Part { traits: self.traits.clone(), ..Part::default() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyEntity {
    pub lineage: Key,
    pub kingdom: Key,
    pub scale: Key,
    pub provenance: Provenance,
    pub method: Method,
    pub place: Id,
    pub arrived: Tick,
    pub visits: Vec<Visit>,
    pub born: Tick,
    pub alive: bool,
    pub body_revision: u64,
    pub parts: BTreeMap<Id, LegacyPart>,
    pub traits: BTreeSet<Key>,
    pub accounts: Ledger,
    pub skills: BTreeMap<Key, u64>,
    pub tenets: BTreeMap<Key, Tenet>,
    pub disposition: [i16; 5],
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub soma: Vec<u8>,
}

impl LegacyEntity {
    pub(super) fn native(&self) -> Result<Entity> {
        let parts = self.parts.iter().map(|(&id, p)| {
            Ok((PartId(u32::try_from(id).map_err(|_| "legacy part identity exceeds u32")?), std::sync::Arc::new(p.native())))
        }).collect::<Result<_>>()?;
        Ok(Entity { lineage:self.lineage.clone(), kingdom:self.kingdom.clone(), scale:self.scale.clone(),
            provenance:self.provenance.clone(), method:self.method, place:self.place, arrived:self.arrived,
            visits:self.visits.clone(), journey:None, born:self.born, alive:self.alive,
            body_revision:self.body_revision, body:None, parts, traits:self.traits.clone(),
            accounts:self.accounts.clone(), skills:self.skills.clone(), tenets:self.tenets.clone(),
            disposition:self.disposition, soma:self.soma.clone(), systems:Default::default(),
            varied:vec![], patch:None, authored:None, settled:None })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyCohort {
    pub count: u64,
    pub entity: LegacyEntity,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyPopulation {
    pub groups: BTreeMap<Id, LegacyCohort>,
    pub next_id: Id,
}

/// Shared leaves keep their historical codec. Refuse discarded input members;
/// this check supplies no digest or authority and cannot substitute raw hashing.
pub(super) fn retained<'de, D, T>(d: D) -> std::result::Result<T, D::Error>
where D: Deserializer<'de>, T: DeserializeOwned + Serialize {
    let input = serde_json::Value::deserialize(d)?;
    let typed: T = serde_json::from_value(input.clone()).map_err(serde::de::Error::custom)?;
    let encoded = serde_json::to_value(&typed).map_err(serde::de::Error::custom)?;
    fn members(a: &serde_json::Value, b: &serde_json::Value) -> bool {
        match (a,b) {
            (serde_json::Value::Object(a), serde_json::Value::Object(b)) =>
                a.iter().all(|(key,value)| b.get(key).is_some_and(|held| members(value,held))),
            (serde_json::Value::Array(a), serde_json::Value::Array(b)) =>
                a.len() == b.len() && a.iter().zip(b).all(|(a,b)| members(a,b)),
            _ => true,
        }
    }
    if !members(&input,&encoded) {
        return Err(serde::de::Error::custom("unsupported discarded legacy schema member"));
    }
    Ok(typed)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyGenesis {
    pub version: u32,
    pub seed: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<u64>,
    #[serde(deserialize_with = "retained")]
    pub founding: Option<crate::Founding>,
    #[serde(deserialize_with = "retained")]
    pub world: WorldTraits,
    #[serde(deserialize_with = "retained")]
    pub rules: Rules,
    #[serde(deserialize_with = "retained")]
    pub lineages: BTreeMap<Key, Lineage>,
    #[serde(deserialize_with = "retained")]
    pub sites: BTreeMap<Id, Site>,
    pub population: LegacyPopulation,
}

impl LegacyGenesis {
    pub(super) fn native(&self) -> Result<Genesis> {
        let groups = self.population.groups.iter().map(|(&first,g)| {
            Ok((first,population::Cohort { count:g.count, entity:std::sync::Arc::new(g.entity.native()?) }))
        }).collect::<Result<_>>()?;
        Ok(Genesis { version:self.version, seed:self.seed, dynamics:self.dynamics,
            founding:self.founding.clone(), world:self.world.clone(), rules:self.rules.clone(),
            lineages:self.lineages.clone(), sites:self.sites.clone(),
            population:population::Population { groups, next_id:self.population.next_id },
            spreads:Default::default() })
    }
}
