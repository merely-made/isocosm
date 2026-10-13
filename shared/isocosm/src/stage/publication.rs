// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored capacity overflow shares the native act's unpublished candidate.

use super::*;
use std::collections::BTreeSet;
use crate::simulation::State;

impl Simulation {
    pub(crate) fn commit_resolved(&mut self, mut stage: Stage, next: u64, process: &str) -> Result<()> {
        let ecology = self.ecology_publication(&stage);
        let terrain = stage.terrain.take();
        let step = stage.step.take();
        let births = std::mem::take(&mut stage.birth_places);
        let institution = stage.institution_producer.take();
        if institution.is_none() && self.genesis.rules.spread_capacity.is_none() && self.genesis.rules.item_custody.is_none() {
            self.commit(stage, next, process);
            self.commit_geometry(terrain);
            self.commit_step(step);
            self.place_births(&births);
            self.published_ecology(ecology);
            return Ok(());
        }
        let before = self.state.clone();
        let mut candidate = self.clone();
        candidate.commit(stage, next, process);
        candidate.commit_geometry(terrain);
        candidate.commit_step(step);
        if let Some(institution)=institution {candidate.publish_institution_producer(institution)?;}
        candidate.check_custody_changes(self)?;
        candidate.synchronize_custody()?;
        crate::spread::dispatch_changed(&before, &mut candidate.state, &candidate.genesis.rules)?;
        self.publish_candidate(candidate)?;
        self.place_births(&births);
        self.published_ecology(ecology);
        Ok(())
    }

    /// Clocked native arrival has already journaled its residency changes.
    pub(crate) fn overflow_resolved(&mut self, before: &State) -> Result<()> {
        if self.genesis.rules.spread_capacity.is_none() && self.genesis.rules.item_custody.is_none() { return Ok(()); }
        let mut candidate = self.clone();
        candidate.synchronize_custody()?;
        crate::spread::dispatch_changed(before, &mut candidate.state, &candidate.genesis.rules)?;
        self.publish_candidate(candidate)
    }

    pub(crate) fn publish_candidate(&mut self, mut candidate: Simulation) -> Result<()> {
        candidate.check_custody_changes(self)?;
        candidate.synchronize_custody()?;
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
        let titles: BTreeSet<_> = before.projects.ownership.keys().chain(candidate.state.projects.ownership.keys())
            .filter(|key| before.projects.ownership.get(*key) != candidate.state.projects.ownership.get(*key))
            .cloned().collect();
        let produced: BTreeSet<_> = before.projects.produced.keys().chain(candidate.state.projects.produced.keys())
            .filter(|key| before.projects.produced.get(*key) != candidate.state.projects.produced.get(*key))
            .cloned().collect();
        if let Some(journal) = &mut candidate.journal {
            journal.groups(&before.population, &changed.iter().copied().collect::<Vec<_>>());
            for id in &sites { journal.site(*id, &before.sites[id]); }
        }
        if let Some(pass) = &mut candidate.pass {
            for id in &changed { pass.lifting(&before.population, *id); }
        }
        if let Some(frame) = &mut candidate.frame {
            for key in &titles { frame.keep_stock_lot(key,before.projects.ownership.get(key)); }
            for key in &produced { frame.keep_produced(key,before.projects.produced.get(key)); }
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
