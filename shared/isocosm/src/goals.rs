// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Consequential goal notes and their derived causal threads (D31,866).
//! Explicit participation is distinct from an act's immediate cause.

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const ADOPT: &str = "goal:adopt";
pub const PARTICIPATE: &str = "goal:participate";
pub const END: &str = "goal:end";
/// These kinds are authored vocabulary, never added to an older founding.
pub const NOTE_KINDS: [&str; 3] = [ADOPT, PARTICIPATE, END];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ending { Met, Abandoned }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Change {
    Adopt { actor: Id, goal: Key, event: Key, text: String },
    Participate { actor: Id, event: Key, goals: BTreeSet<Key> },
    End { actor: Id, goal: Key, event: Key, ending: Ending },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participation { pub actor: Id, pub event: Event }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct End { pub event: Event, pub ending: Ending }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CauseLink { pub event: Key, pub cause: Key }

/// Actual native state at adoption, captured internally and replayed exactly.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionCut { pub tick: Tick, pub next_action: u64, pub events: BTreeSet<Key> }

/// A reading of notes and events, not independently saved Arc state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thread {
    pub goal: Key,
    pub holder: Id,
    pub adoption: Event,
    pub recorded_at: Tick,
    pub text: String,
    pub participation: Vec<Participation>,
    pub ending: Option<End>,
    pub holder_alive: bool,
    pub causes: Vec<CauseLink>,
}

fn record(note: &Note) -> Option<Change> {
    if !NOTE_KINDS.contains(&note.core.kind.as_str()) { return None; }
    let change: Change = serde_json::from_str(note.extra.get("goal-record")?).ok()?;
    let (actor, event, kind, named) = match &change {
        Change::Adopt { actor,goal,event, .. } => (*actor,event,ADOPT,goal == &note.core.object),
        Change::Participate { actor,event,goals } => (*actor,event,PARTICIPATE,goals.contains(&note.core.object)),
        Change::End { actor,goal,event, .. } => (*actor,event,END,goal == &note.core.object),
    };
    (named && note.core.subject == format!("entity:{actor}") && note.core.cause == *event
        && note.core.kind == kind && note.core.stance == wing_impresa::Stance::Associate).then_some(change)
}

impl Simulation {
    fn goal_act(&self, actor: Id, event: &str) -> Result<&Event> {
        let act = self.state.events.get(event).ok_or("goal names no retained native act")?;
        if act.authored.is_some() || act.subject != actor || act.tick > self.state.tick
            || self.state.population.get(actor).is_none() {
            return Err("goal witness is not that actual native actor's accepted event".into());
        }
        Ok(act)
    }

    /// Admit an explicit goal record atomically. This grants no body control.
    pub(crate) fn goal_change(&mut self, change: &Change) -> Result<String> {
        let (actor,event,kind,goals) = match change {
            Change::Adopt { actor,goal,event, .. } => (*actor,event,ADOPT,BTreeSet::from([goal.clone()])),
            Change::Participate { actor,event,goals } => (*actor,event,PARTICIPATE,goals.clone()),
            Change::End { actor,goal,event, .. } => (*actor,event,END,BTreeSet::from([goal.clone()])),
        };
        let act = self.goal_act(actor,event)?;
        if !self.genesis.rules.note_kinds.contains(kind) || goals.is_empty() {
            return Err("goal record requires its declared kind and every explicitly served goal".into());
        }
        for goal in &goals {
            crate::validation::key(goal)?;
            match change {
                Change::Adopt { .. } => {
                    if self.goal_notes().any(|(_,held)| matches!(held,Change::Adopt { goal:was, .. } if &was == goal)) {
                        return Err("goal identity already has an accepted adoption".into());
                    }
                },
                _ => {
                    let thread = self.goal_thread(goal)?;
                    let cut = self.goal_adoption_cut(goal)?;
                    if (event != &thread.adoption.id && (act.tick < cut.tick || cut.events.contains(event)))
                        || thread.ending.is_some() || !thread.holder_alive {
                        return Err("participation names a not-yet-adopted or ended goal".into());
                    }
                    if let Change::End { .. } = change {
                        if thread.holder != actor { return Err("only its adopting actor ends this goal".into()); }
                    } else if thread.holder != actor && !self.knows(actor,&thread.adoption.id)? {
                        return Err("actor does not know the goal's actual adoption event".into());
                    }
                    if matches!(change,Change::Participate { .. })
                        && thread.participation.iter().any(|held| held.event.id == *event) {
                        return Err("goal already records this act's participation".into());
                    }
                },
            }
        }
        let held = self.state.notes.len().checked_add(goals.len()).ok_or("goal note count overflow")?;
        if held > self.genesis.rules.limits.notes { return Err("goal notes exceed native note budget".into()); }
        let encoded = serde_json::to_string(change).map_err(|error| error.to_string())?;
        let cut = if matches!(change,Change::Adopt { .. }) {
            Some(serde_json::to_string(&AdoptionCut { tick:self.state.tick,next_action:self.state.next_action,
                events:self.state.events.keys().cloned().collect() }).map_err(|error| error.to_string())?)
        } else { None };
        let mut notes = Vec::new();
        for goal in &goals {
            let text = match change { Change::Adopt { text, .. } => text.clone(), _ => String::new() };
            let mut note = self.note(actor,goal.clone(),kind,text,None,event.clone())?;
            note.extra.insert("goal-record".into(),encoded.clone());
            if let Some(cut) = &cut { note.extra.insert("goal-adoption-cut".into(),cut.clone()); }
            notes.push(note);
        }
        let mut candidate = self.clone();
        candidate.inspect(actor)?;
        candidate.state.notes.extend(notes);
        self.publish_candidate(candidate)?;
        Ok(format!("goals recorded: {}",goals.len()))
    }

