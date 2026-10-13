// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored capacity overflow shares the native act's unpublished candidate.

use super::*;
use std::collections::BTreeSet;
use crate::simulation::State;

impl Simulation {
    pub(crate) fn commit_resolved(&mut self, mut stage: Stage, next: u64, process: &str) -> Result<()> {
        let terrain = stage.terrain.take();
        let step = stage.step.take();
        let births = std::mem::take(&mut stage.birth_places);
        if self.genesis.rules.spread_capacity.is_none() {
            self.commit(stage, next, process);
            self.commit_geometry(terrain);
            self.commit_step(step);
            self.place_births(&births);
            return Ok(());
        }
        let before = self.state.clone();
        let mut candidate = self.clone();
        candidate.commit(stage, next, process);
        candidate.commit_geometry(terrain);
        candidate.commit_step(step);
        crate::spread::dispatch_changed(&before, &mut candidate.state, &candidate.genesis.rules)?;
        self.publish_candidate(candidate)?;
        self.place_births(&births);
        Ok(())
    }

    /// Clocked native arrival has already journaled its residency changes.
    pub(crate) fn overflow_resolved(&mut self, before: &State) -> Result<()> {
        if self.genesis.rules.spread_capacity.is_none() { return Ok(()); }
        let mut candidate = self.clone();
        crate::spread::dispatch_changed(before, &mut candidate.state, &candidate.genesis.rules)?;
        self.publish_candidate(candidate)
    }

    fn publish_candidate(&mut self, mut candidate: Simulation) -> Result<()> {
        if candidate.matter() != candidate.conserved {
            return Err("resolved overflow would change conserved native matter".into());
        }
        let before = &self.state;
        let changed: BTreeSet<Id> = before.population.groups.keys()
            .chain(candidate.state.population.groups.keys()).copied()
            .filter(|id| before.population.groups.get(id) != candidate.state.population.groups.get(id))
            .collect();
        let sites: Vec<Id> = before.sites.iter().filter_map(|(id, site)|
            (candidate.state.sites.get(id) != Some(site)).then_some(*id)).collect();
        if let Some(journal) = &mut candidate.journal {
            journal.groups(&before.population, &changed.iter().copied().collect::<Vec<_>>());
            for id in &sites { journal.site(*id, &before.sites[id]); }
        }
        if let Some(pass) = &mut candidate.pass {
            for id in &changed { pass.lifting(&before.population, *id); }
        }
        if let Some(frame) = &mut candidate.frame {
            for (&first, group) in &before.population.groups {
                if changed.range(first..first + group.count).next().is_some() {
                    frame.keep_body(first, group.count, &group.entity);
                }
            }
            for id in &sites { frame.keep_site(*id, &before.sites[id]); }
        }
        if let Some(targets) = &mut candidate.targets {
            targets.touch(&candidate.state.population, changed.iter().copied());
        }
        if let Some(journeys) = &mut candidate.journeys {
            journeys.touch(&candidate.state.population, changed.iter().copied());
        }
        if let Some(filed) = &mut candidate.filed {
            filed.touch(&candidate.state.population, changed,
                candidate.state.tick, &candidate.genesis.rules);
        }
        *self = candidate;
        Ok(())
    }
}
