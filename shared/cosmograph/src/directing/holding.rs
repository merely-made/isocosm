// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A handle for a lineage that has no critter body (cases 1216, 1217, 1221,
//! 1223, 1224, 1226): a participant holds a surface lineage or a world-class
//! one, steers a surface by its gene switches, and turns the world's cycles
//! within the bound the climate sets. A critter lineage is taken, never held.

use super::WORLD;
use crate::{Result, schema::*, simulation::Simulation, spread};
use std::collections::BTreeSet;

impl Simulation {
    /// The lineages `participant` holds, in key order.
    pub fn holds(&self, participant: Id) -> Vec<Key> {
        let held = self.state.lineages.iter().filter(|(_, l)| l.held.contains(&participant));
        held.map(|(key, _)| key.clone()).collect()
    }

    /// The participants that hold `lineage`, lowest first (case 1221).
    pub fn holders(&self, lineage: &str) -> Vec<Id> {
        let held = self.state.lineages.get(lineage).map(|l| l.held.iter().copied());
        held.into_iter().flatten().collect()
    }

    /// Whether `lineage` is a registered surface.
    pub fn is_surface(&self, lineage: &str) -> bool {
        self.state.spreads.identities.contains(&spread::Identity::Surface(lineage.into()))
    }

    /// `participant` holds `lineage`, beside whoever else does (case 1221).
    pub(crate) fn hold(&mut self, participant: Id, lineage: &str) -> Result<()> {
        self.participant(participant)?;
        let surface = self.is_surface(lineage);
        let line = self.state.lineages.get(lineage).ok_or("the lineage is unknown")?;
        if !surface && line.kingdom != WORLD {
            return Err("a critter lineage is taken, not held".into());
        }
        if line.held.contains(&participant) {
            return Err("the participant already holds this lineage".into());
        }
        if let Some(line) = self.state.lineages.get_mut(lineage) {
            line.held.insert(participant);
        }
        Ok(())
    }

    /// `participant` lets go of `lineage`.
    pub(crate) fn unhold(&mut self, participant: Id, lineage: &str) -> Result<()> {
        self.participant(participant)?;
        let line = self.state.lineages.get_mut(lineage).ok_or("the lineage is unknown")?;
        if !line.held.remove(&participant) {
            return Err("the participant does not hold this lineage".into());
        }
        Ok(())
    }

    fn require_held(&self, participant: Id, lineage: &str) -> Result<()> {
        self.participant(participant)?;
        let line = self.state.lineages.get(lineage).ok_or("the lineage is unknown")?;
        if !line.held.contains(&participant) {
            return Err("the participant does not hold this lineage".into());
        }
        Ok(())
    }

    /// A holder's standing stance on a surface: the same switch an authored
    /// `spread::Command::Switch` makes, conserved and checked as it is.
    pub(crate) fn steer(&mut self, participant: Id, lineage: &str,
        on: &BTreeSet<Key>, off: &BTreeSet<Key>) -> Result<String> {
        self.require_held(participant, lineage)?;
        let switch = spread::Command::Switch { lineage: lineage.into(), on: on.clone(), off: off.clone() };
        self.spread(&switch)
    }

    /// A holder of a world-class lineage shifts every temperature by `shift`
    /// degrees, within the climate's bound; nought puts the cycles back.
    pub(crate) fn turn(&mut self, participant: Id, lineage: &str, shift: i32) -> Result<()> {
        self.require_held(participant, lineage)?;
        if self.state.lineages.get(lineage).is_none_or(|l| l.kingdom != WORLD) {
            return Err("only a world-class lineage turns the cycles".into());
        }
        let climate = self.genesis.rules.climate.as_ref().ok_or("the world has no climate")?;
        let bound = climate.turn.ok_or("the world's cycles cannot be turned")?;
        if i64::from(shift).abs() > i64::from(bound) {
            return Err(format!("a turn of {shift} is beyond the bound of {bound}"));
        }
        self.state.turn = i64::from(shift);
        Ok(())
    }
}
