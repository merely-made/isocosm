// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::rules::Process;

impl Simulation {
    pub(crate) fn stage_goal_directives(&self, stage: &mut Stage, process: &Process, next: u64) -> Result<()> {
        if process.goal_directives.is_empty() { return Ok(()); }
        crate::society()?;
        if stage.count != 1 { return Err("consequential goal directives require an individual native act".into()); }
        let event = &stage.event.as_ref().ok_or("goal directives have no actual staged native event")?.0;
        if event.process != process.id || stage.process != process.id || stage.action != event.id {
            return Err("goal directives name another native process or accepted action identity".into());
        }
        let notes = self.process_goal_notes(stage.actor,event,&stage.notes,next,&process.goal_directives)?;
        let count = self.state.notes.len().checked_add(stage.notes.len())
            .and_then(|count| count.checked_add(notes.len())).ok_or("staged goal note count overflow")?;
        if count > self.genesis.rules.limits.notes || !self.goal_notes_fit(&notes) {
            return Err("staged goal notes exhaust the budget or conflict with accepted goal identities".into());
        }
        stage.notes.extend(notes);
        Ok(())
    }
}
