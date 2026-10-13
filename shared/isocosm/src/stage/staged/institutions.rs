// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Staged<'_> {
    pub(super) fn institution(&mut self,key:&str,action:&str) -> Result<()> {
        if self.stage.institution_producer.is_some() || self.stage.count!=1
            || self.stage.demands.is_none() && !self.stage.scheduled
            || self.stage.action!=action
            || self.sim.owned_action(self.stage.actor,&self.stage.process,self.stage.act)!=action {
            return Err("institution producer is repeated or differs from its actual native action".into());
        }
        self.stage.institution_producer=Some(self.sim.stage_institution_producer(self.stage.actor,
            self.stage.target,key,action,self.stage.demands.is_some())?);
        Ok(())
    }
}
