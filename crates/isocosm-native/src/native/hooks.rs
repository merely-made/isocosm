// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The DM's statement over actual consequential-goal threads (D31,191,866).

use super::{GameBindings, movement::current_access};
use cosmograph::{Command, Session, asserted::{Assertion,HistoryLine}, goals, schema::{Id,Key,Tick}};
use cosmograph_overlay::{EventHandle, Intent, OrdinaryAccess, Seat, key_for_handle, key_handle,
    vtt::{HookIntent,VttIntent,VttIntentEnvelope}};
use cosmograph::directing::readings::Mode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoalHook { pub goal: goals::Thread, pub event: EventHandle }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookWitness { pub entry: Key, pub event_key: Key, pub event: EventHandle, pub tick: Tick }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HookRecord {
    pub participant: Id, pub goal: Key, pub adoption: Key,
    pub intent: HookIntent, pub witness: HookWitness,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Decision { participant: Id, goal: Key, adoption: Key, intent: HookIntent }

fn handle(intent: &HookIntent) -> EventHandle {
    match intent { HookIntent::TakeUp { arc } | HookIntent::Drop { arc }
        | HookIntent::Reshape { arc, .. } | HookIntent::TakeUpGoal { arc, .. }
        | HookIntent::DropGoal { arc, .. } | HookIntent::ReshapeGoal { arc, .. } => *arc }
}

fn named_goal(intent: &HookIntent) -> Option<&str> {
    match intent { HookIntent::TakeUpGoal { goal, .. } | HookIntent::DropGoal { goal, .. }
        | HookIntent::ReshapeGoal { goal, .. } => Some(goal), _ => None }
}

fn adopted_before(session: &Session, thread: &goals::Thread, order: u64) -> bool {
    let Some(entry) = session.entries.iter().find(|entry| entry.order == order) else { return false; };
    let Command::Assert(Assertion::History(line)) = &entry.command else { return false; };
    session.sim.goal_adoption_cut(&thread.goal).is_ok_and(|cut|
        cut.tick <= entry.tick && !cut.events.contains(&line.event_key()))
}

fn selected(session: &Session, intent: &HookIntent, order: Option<u64>) -> Result<goals::Thread,String> {
    let adoption = key_for_handle(session.sim.state().events.keys().map(String::as_str),handle(intent).0)
        .ok_or("hook handle names no unique native event")?;
    if let Some(goal) = named_goal(intent) {
        let thread = session.sim.goal_thread(goal)?;
        if thread.adoption.id != adoption || order.is_some_and(|order| !adopted_before(session,&thread,order)) {
            return Err("hook goal and adoption event name no already adopted native thread".into());
        }
        return Ok(thread);
    }
    let matching: Vec<_> = session.sim.goal_threads()?.into_iter()
        .filter(|thread| thread.adoption.id == adoption && order.is_none_or(|order| adopted_before(session,thread,order))).collect();
    let [thread] = matching.as_slice() else {
        return Err("legacy hook event must name exactly one genuine goal adoption".into());
    };
    Ok(thread.clone())
}

/// Each displayed act remains within the actual native knowing cut.
pub fn goal_hooks(session: &Session, observer: Id, mode: Mode) -> Result<Vec<GoalHook>, String> {
    let known = cosmograph::directing::readings::view(&session.sim,observer,mode);
    let mut hooks = Vec::new();
    for mut goal in session.sim.goal_threads()? {
        if !known.events.contains(&goal.adoption.id) { continue; }
        let numeric = key_handle(&goal.adoption.id);
        if key_for_handle(session.sim.state().events.keys().map(String::as_str),numeric)
            != Some(goal.adoption.id.as_str()) { return Err("goal event handle is ambiguous".into()); }
        goal.participation.retain(|part| known.events.contains(&part.event.id));
        if goal.ending.as_ref().is_some_and(|end| !known.events.contains(&end.event.id)) { goal.ending = None; }
        goal.causes.retain(|link| known.events.contains(&link.event) && known.events.contains(&link.cause));
        hooks.push(GoalHook { goal,event:EventHandle(numeric) });
    }
    Ok(hooks)
}

impl HookRecord {
    fn decision(&self) -> Decision { Decision { participant:self.participant,goal:self.goal.clone(),
        adoption:self.adoption.clone(),intent:self.intent.clone() } }

    fn validate(&self, session: &Session) -> Result<u64, String> {
        let entry = session.entries.iter().find(|entry| entry.id == self.witness.entry)
            .ok_or("hook accepted entry is absent")?;
        let thread = selected(session,&self.intent,Some(entry.order))?;
        if thread.goal != self.goal || thread.adoption.id != self.adoption || handle(&self.intent).0 != key_handle(&self.adoption)
            || key_for_handle(session.sim.state().events.keys().map(String::as_str),handle(&self.intent).0)
                != Some(self.adoption.as_str())
            || !session.sim.state().population.get(self.participant).is_some_and(cosmograph::directing::is_participant) {
            return Err("hook differs from its genuine native goal/participant".into());
        }
        let Command::Assert(Assertion::History(line)) = &entry.command else {
            return Err("hook has no native history assertion".into());
        };
        let decision: Decision = serde_json::from_str(&line.text).map_err(|error| error.to_string())?;
        let event = session.sim.state().events.get(&self.witness.event_key).ok_or("hook native event is absent")?;
        if decision != self.decision() || line.key != format!("vtt:hook:{}:{}",entry.order,entry.tick)
            || line.kind != "vtt:hook" || line.time != i64::try_from(entry.tick).map_err(|_| "hook native tick exceeds authored time")?
            || line.participants != vec![format!("entity:{}",self.participant)] || line.place.is_some()
            || !line.tags.is_empty() || line.event_key() != self.witness.event_key
            || event.authored.as_ref() != Some(line) || event.tick != entry.tick
            || entry.tick != self.witness.tick || self.witness.event.0 != key_handle(&self.witness.event_key)
            || key_for_handle(session.sim.state().events.keys().map(String::as_str),self.witness.event.0)
                != Some(self.witness.event_key.as_str()) || entry.outcome != self.witness.event_key {
            return Err("hook witness differs from its actual native entry/event".into());
        }
        Ok(entry.order)
    }
}

pub fn validate_bindings(session: &Session, records: &[HookRecord]) -> Result<(), String> {
    let mut previous = None;
    let mut seen = BTreeSet::new();
    for record in records {
        let order = record.validate(session)?;
        if previous.is_some_and(|prior| order <= prior) || !seen.insert(&record.witness.entry) {
            return Err("hook witnesses repeat or change accepted native order".into());
        }
        previous = Some(order);
    }
    Ok(())
}

/// The actual host admits DM access, then publishes native+game bytes together.
pub fn submit_hook(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    envelope: &VttIntentEnvelope) -> Result<(String,Vec<u8>), String> {
    current_access(session,access)?;
    if access.seat != Seat::Dm || access.participant != envelope.participant
        || envelope.tick.0 != session.sim.state().tick { return Err("hook needs current authenticated DM input".into()); }
    let Intent::Game(VttIntent::Hook(intent)) = &envelope.intent else { return Err("input is not a native hook decision".into()); };
    let thread = selected(session,intent,None)?;
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut candidate = session.clone();
    let tick = candidate.sim.state().tick;
    let decision = Decision { participant:access.participant.0,goal:thread.goal,
        adoption:thread.adoption.id,intent:intent.clone() };
    let line = HistoryLine { key:format!("vtt:hook:{}:{tick}",candidate.entries.len()),
        time:i64::try_from(tick).map_err(|_| "hook native tick exceeds authored time")?,kind:"vtt:hook".into(),
        text:serde_json::to_string(&decision).map_err(|error| error.to_string())?,
        participants:vec![format!("entity:{}",access.participant.0)],place:None,tags:BTreeSet::new() };
    let event_key = line.event_key();
    if candidate.sim.state().events.contains_key(&event_key) { return Err("hook history identity already exists".into()); }
    let outcome = candidate.command(Command::Assert(Assertion::History(line)))?;
    let entry = candidate.entries.last().ok_or("hook created no accepted native entry")?;
    let record = HookRecord { participant:decision.participant,goal:decision.goal,adoption:decision.adoption,
        intent:decision.intent,witness:HookWitness { entry:entry.id.clone(),event:EventHandle(key_handle(&event_key)),
            event_key,tick } };
    record.validate(&candidate)?;
    bindings.record_hook(record);
    bindings.validate(&candidate)?;
    let next = bindings.encode()?;
    *session = candidate;
    Ok((outcome,next))
}

/// Replicated hook publication can append only this exact typed decision.
pub fn validate_transition(before: &Session, before_game: &[u8], after: &Session, after_game: &[u8]) -> Result<HookRecord,String> {
    let old = before.save();
    let next = after.save();
    let a = GameBindings::decode(before_game)?;
    let b = GameBindings::decode(after_game)?;
    a.validate(before)?;
    b.validate(after)?;
    if old.genesis != next.genesis || old.branch != next.branch || old.movement != next.movement
        || old.tick != next.tick || old.checkpoints != next.checkpoints
        || next.entries.len() != old.entries.len() + 1 || !next.entries.starts_with(&old.entries)
        || !a.same_except_hooks(&b) || b.hooks().len() != a.hooks().len() + 1 || !b.hooks().starts_with(a.hooks()) {
        return Err("hook transition changes its native prefix, tick or unrelated game bindings".into());
    }
    let record = b.hooks().last().ok_or("hook transition lacks its typed record")?.clone();
    if record.witness.entry != next.entries.last().ok_or("hook transition lacks its native entry")?.id {
        return Err("hook record is not the newly appended actual native event".into());
    }
    record.validate(after)?;
    let mut expected = a;
    expected.record_hook(record.clone());
    if expected != b { return Err("hook transition rewrites its retained history".into()); }
    Ok(record)
}
