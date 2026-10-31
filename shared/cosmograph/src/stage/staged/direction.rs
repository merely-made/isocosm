// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::BTreeSet;

impl Staged<'_> {
    pub(super) fn direct_overflow(&mut self, from: Id, destinations: &BTreeSet<Id>) -> Result<()> {
        if self.stage.direction.is_some() { return Err("one act directs its territory overflow once".into()); }
        if self.stage.count != 1 { return Err("territory direction names one actual native entity".into()); }
        let body = &self.stage.bodies[&self.stage.actor];
        if !body.alive || body.method == Method::Inert { return Err("territory direction needs an active native body".into()); }
        let process = self.sim.genesis.rules.processes.get(&self.stage.process).ok_or("direction process absent")?;
        self.sim.direction_ready(self.stage.actor, process)?;
        if body.method == Method::Deliberative && !self.sim.owned_planning
            && !self.sim.institution_chosen(self.stage.actor, &self.stage.process) {
            return Err("a deliberative territory directs only its actually chosen native process".into());
        }
        self.stage.direction = Some(crate::spread::Direction { actor: self.stage.actor,
            from, destinations: destinations.clone() });
        Ok(())
    }
}
