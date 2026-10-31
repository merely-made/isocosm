// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Exact authored records accompanying each accepted native process event.

use super::*;
use crate::rules::{Process,Rules};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Directive {
    Adopt { goal: Key, text: String },
    Participate { goals: BTreeSet<Key> },
    End { goal: Key, ending: Ending },
}

impl Process {
    pub(crate) fn validate_goals(&self, rules: &Rules) -> Result<()> {
        if self.goal_directives.is_empty() { return Ok(()); }
        if !self.note { return Err("goal directives require the process's genuine actor event".into()); }
        let mut adopted = BTreeSet::new();
        let mut served = BTreeSet::new();
        let mut ended = BTreeSet::new();
        let mut operations = 0usize;
        for directive in &self.goal_directives {
            let (kind,goals,seen) = match directive {
                Directive::Adopt { goal, .. } => (ADOPT,BTreeSet::from([goal.as_str()]),&mut adopted),
                Directive::Participate { goals } => (PARTICIPATE,goals.iter().map(String::as_str).collect(),&mut served),
                Directive::End { goal, .. } => (END,BTreeSet::from([goal.as_str()]),&mut ended),
            };
            if goals.is_empty() || !rules.note_kinds.contains(kind) {
                return Err("goal directive lacks its explicitly declared kind or served goals".into());
            }
            operations = operations.checked_add(goals.len()).ok_or("goal directive count overflow")?;
            for goal in goals {
                crate::validation::key(goal)?;
                if !seen.insert(goal) { return Err("process repeats the same goal directive".into()); }
            }
        }
        let effects = self.commitments.len().checked_add(self.effects.len())
            .and_then(|count| count.checked_add(self.risk.as_ref().map_or(0,|risk| risk.effects.len())))
            .ok_or("process effect count overflow")?;
        if operations.checked_add(effects).is_none_or(|count| count > rules.limits.operations) {
            return Err("goal directives and effects exceed native operation budget".into());
        }
        Ok(())
    }
}

impl Simulation {
    pub(crate) fn process_goal_notes(&self, actor: Id, event: &Event, staged: &[Note],
        next_action: u64, directives: &[Directive]) -> Result<Vec<Note>> {
        if directives.is_empty() { return Ok(Vec::new()); }
        if event.subject != actor || event.authored.is_some() || event.tick != self.state.tick {
            return Err("goal directives differ from the current native actor event".into());
        }
        let bound = self.frame.as_ref().and_then(|frame| frame.notes_bound()).unwrap_or(self.state.notes.len());
        let notes = self.state.notes.get(..bound).ok_or("pass-start native note prefix is unavailable")?;
        let subject = format!("entity:{actor}");
        let mut visible: Vec<_> = notes.iter().filter(|note| NOTE_KINDS.contains(&note.core.kind.as_str())
            || note.core.subject == subject && matches!(note.core.kind.as_str(),"sim:discover" | "sim:act"))
            .cloned().collect();
        visible.extend_from_slice(staged);
        let mut prepared = Vec::new();
        for directive in directives {
            let change = match directive {
                Directive::Adopt { goal,text } => Change::Adopt { actor,goal:goal.clone(),event:event.id.clone(),text:text.clone() },
                Directive::Participate { goals } => Change::Participate { actor,event:event.id.clone(),goals:goals.clone() },
                Directive::End { goal,ending } => Change::End { actor,goal:goal.clone(),event:event.id.clone(),ending:*ending },
            };
            let notes = self.goal_change_notes(&change,&visible,Some(event),next_action,true)?;
            visible.extend_from_slice(&notes);
            prepared.extend(notes);
        }
        Ok(prepared)
    }

    /// Identity publication cannot duplicate another act's adopted goal or ending.
    pub(crate) fn goal_notes_fit(&self, added: &[Note]) -> bool {
        records(added).all(|(note,change)| match change {
            Change::Adopt { .. } => !self.goal_notes().any(|(held,change)| held.core.object == note.core.object
                && matches!(change,Change::Adopt { .. })),
            Change::End { .. } => !self.goal_notes().any(|(held,change)| held.core.object == note.core.object
                && matches!(change,Change::End { .. })),
            Change::Participate { .. } => !self.goal_notes().any(|(held,change)| held.core.object == note.core.object
                && held.core.cause == note.core.cause && matches!(change,Change::Participate { .. })),
        })
    }
}
