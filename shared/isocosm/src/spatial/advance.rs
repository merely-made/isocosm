// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One native tick, retried only on private candidates with exact game answers.

use super::*;
use crate::{Session, Command, operations, simulation::Work, flows::{Flow, FlowResult}};

pub enum Preparation { Terrain(operations::Pending), Step(Pending), Ready(Prepared), Birth(BirthPending) }

/// One spatial clock attempt. Pending requests leave the supplied Session
/// unchanged; Advanced publishes its private candidate synchronously.
#[must_use]
pub enum SpatialAdvance {
    Terrain(operations::Pending),
    Step(Pending),
    Birth(BirthPending),
    Advanced(FlowResult<Work>),
}

enum StagedAdvance {
    Terrain(operations::Pending),
    Step(Pending),
    Birth(BirthPending),
    Ready { candidate: Session, work: Work, flows: Vec<Flow> },
}

pub struct Prepared {
    candidate: Session, source: Key, pub work: Work,
    pub arrivals: Vec<ArrivalReport>, flows: Vec<Flow>,
    pub steps: BTreeSet<Key>,
    pub births: Vec<BirthReport>,
    pub order: Vec<Publication>,
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
        let command = Command::AdvanceSpatial {
            ticks, terrain: terrain.to_vec(), steps: steps.to_vec(), arrivals: arrivals.to_vec(),
        };
        self.prepare_spatial_command(command)
    }

    pub fn prepare_birth_spatial(&self, ticks: Tick, terrain: &[TerrainResolution],
        steps: &[Resolution], arrivals: &[ArrivalResolution], births: &[BirthResolution]) -> Result<Preparation> {
        let command = Command::AdvanceBirthSpatial {
            ticks, terrain: terrain.to_vec(), steps: steps.to_vec(), arrivals: arrivals.to_vec(), births: births.to_vec(),
        };
        self.prepare_spatial_command(command)
    }

    /// Prepares and publishes one owned spatial tick without hashing the
    /// source Session: the exclusive borrow makes it impossible to change
    /// between staging and publication. Pending requests do not mutate it.
    pub fn try_advance_birth_spatial_with_flows(&mut self, ticks: Tick,
        terrain: &[TerrainResolution], steps: &[Resolution], arrivals: &[ArrivalResolution],
        births: &[BirthResolution]) -> Result<SpatialAdvance> {
        let command = Command::AdvanceBirthSpatial {
            ticks, terrain: terrain.to_vec(), steps: steps.to_vec(), arrivals: arrivals.to_vec(), births: births.to_vec(),
        };
        match self.stage_spatial_command(command)? {
            StagedAdvance::Terrain(pending) => Ok(SpatialAdvance::Terrain(pending)),
            StagedAdvance::Step(pending) => Ok(SpatialAdvance::Step(pending)),
            StagedAdvance::Birth(pending) => Ok(SpatialAdvance::Birth(pending)),
            StagedAdvance::Ready { candidate, work, flows } => {
                let tick = candidate.sim.state().tick;
                *self = candidate;
                Ok(SpatialAdvance::Advanced(FlowResult { tick, result: work, flows }))
            },
        }
    }

    fn prepare_spatial_command(&self, command: Command) -> Result<Preparation> {
        match self.stage_spatial_command(command)? {
            StagedAdvance::Terrain(pending) => Ok(Preparation::Terrain(pending)),
            StagedAdvance::Step(pending) => Ok(Preparation::Step(pending)),
            StagedAdvance::Birth(pending) => Ok(Preparation::Birth(pending)),
            StagedAdvance::Ready { candidate, work, flows } => Ok(Preparation::Ready(Prepared {
                source: operations::source(self), work,
                arrivals: candidate.sim.spatial_resolutions.reports.clone(),
                steps: candidate.sim.spatial_resolutions.accepted.clone(),
                births: candidate.sim.spatial_resolutions.birth_reports.clone(),
                order: candidate.sim.spatial_resolutions.order.clone(), candidate, flows,
            })),
        }
    }

    fn stage_spatial_command(&self, command: Command) -> Result<StagedAdvance> {
        let _span = crate::profile::Span::new("spatial.prepare");
        let mut candidate = self.clone();
        match candidate.command_with_flows(command) {
            Ok(outcome) => {
                let work = candidate.sim.spatial_resolutions.work.take().ok_or("spatial work absent")?;
                Ok(StagedAdvance::Ready { candidate, work, flows: outcome.flows })
            },
            Err(why) => {
                if let Some(pending) = candidate.sim.spatial_resolutions.birth_pending.take() {
                    Ok(StagedAdvance::Birth(pending))
                } else if let Some(pending) = candidate.sim.spatial_resolutions.pending.take() {
                    Ok(StagedAdvance::Step(pending))
                } else if let Some(pending) = candidate.sim.geometry_resolutions.pending.take() {
                    Ok(StagedAdvance::Terrain(pending))
                } else { Err(why) }
            },
        }
    }

    pub(crate) fn advance_spatial(&mut self, ticks: Tick, terrain: &[TerrainResolution],
        steps: &[Resolution], arrivals: &[ArrivalResolution]) -> Result<String> {
        self.advance_spatial_inputs(ticks, terrain, steps, arrivals, None)
    }

    pub(crate) fn advance_birth_spatial(&mut self, ticks: Tick, terrain: &[TerrainResolution],
        steps: &[Resolution], arrivals: &[ArrivalResolution], births: &[BirthResolution]) -> Result<String> {
        self.advance_spatial_inputs(ticks, terrain, steps, arrivals, Some(births))
    }

    fn advance_spatial_inputs(&mut self, ticks: Tick, terrain: &[TerrainResolution],
        steps: &[Resolution], arrivals: &[ArrivalResolution], births: Option<&[BirthResolution]>) -> Result<String> {
        let _span = crate::profile::Span::new("spatial.command");
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
        if let Some(inputs) = births {
            if inputs.len() as u128 > u128::from(self.sim.genesis().rules.limits.entities) {
                return Err("birth placement inputs exceed the authored entity limit".into());
            }
            held.birth_enabled = true;
            for input in inputs {
                if held.births.insert(input.request.child, input.clone()).is_some() {
                    return Err("duplicate biological child placement input".into());
                }
            }
        }
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
                self.sim.spatial_resolutions.birth_pending = candidate.sim.spatial_resolutions.birth_pending.take();
                return Err(why);
            },
        };
        let geometry = &candidate.sim.geometry_resolutions;
        let spatial = &candidate.sim.spatial_resolutions;
        if geometry.used.len() != geometry.inputs.len() + geometry.refused.len() || spatial.used.len() != spatial.inputs.len()
            || spatial.arrived.len() != spatial.arrivals.len()
            || spatial.born.len() != spatial.births.len()
        { return Err("spatial input does not name an executed native action or arrival".into()); }
        let reports = spatial.reports.clone();
        let accepted = spatial.accepted.clone();
        let born = spatial.birth_reports.clone();
        let order = spatial.order.clone();
        candidate.sim.geometry_resolutions = Default::default();
        candidate.sim.spatial_resolutions = Resolutions { work: Some(work), reports: reports.clone(),
            accepted: accepted.clone(),
            birth_reports: born.clone(),
            order: order.clone(),
            ..Default::default() };
        let outcome = if births.is_some() {
            format!("birth spatial advanced {ticks}; answers {}", crate::digest(&(reports, accepted, born, order)))
        } else { format!("spatial advanced {ticks}; answers {}", crate::digest(&(reports, accepted))) };
        *self = candidate;
        Ok(outcome)
    }
}
