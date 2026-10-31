// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native witness preimages, separate from the current table wire (809).
//! Absent Character metadata preserves the former five-field preimage;
//! present metadata remains witnessed. No hash algorithm or version changes.

use crate::{asserted, population, schema::*};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
pub(crate) mod event;
mod state;
pub(super) use state::State;

mod groups;
pub(super) use groups::{Applied, Records};

#[derive(Serialize)]
pub(super) struct Character<'a> {
    key: &'a Key,
    name: &'a String,
    tags: &'a BTreeSet<Key>,
    faction: &'a Option<Key>,
    place: &'a Option<Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    table: Option<&'a asserted::CharacterTable>,
}

impl<'a> From<&'a asserted::Character> for Character<'a> {
    fn from(value: &'a asserted::Character) -> Self {
        let asserted::Character {
            key,
            name,
            tags,
            faction,
            place,
            table,
        } = value;
        Self {
            key,
            name,
            tags,
            faction,
            place,
            table: table.as_ref(),
        }
    }
}

/// Same order and omissions as Entity; only authored Character is projected.
/// Exhaustive destructuring makes added native fields require a review here.
#[derive(Serialize)]
pub(super) struct Entity<'a> {
    lineage: &'a Key,
    kingdom: &'a Key,
    scale: &'a Key,
    provenance: &'a Provenance,
    method: Method,
    place: Id,
    arrived: Tick,
    visits: &'a Vec<Visit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    journey: Option<&'a Journey>,
    born: Tick,
    alive: bool,
    body_revision: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<&'a BodyDocument>,
    parts: &'a BTreeMap<PartId, std::sync::Arc<Part>>,
    traits: &'a BTreeSet<Key>,
    accounts: &'a Ledger,
    skills: &'a BTreeMap<Key, u64>,
    tenets: &'a BTreeMap<Key, Tenet>,
    disposition: &'a [i16; 5],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    soma: &'a Vec<u8>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    systems: &'a BTreeMap<Key, crate::rules::System>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    varied: &'a Vec<Varied>,
    #[serde(skip_serializing_if = "Option::is_none")]
    patch: Option<&'a isometer_space::places::PlaceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    authored: Option<Character<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    settled: Option<Tick>,
}

impl<'a> From<&'a crate::schema::Entity> for Entity<'a> {
    fn from(value: &'a crate::schema::Entity) -> Self {
        let crate::schema::Entity {
            lineage,
            kingdom,
            scale,
            provenance,
            method,
            place,
            arrived,
            visits,
            journey,
            born,
            alive,
            body_revision,
            body,
            parts,
            traits,
            accounts,
            skills,
            tenets,
            disposition,
            soma,
            systems,
            varied,
            patch,
            authored,
            settled,
        } = value;
        Self {
            lineage,
            kingdom,
            scale,
            provenance,
            method: *method,
            place: *place,
            arrived: *arrived,
            visits,
            journey: journey.as_ref(),
            born: *born,
            alive: *alive,
            body_revision: *body_revision,
            body: body.as_ref(),
            parts,
            traits,
            accounts,
            skills,
            tenets,
            disposition,
            soma,
            systems,
            varied,
            patch: patch.as_ref(),
            authored: authored.as_ref().map(Character::from),
            settled: *settled,
        }
    }
}

#[derive(Serialize)]
struct Cohort<'a> {
    count: u64,
    entity: Entity<'a>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pool: &'a population::Pool,
}

impl<'a> From<&'a population::Cohort> for Cohort<'a> {
    fn from(value: &'a population::Cohort) -> Self {
        let population::Cohort { count, entity, pool } = value;
        Self {
            count: *count,
            entity: entity.as_ref().into(),
            pool,
        }
    }
}

#[derive(Serialize)]
pub(super) struct Population<'a> {
    groups: BTreeMap<Id, Cohort<'a>>,
    next_id: Id,
}

impl<'a> From<&'a population::Population> for Population<'a> {
    fn from(value: &'a population::Population) -> Self {
        let population::Population { groups, next_id } = value;
        Self {
            groups: groups
                .iter()
                .map(|(id, group)| (*id, group.into()))
                .collect(),
            next_id: *next_id,
        }
    }
}

#[cfg(test)]
mod tests;
