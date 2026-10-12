// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One native tick, retried only on private candidates with exact game answers.

use super::*;
use crate::{Session, Command, operations, simulation::Work, flows::{Flow, FlowResult}};

pub enum Preparation { Terrain(operations::Pending), Step(Pending), Ready(Prepared) }

pub struct Prepared {
    candidate: Session, source: Key, pub work: Work,
    pub arrivals: Vec<ArrivalReport>, flows: Vec<Flow>,
    pub steps: BTreeSet<Key>,
}

impl Prepared {
    pub fn publish(self, session: &mut Session) -> Result<FlowResult<Work>> {
        if operations::source(session) != self.source { return Err("spatial candidate is stale".into()); }
        let tick = self.candidate.sim.state().tick;
        *session = self.candidate;
        Ok(FlowResult { tick, result: self.work, flows: self.flows })
    }
}

impl Session {
    pub fn prepare_spatial(&self, ticks: Tick, terrain: &[TerrainResolution],
        steps: &[Resolution], arrivals: &[ArrivalResolution]) -> Result<Preparation> {
        let mut candidate = self.clone();
        let command = Command::AdvanceSpatial {
            ticks, terrain: terrain.to_vec(), steps: steps.to_vec(), arrivals: arrivals.to_vec(),
        };
        match candidate.command_with_flows(command) {
            Ok(outcome) => {
                let work = candidate.sim.spatial_resolutions.work.take().ok_or("spatial work absent")?;
                Ok(Preparation::Ready(Prepared { source: operations::source(self), work,
                    arrivals: candidate.sim.spatial_resolutions.reports.clone(),
                    steps: candidate.sim.spatial_resolutions.accepted.clone(),
                    candidate, flows: outcome.flows }))
            },
            Err(why) => {
                if let Some(pending) = candidate.sim.spatial_resolutions.pending.take() {
                    Ok(Preparation::Step(pending))
                } else if let Some(pending) = candidate.sim.geometry_resolutions.pending.take() {
                    Ok(Preparation::Terrain(pending))
                } else { Err(why) }
            },
        }
    }

    pub(crate) fn advance_spatial(&mut self, ticks: Tick, terrain: &[TerrainResolution],
        steps: &[Resolution], arrivals: &[ArrivalResolution]) -> Result<String> {
        let count = terrain.len().checked_add(steps.len()).and_then(|n| n.checked_add(arrivals.len()))
            .ok_or("spatial input count overflow")?;
        if ticks != 1 || count > self.sim.genesis().rules.limits.events_per_advance {
            return Err("spatial advance needs one tick within the authored input budget".into());
        }
        let mut candidate = self.clone();
        candidate.sim.geometry_resolutions = operations::Resolutions { allow_refusals: true, ..Default::default() };
        let geometry = &mut candidate.sim.geometry_resolutions;
        for answer in terrain {
            if geometry.inputs.contains_key(&answer.request.action) || geometry.refused.contains_key(&answer.request.action) {
                return Err("duplicate terrain answer".into());
            }
            match &answer.geometry {
                Ok(actual) => { geometry.inputs.insert(answer.request.action.clone(), operations::Resolution {
                    request: answer.request.clone(), geometry: actual.clone() }); },
                Err(why) => { geometry.refused.insert(answer.request.action.clone(), (answer.request.clone(), why.clone())); },
            }
        }
        let mut held = Resolutions { enabled: true, ..Default::default() };
        for step in steps {
            if held.inputs.insert(step.request.action.clone(), step.clone()).is_some()
                || candidate.sim.geometry_resolutions.inputs.contains_key(&step.request.action)
                || candidate.sim.geometry_resolutions.refused.contains_key(&step.request.action)
            { return Err("duplicate spatial action input".into()); }
        }
        for arrival in arrivals {
            if held.arrivals.insert(arrival.request.actor, arrival.clone()).is_some() {
                return Err("duplicate spatial arrival input".into());
            }
        }
        candidate.sim.spatial_resolutions = held;
        let work = match candidate.advance(ticks) {
            Ok(work) => work,
            Err(why) => {
                self.sim.geometry_resolutions.pending = candidate.sim.geometry_resolutions.pending.take();
                self.sim.spatial_resolutions.pending = candidate.sim.spatial_resolutions.pending.take();
                return Err(why);
            },
        };
        let geometry = &candidate.sim.geometry_resolutions;
        let spatial = &candidate.sim.spatial_resolutions;
        if geometry.used.len() != geometry.inputs.len() + geometry.refused.len() || spatial.used.len() != spatial.inputs.len()
            || spatial.arrived.len() != spatial.arrivals.len()
        { return Err("spatial input does not name an executed native action or arrival".into()); }
        let reports = spatial.reports.clone();
        let accepted = spatial.accepted.clone();
        candidate.sim.geometry_resolutions = Default::default();
        candidate.sim.spatial_resolutions = Resolutions { work: Some(work), reports: reports.clone(),
            accepted: accepted.clone(),
            ..Default::default() };
        let outcome = format!("spatial advanced {ticks}; answers {}", crate::digest(&(reports, accepted)));
        *self = candidate;
        Ok(outcome)
    }
}
