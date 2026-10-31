// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::operations::{Change, Request, StagedTerrain};

impl Staged<'_> {
    pub(super) fn terrain(&mut self, action: &str, change: &Change,
        cells: &Amount, reach: u64) -> Result<()> {
        if self.stage.terrain.is_some() || self.stage.step.is_some() || self.stage.count != 1 {
            return Err("terrain geometry needs one operation on one native member".into());
        }
        let body = &self.stage.bodies[&self.stage.actor];
        let site = body.resident_site().ok_or("terrain actor is not resident")?;
        let patch = body.patch.ok_or("terrain actor has no admitted native place")?;
        let request = Request {
            action: action.into(), process: self.stage.process.clone(), tick: self.sim.state.tick,
            revision: self.sim.revision.clone(), actor: self.stage.actor, target: self.stage.target,
            site, patch, body_revision: body.body_revision,
            terrain_revision: self.sim.terrain_revision(), change: change.clone(),
            cells: cells.resolved()?, reach,
        };
        let mut terrain = StagedTerrain { request: request.clone(), answered: false, edit: None, gained: 0, paid: 0 };
        if let Some((declared, why)) = self.sim.geometry_resolutions.refused.get(action) {
            if declared != &request { return Err("terrain refusal names another native action".into()); }
            terrain.answered = true;
            self.stage.terrain = Some(terrain);
            return Err(format!("game terrain refused: {why}"));
        }
        let Some(resolved) = self.sim.geometry_resolutions.inputs.get(action) else {
            self.stage.terrain = Some(terrain);
            return Err("native operation needs game geometry".into());
        };
        if resolved.request != request { return Err("geometry names another native operation".into()); }
        terrain.answered = true;
        self.stage.terrain = Some(terrain.clone());
        let (gained, paid) = crate::operations::geometry::price(self.sim, &request, &resolved.geometry)?;
        for (key, amount) in gained {
            credit(&mut self.stage.bodies.get_mut(&self.stage.actor).map(Arc::make_mut).ok_or("terrain actor absent")?.accounts, &key, amount)?;
            terrain.gained += u128::from(amount);
        }
        for (key, amount) in paid {
            self.guard_owned_debit(Binding::Actor,&key,amount)?;
            debit(&mut self.stage.bodies.get_mut(&self.stage.actor).map(Arc::make_mut).ok_or("terrain actor absent")?.accounts,&key,amount)?;
            terrain.paid += u128::from(amount);
        }
        self.sim.conserved.checked_add(terrain.gained).and_then(|n| n.checked_sub(terrain.paid))
            .ok_or("terrain conserved mass overflow")?;
        terrain.edit = Some(resolved.geometry.edit.clone());
        self.stage.terrain = Some(terrain);
        Ok(())
    }
}