    fn goal_notes(&self) -> impl Iterator<Item = (&Note,Change)> {
        self.state.notes.iter().filter_map(|note| record(note).map(|change| (note,change)))
    }

    pub fn goal_threads(&self) -> Result<Vec<Thread>> {
        let goals: BTreeSet<_> = self.goal_notes().filter_map(|(_,change)| match change {
            Change::Adopt { goal, .. } => Some(goal), _ => None,
        }).collect();
        goals.iter().map(|goal| self.goal_thread(goal)).collect()
    }

    /// Trusted source witness. Knowledge-filtered hooks do not expose other event IDs.
    pub fn goal_adoption_cut(&self, goal: &str) -> Result<AdoptionCut> {
        let mut notes = self.goal_notes().filter(|(_,change)| matches!(change,
            Change::Adopt { goal:named, .. } if named == goal));
        let (note,change) = notes.next().ok_or("goal has no accepted adoption cut")?;
        if notes.next().is_some() { return Err("goal has ambiguous adoption cuts".into()); }
        let Change::Adopt { event, .. } = change else { unreachable!() };
        let cut: AdoptionCut = serde_json::from_str(note.extra.get("goal-adoption-cut")
            .ok_or("goal adoption has no exact native event cut")?).map_err(|error| error.to_string())?;
        if cut.tick != note.core.tick || cut.tick > self.state.tick || cut.next_action > self.state.next_action
            || !cut.events.contains(&event) || cut.events.iter().any(|key|
                self.state.events.get(key).is_none_or(|event| event.tick > cut.tick)) {
            return Err("goal adoption cut differs from its actual native record/events".into());
        }
        Ok(cut)
    }

    /// Retained immediate cause edges and explicit goal service stay separate.
    pub fn goal_thread(&self, goal: &str) -> Result<Thread> {
        let mut adopted = None;
        let mut participation = Vec::new();
        let mut ending = None;
        for (note,change) in self.goal_notes().filter(|(note,_)| note.core.object == goal) {
            match change {
                Change::Adopt { actor,event,text, .. } => {
                    if adopted.is_some() { return Err("goal has ambiguous native adoption notes".into()); }
                    let event = self.goal_act(actor,&event)?.clone();
                    if note.core.tick < event.tick || note.djot != text { return Err("goal adoption note differs from its event/text".into()); }
                    adopted = Some((actor,event,text));
                },
                Change::Participate { actor,event, .. } => {
                    participation.push(Participation { actor,event:self.goal_act(actor,&event)?.clone() });
                },
                Change::End { actor,event,ending:why, .. } => {
                    if ending.is_some() { return Err("goal has multiple accepted endings".into()); }
                    ending = Some(End { event:self.goal_act(actor,&event)?.clone(),ending:why });
                },
            }
        }
        let (holder,adoption,text) = adopted.ok_or("no genuine consequential goal adoption")?;
        let cut = self.goal_adoption_cut(goal)?;
        participation.sort_by(|a,b| (a.event.tick,&a.event.id).cmp(&(b.event.tick,&b.event.id)));
        let mut seen = BTreeSet::new();
        let before_cut = |event: &Event| event.id != adoption.id
            && (event.tick < cut.tick || cut.events.contains(&event.id));
        if participation.iter().any(|part| before_cut(&part.event) || !seen.insert(&part.event.id))
            || ending.as_ref().is_some_and(|end| end.event.subject != holder || before_cut(&end.event)) {
            return Err("goal participation or ending differs from its native adoption".into());
        }
        let starts = std::iter::once(&adoption).chain(participation.iter().map(|part| &part.event))
            .chain(ending.iter().map(|end| &end.event));
        let mut causes = BTreeMap::new();
        for event in starts {
            let mut at = event;
            let mut visited = BTreeSet::new();
            while let Some(cause) = &at.cause {
                if !visited.insert(&at.id) { return Err("goal's native immediate cause chain cycles".into()); }
                causes.insert(at.id.clone(),cause.clone());
                let Some(parent) = self.state.events.get(cause) else { break; };
                at = parent;
            }
        }
        Ok(Thread { goal:goal.into(),holder,adoption,recorded_at:cut.tick,text,participation,ending,
            holder_alive:self.state.population.get(holder).is_some_and(|entity| entity.alive),
            causes:causes.into_iter().map(|(event,cause)| CauseLink { event,cause }).collect() })
    }
}
