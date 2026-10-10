// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A subject's body as the game reads it from the sim (wing ruling 767):
//! hunger is how far its matter reserve is below full and fatigue how far
//! its `Energy` is, the wound the share of its cells lost, its name the one
//! it was given. The game keeps only what the sim does not: the seed its
//! profile is drawn from and when it was named and died.

use std::collections::BTreeMap;

use isocosm::schema::{Entity, Id};
use isometer_core::snapshot::{self, hash_bytes};
use serde::{Deserialize, Serialize};

use crate::founding::{ENERGY, RESERVE};
use crate::identity::{BodyRevisionId, SubjectId, Tick};

pub const MAX_NEED: u16 = 100;
pub const SAFE_FALL: i32 = 4;
pub const MOBILITY_WOUND: u16 = 50;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Name(String);

impl Name {
    pub fn new(value: impl Into<String>) -> Result<Self, BodyError> {
        let name = Self(value.into());
        name.validate()?;
        Ok(name)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub(crate) fn validate(&self) -> Result<(), BodyError> {
        let trimmed = self.0.trim();
        if trimmed.is_empty() {
            return Err(BodyError::EmptyName);
        }
        if trimmed.chars().count() > 64 {
            return Err(BodyError::NameTooLong);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Needs {
    pub hunger: u16,
    pub fatigue: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyProfile {
    pub mass_mg: u32,
    pub carry_capacity_mg: u32,
    pub sight_range: i32,
    pub recovery_per_rest: u16,
}

impl BodyProfile {
    /// The profile a body draws from its world, seed and subject.
    pub fn draw(world_seed: u64, body_seed: u64, subject: SubjectId) -> Self {
        let bytes = snapshot::encode(&(world_seed, body_seed, subject)).expect("fixed-width genesis encodes");
        let draw = hash_bytes(&bytes);
        Self {
            mass_mg: 48_000 + (draw % 8_000) as u32,
            carry_capacity_mg: 1_600 + ((draw >> 13) % 400) as u32,
            sight_range: 16 + ((draw >> 27) % 9) as i32,
            recovery_per_rest: 40 + ((draw >> 39) % 11) as u16,
        }
    }
}

/// What the game keeps of a body beside the sim.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyRecord {
    pub entity: Id,
    /// Moves when a severing changes the body (717), as its admitted
    /// anatomy is reconciled.
    pub revision: BodyRevisionId,
    pub body_seed: u64,
    pub named_at: Option<Tick>,
    pub died_at: Option<Tick>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Body {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub profile: BodyProfile,
    pub name: Option<Name>,
    pub born_at: Option<Tick>,
    pub died_at: Option<Tick>,
    pub vitality: u16,
    pub wound: u16,
    pub needs: Needs,
}

impl Body {
    /// `subject`'s body as the sim holds it.
    pub fn read(world: &crate::World, subject: SubjectId, record: &BodyRecord) -> Option<Self> {
        let sim = world.sim();
        let e = sim.state().population.get(record.entity)?;
        let rules = &sim.genesis().rules;
        let need = |account| {
            let want = isocosm::needs::want(rules, e, account);
            want.map_or(0, |w| w.deficit().min(u64::from(MAX_NEED)) as u16)
        };
        let wound = wound(e);
        Some(Self {
            subject,
            revision: record.revision,
            profile: BodyProfile::draw(world.seed(), record.body_seed, subject),
            name: sim.name_of(record.entity).map(|n| Name(n.into())),
            born_at: record.named_at,
            died_at: record.died_at,
            vitality: if e.alive { 100 - wound } else { 0 },
            wound,
            needs: Needs { hunger: need(RESERVE), fatigue: need(ENERGY) },
        })
    }

    pub fn alive(&self) -> bool {
        self.name.is_some() && self.died_at.is_none() && self.vitality > 0
    }
    pub fn named(&self) -> bool {
        self.name.is_some()
    }
    pub fn mobile(&self) -> bool {
        self.alive() && self.wound < MOBILITY_WOUND && self.needs.fatigue < MAX_NEED
    }
    pub fn can_carry(&self, carried_mg: u32, added_mg: u32) -> bool {
        carried_mg.saturating_add(added_mg) <= self.profile.carry_capacity_mg
    }
}

/// The share of a body's cells it has lost, out of a hundred: lost cells
/// and every severed part's whole lattice.
fn wound(e: &Entity) -> u16 {
    let (mut lost, mut all) = (0u64, 0u64);
    for (id, p) in &e.parts {
        let cells = u64::from(isocosm::anatomy::capacity(e.extent(*id)));
        all += cells;
        lost += if e.lives(*id) { p.lost.len() as u64 } else { cells };
    }
    (lost * 100).checked_div(all).unwrap_or(0).min(100) as u16
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bodies {
    bodies: BTreeMap<SubjectId, Body>,
}

impl Bodies {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, subject: SubjectId) -> Option<&Body> {
        self.bodies.get(&subject)
    }
    pub fn all(&self) -> impl Iterator<Item = &Body> {
        self.bodies.values()
    }
    /// Every recorded body, read from the sim.
    pub(crate) fn read(world: &crate::World, records: &BTreeMap<SubjectId, BodyRecord>) -> Self {
        let bodies = records
            .iter()
            .filter_map(|(s, r)| Some((*s, Body::read(world, *s, r)?)))
            .collect();
        Self { bodies }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyError {
    SubjectExists(SubjectId),
    MissingSubject(SubjectId),
    EmptyName,
    NameTooLong,
    AlreadyNamed(SubjectId),
    Unnamed(SubjectId),
    Dead(SubjectId),
    Immobile(SubjectId),
}
