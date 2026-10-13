// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit biological geometry for the existing dev ForceBirth command (848).
//! This admits neither arbitrary commands nor terrain/Step requests.

use super::*;
use crate::{Command, Session, flows::{Flow, FlowResult}};

pub enum BirthPreparation { Birth(BirthPending), Ready(PreparedBirth) }

pub struct PreparedBirth {
    candidate: Session, source: Key, pub outcome: String,
    pub births: Vec<BirthReport>, pub order: Vec<Publication>, flows: Vec<Flow>,
}

impl PreparedBirth {
    pub fn publish(self, session: &mut Session) -> Result<FlowResult<String>> {
        if crate::operations::source(session) != self.source {
            return Err("birth command candidate is stale".into());
        }
        let tick = self.candidate.sim.state().tick;
        *session = self.candidate;
        Ok(FlowResult { tick, result: self.outcome, flows: self.flows })
    }
}

impl Session {
    pub fn prepare_birth_command(&self, command: Command, births: &[BirthResolution])
        -> Result<BirthPreparation> {
        if !matches!(&command, Command::ForceBirth { .. }) {
            return Err("birth geometry wrapper accepts only ForceBirth".into());
        }
        let mut candidate = self.clone();
        let result = candidate.command_with_flows(Command::BirthSpatial {
            command: Box::new(command), births: births.to_vec(),
        });
        match result {
            Ok(result) => {
                let held = &mut candidate.sim.spatial_resolutions;
                let outcome = held.command_outcome.take().ok_or("native birth command outcome absent")?;
                Ok(BirthPreparation::Ready(PreparedBirth { source: crate::operations::source(self),
                    outcome, births: held.birth_reports.clone(), order: held.order.clone(),
                    candidate, flows: result.flows }))
            },
            Err(why) => match candidate.sim.spatial_resolutions.birth_pending.take() {
                Some(pending) => Ok(BirthPreparation::Birth(pending)),
                None => Err(why),
            },
        }
    }

    pub(crate) fn birth_spatial(&mut self, command: &Command, births: &[BirthResolution]) -> Result<String> {
        if !matches!(command, Command::ForceBirth { .. }) {
            return Err("birth geometry wrapper accepts only ForceBirth".into());
        }
        if births.len() as u128 > u128::from(self.sim.genesis().rules.limits.entities) {
            return Err("birth placement inputs exceed the authored entity limit".into());
        }
        let mut candidate = self.clone();
        let mut held = Resolutions { birth_enabled: true, ..Default::default() };
        for input in births {
            if held.births.insert(input.request.child, input.clone()).is_some() {
                return Err("duplicate biological child placement input".into());
            }
        }
        candidate.sim.spatial_resolutions = held;
        // ForceBirth keeps its original native stock issuance and act refusal.
        // Geometry cannot authorize a different process or supply its costs.
        let outcome = crate::history::run(&mut candidate.sim, command)?;
        if let Some(pending) = candidate.sim.spatial_resolutions.birth_pending.take() {
            self.sim.spatial_resolutions.birth_pending = Some(pending);
            return Err("native biological birth needs game geometry".into());
        }
        let held = &candidate.sim.spatial_resolutions;
        if held.born.len() != held.births.len() {
            return Err("birth input does not name an accepted biological child".into());
        }
        let reports = held.birth_reports.clone();
        let order = held.order.clone();
        let result = format!("birth spatial command; answers {}", crate::digest(&(&outcome, &reports, &order)));
        candidate.sim.geometry_resolutions = Default::default();
        candidate.sim.spatial_resolutions = Resolutions { birth_reports: reports, order,
            command_outcome: Some(outcome), ..Default::default() };
        *self = candidate;
        Ok(result)
    }
}
