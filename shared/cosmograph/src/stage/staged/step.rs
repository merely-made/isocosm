// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::spatial::{Request, StagedStep};

impl Staged<'_> {
    pub(super) fn step(&mut self, action: &str, key: &str) -> Result<()> {
        if self.stage.step.is_some() || self.stage.terrain.is_some() || self.stage.count != 1 {
            return Err("step needs one geometry act on one native member".into());
        }
        let profile = self.sim.genesis.rules.step_profiles.get(key).ok_or("step profile absent")?.clone();
        if profile.process != self.stage.process { return Err("step profile names another chosen process".into()); }
        let cells = self.compute(|read, draw, parts| profile.cells.resolve(read, draw, parts))?.resolved()?;
        if cells == 0 { return Err("chosen step has no resolved cell budget".into()); }
        let actor = &self.stage.bodies[&self.stage.actor];
        let request = Request {
            action: action.into(), process: self.stage.process.clone(), profile: key.into(),
            tick: self.sim.state.tick, revision: self.sim.revision.clone(), actor: self.stage.actor,
            target: self.stage.target, site: actor.resident_site().ok_or("step actor is travelling")?,
            patch: actor.patch.ok_or("step actor has no admitted native place")?,
            body_revision: actor.body_revision, terrain_revision: self.sim.terrain_revision(), cells,
        };
        let mut step = StagedStep { request: request.clone(), resolved: false };
        let Some(resolution) = self.sim.spatial_resolutions.inputs.get(action) else {
            self.stage.step = Some(step);
            return Err("native step needs game geometry".into());
        };
        if resolution.request != request { return Err("step geometry names another native action".into()); }
        step.resolved = true;
        self.stage.step = Some(step);
        let geometry = resolution.geometry.as_ref().map_err(|why| format!("game step refused: {why}"))?;
        let patch = crate::spatial::check_step(self.sim, &request, geometry)?;
        self.actor().patch = Some(patch);
        Ok(())
    }
}
