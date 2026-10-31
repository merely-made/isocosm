// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Simulation {
    pub(crate) fn stage_graft_receipt(&self, stage: &mut Stage) -> Result<()> {
        let Some(record) = &stage.graft else { return Ok(()); };
        let event = stage.event.as_ref().map(|(event,_)| event).ok_or("graft carrying has no genuine actor event")?;
        if event.id != record.action || event.subject != record.actor
            || event.object != Some(record.donor) || event.process != record.process {
            return Err("graft receipt does not name its genuine actor event".into());
        }
        self.room(self.state.notes.len() + stage.notes.len())?;
        let mut note = self.note(record.actor,format!("entity:{}",record.donor),crate::graft::KIND,
            String::new(),None,record.action.clone())?;
        note.extra.insert("graft".into(),serde_json::to_string(record).map_err(|e|e.to_string())?);
        stage.notes.push(note);
        Ok(())
    }
}
