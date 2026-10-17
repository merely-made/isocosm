// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Optional destination geometry never prevents the native journey clock landing.

use super::*;

impl Simulation {
    pub(crate) fn place_arrivals(&mut self, first: Id, count: u64, journey: &Journey) -> Result<()> {
        if !self.spatial_resolutions.enabled { return Ok(()); }
        let end = first.checked_add(count).ok_or("arrival cohort overflow")?;
        let inputs: Vec<_> = self.spatial_resolutions.arrivals.range(first..end)
            .map(|(_, input)| input.clone()).collect();
        for input in inputs {
            let r = &input.request;
            let patch = if r.from != journey.from || r.to != journey.to || r.departed != journey.departed
                || r.arrives != journey.arrives {
                Err("arrival offer names another native journey".into())
            } else { geometry::check_arrival(self, r, &input.geometry) };
            if let Ok(patch) = &patch {
                if let Some((&base, group)) = self.state.population.groups.range(..=r.actor).next_back() {
                    let last = base.checked_add(group.count).ok_or("arrival identity overflow")?;
                    let reached = [base, r.actor, r.actor.checked_add(1).filter(|id| *id < last).unwrap_or(base)];
                    if let Some(journal) = &mut self.journal { journal.groups(&self.state.population, &reached); }
                }
                self.state.population.lift(r.actor)?;
                self.state.population.groups.get_mut(&r.actor).ok_or("arrival member absent")?.entity_mut().patch = Some(*patch);
                let touched: Vec<_> = self.state.population.groups.range(first..end).map(|(id, _)| *id).collect();
                if let Some(targets) = &mut self.targets { targets.touch(&self.state.population, touched.iter().copied()); }
                if let Some(filed) = &mut self.filed {
                    filed.touch(&self.state.population, touched, self.state.tick, &self.genesis.rules);
                }
            }
            self.spatial_resolutions.arrived.insert(r.actor);
            self.spatial_resolutions.reports.push(ArrivalReport { actor: r.actor, patch });
        }
        Ok(())
    }
}
