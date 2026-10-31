// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Public table reveals paired with authored native statements, never knowing.

use super::*;
use crate::native_table::table_digest;
use cosmograph::{Command, Execution, Session, asserted::{Assertion, Fact}};
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSecretResolved {
    pub publication: NativePublication,
    pub fact: WorldFact,
    pub expected_table: Option<String>,
}

fn matches_fact(native: &Fact, fact: &WorldFact) -> bool {
    !native.key.trim().is_empty() && native.about == fact.id && native.kind == fact.kind
        && native.text == fact.text && native.tags == fact.tags.iter().cloned().collect::<BTreeSet<_>>()
}

fn retained_note(session: &Session, native: &Fact) -> bool {
    let mut extra = BTreeMap::new();
    if !native.tags.is_empty() {
        extra.insert("tags".to_string(), native.tags.iter().cloned().collect::<Vec<_>>().join(" "));
    }
    session.sim.state().notes.iter().any(|note| note.core.cause == format!("assert:{}", native.key)
        && note.core.subject == native.about && note.core.object == native.key
        && note.core.kind == native.kind && note.djot == native.text
        && note.extra == extra && note.expires.is_none())
}

fn admitted_fact(session: &Session, fact: &WorldFact) -> bool {
    let mut pending: Vec<&Assertion> = session.entries.iter().filter_map(|entry| match &entry.command {
        Command::Assert(assertion) => Some(assertion), _ => None,
    }).collect();
    while let Some(assertion) = pending.pop() {
        match assertion {
            Assertion::Fact(native) if matches_fact(native, fact) && retained_note(session, native) => return true,
            Assertion::Storylet(applied) | Assertion::PackForced(applied) => pending.extend(&applied.asserts),
            _ => {},
        }
    }
    false
}

/// Native reveals require their genuine authored statement before table admission.
pub(crate) fn validate_reveal_fact(state: &GameSnapshot, fact: &WorldFact) -> Result<(), GameError> {
    let Some(world) = &state.native else { return Ok(()); };
    if fact.kind != "reveal" { return Ok(()); }
    if fact.id.trim().is_empty() { return Err(GameError::Native("native reveal has no authored fact id".into())); }
    let campaign = world.load(Execution::Grouped).map_err(GameError::Native)?;
    if !admitted_fact(campaign.table_session(), fact) {
        return Err(GameError::Native("public reveal lacks its exact admitted native statement".into()));
    }
    Ok(())
}

fn prepare(state: &GameSnapshot, fact: &WorldFact) -> Result<NativeSecretResolved, String> {
    if fact.kind != "reveal" || fact.id.trim().is_empty() {
        return Err("native secret reveal requires its authored id and reveal kind".into());
    }
    let previous = state.native.as_ref().ok_or("native reveal has no native world")?;
    let old = previous.load(Execution::Grouped)?;
    let table = state.native_table.as_ref().ok_or("native reveal needs its admitted host table")?;
    table.validate_history(old.table_session())?;
    let access = OrdinaryAccess { participant: ParticipantHandle(table.host_participant), seat: Seat::Dm,
        subject: old.table_session().sim.plays(table.host_participant).map(EntityHandle) };
    // Check the existing assertion door's actual DM cut without replacing
    // the authored string subject with an invented contract Pointable.
    let assertions = Vec::<cosmograph_overlay::vtt::Assertion>::new();
    let bytes = serde_json::to_vec(&assertions).map_err(|error| error.to_string())?;
    let (checked, game, _) = old.with_table_input(table.game.clone(), access, &bytes)?;
    if game != table.game { return Err("reveal changes unrelated product bytes".into()); }
    let admitted = if admitted_fact(checked.table_session(), fact) { checked } else {
        checked.with_assertions(&[Assertion::Fact(Fact { about: fact.id.clone(), key: fact.id.clone(),
            kind: fact.kind.clone(), text: fact.text.clone(), tags: fact.tags.iter().cloned().collect() })])
            .map_err(|error| error.to_string())?
    };
    if !admitted_fact(admitted.table_session(), fact) {
        return Err("native reveal did not retain its exact admitted statement".into());
    }
    Ok(NativeSecretResolved { publication: NativePublication::update(previous, &admitted)?,
        fact: fact.clone(), expected_table: table_digest(Some(table)) })
}

pub(crate) fn apply_native_secret(state: &mut GameSnapshot, result: &NativeSecretResolved)
    -> Result<(), GameError> {
    let fail = |why: &str| GameError::Native(why.into());
    if table_digest(state.native_table.as_ref()) != result.expected_table
        || !result.publication.boards.is_empty() || !result.publication.inventories.is_empty()
        || result.publication.activate.is_some() {
        return Err(fail("reveal changes its prior table cut or unrelated public content"));
    }
    let expected = prepare(state, &result.fact).map_err(GameError::Native)?;
    if expected != *result { return Err(fail("reveal differs from its exact re-admitted native statement")); }
    let mut candidate = state.clone();
    super::native::apply_publication(&mut candidate, &result.publication)?;
    validate_reveal_fact(&candidate, &result.fact)?;
    apply_game(&mut candidate, &GameEvent::Fact(result.fact.clone()))?;
    *state = candidate;
    Ok(())
}

impl HostSession {
    /// Both public halves complete before pending private custody can finish.
    pub(super) fn commit_reveal_fact(&mut self, fact: &WorldFact) -> Result<Vec<Outbound>, String> {
        let public = match self.state.journal.iter().find(|held| held.id == fact.id) {
            Some(held) if held != fact => return Err(format!("conflicting public campaign fact: {}", fact.id)),
            Some(_) => true,
            None => false,
        };
        let Some(previous) = &self.state.native else {
            return if public { Ok(Vec::new()) } else { self.try_commit(GameEvent::Fact(fact.clone())) };
        };
        let old = previous.load(Execution::Grouped)?;
        let native = admitted_fact(old.table_session(), fact);
        if native && public { return Ok(Vec::new()); }
        let result = prepare(&self.state, fact)?;
        let mut candidate = Self::with_history(self.state.clone(), self.campaign.clone(), self.history.clone());
        let out = candidate.try_commit(GameEvent::NativeSecretResolved(result))?;
        self.state = candidate.state;
        self.history = candidate.history;
        self.seq = candidate.seq;
        self.log_hash = candidate.log_hash;
        Ok(out)
    }
}
