// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exact held copies; later truth changes do not refresh a disclosure (871).

use super::*;
use crate::knowing::{statements, terrain};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Details { pub statements: Vec<statements::Held>, pub terrain: Vec<terrain::Held> }

impl Details {
    pub fn ids(&self) -> Result<BTreeSet<Key>> {
        let mut ids = BTreeSet::new();
        for statement in &self.statements {
            statement.validate()?;
            if !ids.insert(statement.id.clone()) { return Err("disclosure repeats a held statement".into()); }
        }
        for terrain in &self.terrain {
            if terrain.hash != crate::digest(&terrain.source) || !ids.insert(terrain.id.clone()) {
                return Err("disclosure repeats or changes held terrain evidence".into());
            }
        }
        Ok(ids)
    }

    pub(super) fn held(sim: &Simulation, member: Id, selected: &BTreeSet<Key>) -> Result<Self> {
        if selected.is_empty() { return Err("rich disclosure must explicitly name held source records".into()); }
        let statements = sim.held_statements(member)?.into_iter().filter(|r| selected.contains(&r.id)).collect();
        let terrain = sim.held_terrain(member)?.into_iter().filter(|r| selected.contains(&r.id)).collect();
        let details = Self { statements, terrain };
        if details.ids()? != *selected { return Err("body does not hold every named disclosure source".into()); }
        Ok(details)
    }
}

impl Simulation {
    pub(super) fn participant_observer(&self, participant: Id, member: Id) -> Result<()> {
        if !self.state.population.get(participant).is_some_and(crate::directing::is_participant)
            || self.plays(participant) != Some(member) {
            return Err("knowledge input requires its actual native participant and played body".into());
        }
        Ok(())
    }

    pub(super) fn tell_detail(&mut self, member: Id, hearer: Id, record: &str) -> Result<Key> {
        let statement = self.held_statements(member)?.into_iter().any(|r| r.id == record);
        let terrain = self.held_terrain(member)?.into_iter().any(|r| r.id == record);
        match (statement, terrain) {
            (true, false) => self.tell_statement(member, hearer, record),
            (false, true) => self.tell_terrain(member, hearer, record),
            _ => Err("telling must name one uniquely held native detail".into()),
        }
    }
}
