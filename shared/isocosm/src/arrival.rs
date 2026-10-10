// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A life arriving from outside the world, its body admitted, and its name
//! (Eponym's world move, wing rulings 235, 238 and 755). An outsider enters
//! at a site with what it carries, which is issued into the run as the dev
//! source's matter is, from beyond the conserved total. Its geometry is
//! isometer's document and its physiology the sim's, keyed by part (674).
//! A name is a note its namer holds about it (36, 168, 200): naming is a
//! sapient's own act, and a denizen is one somebody named. An authored
//! character, asserted placeless (769), arrives by its key: the asserted
//! entity takes up the body, keeping its fill and its faction (760).

use crate::{Result, meaning::credit, rules::AccountKind, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The note kind a name is kept under.
pub const NAME: &str = "sim:name";

/// An outsider: its lineage, the site it enters at, how it chooses, and
/// what it brings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arrival {
    pub lineage: Key,
    pub site: Id,
    pub method: Method,
    #[serde(default)]
    pub accounts: Ledger,
    #[serde(default)]
    pub traits: BTreeSet<Key>,
    #[serde(default)]
    pub skills: BTreeMap<Key, u64>,
    #[serde(default)]
    pub disposition: [i16; 5],
    /// The authored character arriving, by its key, where one does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<Key>,
}

impl Simulation {
    /// An outsider enters the world; returns its id.
    pub(crate) fn arrive(&mut self, a: &Arrival) -> Result<Id> {
        let lineage = self.state.lineages.get(&a.lineage).ok_or("an arrival of no lineage")?;
        if !self.state.sites.contains_key(&a.site) {
            return Err("an arrival at no site".into());
        }
        if self.state.population.count() >= self.genesis.rules.limits.entities {
            return Err("entity limit".into());
        }
        let mut issued = 0u128;
        for (key, amount) in &a.accounts {
            if let Some(AccountKind::Matter { .. }) = self.genesis.rules.accounts.get(key) {
                issued += u128::from(*amount);
            } else if !self.genesis.rules.accounts.contains_key(key) {
                return Err(format!("{key} is no account"));
            }
        }
        let mut accounts = Ledger::new();
        for (key, amount) in &a.accounts {
            credit(&mut accounts, key, *amount)?;
        }
        let tick = self.state.tick;
        let entity = Entity {
            lineage: a.lineage.clone(),
            kingdom: lineage.kingdom.clone(),
            scale: "scale:meso".into(),
            provenance: Provenance::Born(a.lineage.clone()),
            method: a.method,
            place: a.site,
            arrived: tick,
            visits: vec![],
            born: tick,
            alive: true,
            body_revision: 1,
            body: None,
            parts: Default::default(),
            traits: a.traits.clone(),
            accounts,
            skills: a.skills.clone(),
            tenets: Default::default(),
            disposition: a.disposition,
            soma: vec![],
            systems: Default::default(),
            varied: vec![],
            patch: None,
            authored: None,
        };
        let conserved = self.conserved.checked_add(issued).ok_or("matter overflow")?;
        let total = self.issued.checked_add(issued).ok_or("matter overflow")?;
        let id = match &a.character {
            None => self.state.population.insert(entity, 1)?,
            Some(key) => {
                let (id, held) = self.authored_entity(key).ok_or("no character asserted under that key")?;
                if held.place != crate::directing::PLACELESS || !held.alive {
                    return Err("a character that has already arrived".into());
                }
                let authored = held.authored.clone();
                *self.state.population.lift(id)? = Entity { authored, ..entity };
                id
            },
        };
        self.state.roots.insert(id);
        (self.conserved, self.issued) = (conserved, total);
        Ok(id)
    }

    /// Admits `body` as `entity`'s geometry, each living part laid with
    /// its lattice free and no tissue of its own; the body's revision moves.
    pub(crate) fn embody(&mut self, entity: Id, body: isometer_core::BodyDocument) -> Result<()> {
        let e = self.state.population.lift(entity)?;
        if !e.alive {
            return Err("a body for the dead".into());
        }
        let mut parts = BTreeMap::new();
        for part in body.living() {
            let mut p = e.parts.get(&part.id).cloned().unwrap_or_default();
            crate::mosaic::sync(&mut p, part.half_extent);
            parts.insert(part.id, p);
        }
        e.body = Some(body);
        e.parts = parts;
        e.body_revision = e.body_revision.saturating_add(1);
        crate::mosaic::ports::seed(e);
        Ok(())
    }

    /// `by` names `of`: a note `by` holds.
    pub(crate) fn name(&mut self, by: Id, of: Id, name: &str) -> Result<()> {
        let name = name.trim();
        if name.is_empty() || name.chars().count() > 64 {
            return Err("a name empty or past 64 characters".into());
        }
        if self.state.population.get(of).is_none() {
            return Err("naming nobody".into());
        }
        self.add_note(by, format!("entity:{of}"), NAME, name.into(), None, format!("name:{of}"))
    }

    /// The latest name anyone gave `of`, or the name it was authored with.
    pub fn name_of(&self, of: Id) -> Option<&str> {
        let about = format!("entity:{of}");
        let mut named = self.state.notes.iter().rev();
        let given = named.find(|n| n.core.kind == NAME && n.core.object == about);
        let authored = || self.state.population.get(of)?.authored.as_ref().map(|c| c.name.as_str());
        given.map(|n| n.djot.as_str()).or_else(authored)
    }
}
