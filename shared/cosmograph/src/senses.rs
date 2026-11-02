// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What an active entity senses (case 1071, doctrine Bodies 15): its own
//! state first, then its surroundings, by its form's means. An animal touches
//! what it contacts and smells within its lineage's radius (the spatial
//! foraging reading); a plant reads light, soil and crowding where its leaves
//! and roots reach, which at site grain is its site; a mold or microbe reads
//! its lineage's matter here against neighbouring places; the world entity
//! reads its site's fields and recent activity. Every reading is computed
//! from the world as the pass began and stores nothing.

use crate::{
    Simulation,
    rules::Effect,
    schema::{EventPlace, Id, Key},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The smell bounds a new founding draws within, in base cells.
pub const SMELL: [u64; 2] = [4, 16];

/// A world that senses, and each animal lineage's drawn smell radius.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Senses {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub smell: BTreeMap<Key, u64>,
}

/// The readings a process may require (`Query::Sensed`), by the form that
/// reads them. Interoception needs no key: a body's own accounts are read
/// by the ordinary account queries.
pub const READINGS: [&str; 10] = [
    // An animal's smell radius.
    "sense:smell",
    // A plant's site light and soil, its crowding by plants, and its shares.
    "sense:light",
    "sense:soil",
    "sense:crowding",
    "sense:light-share",
    "sense:soil-share",
    // A territory's matter here, and the neighbours that hold less.
    "sense:here",
    "sense:downhill",
    // The world entity's site activity at the last tick.
    "sense:births",
    "sense:deaths",
];

/// How a form senses: the kingdom names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    Animal,
    Plant,
    Territory,
    World,
}

pub fn form(kingdom: &str) -> Option<Form> {
    match kingdom {
        "kingdom:fauna" => Some(Form::Animal),
        "kingdom:flora" => Some(Form::Plant),
        "kingdom:myco" | "kingdom:micro" => Some(Form::Territory),
        "kingdom:world" => Some(Form::World),
        _ => None,
    }
}

const SUN: &str = "sim:energy";
const SOIL: &str = "world:soil";

impl Simulation {
    /// The actor's smell radius, nought where it does not smell.
    pub fn smell_radius(&self, actor: Id) -> u64 {
        let Some(senses) = &self.genesis.rules.senses else { return 0 };
        self.body_at_start(actor)
            .and_then(|b| senses.smell.get(&b.lineage).copied())
            .unwrap_or(0)
    }

    /// What `actor` reads of `reading` as the pass began, or nothing where
    /// the world does not sense, the reading is not its form's, or it has no
    /// site to read.
    pub fn sensed(&self, actor: Id, reading: &str) -> Option<u64> {
        self.genesis.rules.senses.as_ref()?;
        let body = self.body_at_start(actor)?;
        if !body.alive {
            return None;
        }
        let site = body.resident_site()?;
        let form = form(&body.kingdom)?;
        match (form, reading) {
            (Form::Animal, "sense:smell") => Some(self.smell_radius(actor)),
            (Form::Plant, _) => {
                let place = self.site_at_start(site)?;
                let light = place.accounts.get(SUN).copied().unwrap_or(0);
                let soil = place.accounts.get(SOIL).copied().unwrap_or(0);
                let plants: BTreeSet<Key> = self.genesis.lineages.iter()
                    .filter(|(_, l)| l.kingdom == "kingdom:flora").map(|(k, _)| k.clone()).collect();
                let crowding = self.members_at(site, &plants).max(1);
                match reading {
                    "sense:light" => Some(light),
                    "sense:soil" => Some(soil),
                    "sense:crowding" => Some(crowding),
                    "sense:light-share" => Some(light / crowding),
                    "sense:soil-share" => Some(soil / crowding),
                    _ => None,
                }
            }
            (Form::Territory, "sense:here" | "sense:downhill") => {
                let here = self.matter_at(site, &body.lineage);
                if reading == "sense:here" {
                    return Some(u64::try_from(here).unwrap_or(u64::MAX));
                }
                let routes = &self.site_at_start(site)?.routes;
                let below: BTreeSet<Id> = routes.iter().map(|r| r.to)
                    .filter(|to| self.matter_at(*to, &body.lineage) < here).collect();
                Some(below.len() as u64)
            }
            (Form::World, "sense:births" | "sense:deaths") => {
                let last = self.state.tick.checked_sub(1)?;
                let effect = |e: &Effect| match reading {
                    "sense:births" => matches!(e, Effect::Birth { .. }),
                    _ => matches!(e, Effect::Death),
                };
                let n = self.state.events.values()
                    .filter(|ev| ev.tick == last && ev.place == EventPlace::Site(site))
                    .filter(|ev| self.genesis.rules.processes.get(&ev.process)
                        .is_some_and(|p| p.commitments.iter().chain(&p.effects).any(effect)))
                    .count();
                Some(n as u64)
            }
            _ => None,
        }
    }
}
