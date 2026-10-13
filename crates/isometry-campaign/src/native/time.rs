// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Individual native downtime records (246), with explicit host roster input.
//! Roster selection and advance validity remain gated on rulings 851/852.

use super::GameBindings;
use isocosm::{Command, Session, asserted::{Assertion, HistoryLine}, schema::{Id, Key, Tick}};
use isocosm_overlay::{EventHandle, Intent, OrdinaryAccess, Seat,
    key_for_handle, key_handle, vtt::{TimeIntent, VttIntent, VttIntentEnvelope}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The trusted host supplies this authenticated roster, never request bytes or
/// player names. Its preimage identifies the actual declared roster source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roster { pub players: BTreeSet<Id>, pub preimage: String }

impl Roster {
    pub fn validate(&self, session: &Session) -> Result<(), String> {
        if self.preimage.is_empty() || self.players.iter().any(|id|
            !session.sim.state().population.get(*id).is_some_and(isocosm::directing::is_participant)) {
            return Err("downtime roster lacks its authenticated preimage or native participants".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration { pub proposer: Id, pub tick: Tick, pub ticks: Tick, pub roster: Roster }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness { pub entry: Key, pub event_key: Key, pub event: EventHandle, pub tick: Tick }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consent { pub participant: Id, pub roster: Roster, pub witness: Witness }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Completion { pub from: Tick, pub to: Tick, pub roster: Roster, pub witness: Witness }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub declaration: Declaration,
    pub witness: Witness,
    pub consents: Vec<Consent>,
    pub completed: Option<Completion>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Record {
    Proposed(Declaration),
    Consented { participant: Id, proposal: Key, event: EventHandle, roster: Roster },
}

impl Witness {
    fn validate(&self, session: &Session, record: &Record) -> Result<(), String> {
        let entry = session.entries.iter().find(|entry| entry.id == self.entry)
            .ok_or("native time witness entry is absent")?;
        let Command::Assert(Assertion::History(line)) = &entry.command else {
            return Err("native time witness is not an accepted history record".into());
        };
        let event = session.sim.state().events.get(&self.event_key).ok_or("native time event is absent")?;
        if line.event_key() != self.event_key || key_handle(&self.event_key) != self.event.0
            || key_for_handle(session.sim.state().events.keys().map(String::as_str), self.event.0)
                != Some(self.event_key.as_str())
            || event.authored.as_ref() != Some(line) || entry.tick != self.tick || event.tick != self.tick
            || entry.outcome != self.event_key
            || line.kind != "vtt:time" || line.place.is_some() || !line.participants.is_empty()
            || !line.tags.is_empty() || i64::try_from(self.tick).ok() != Some(line.time)
            || line.text != serde_json::to_string(record).map_err(|error| error.to_string())?
            || serde_json::from_str::<Record>(&line.text).map_err(|error| error.to_string())? != *record {
            return Err("native time entry, actual event and typed record differ".into());
        }
        Ok(())
    }
}

fn append(session: &mut Session, record: &Record) -> Result<Witness, String> {
    let tick = session.sim.state().tick;
    let line = HistoryLine { key:format!("vtt:time:{}:{tick}", session.entries.len()),
        time:i64::try_from(tick).map_err(|_| "native tick exceeds authored history timestamp representation")?,
        kind:"vtt:time".into(), text:serde_json::to_string(record).map_err(|error| error.to_string())?,
        participants:Vec::new(), place:None, tags:BTreeSet::new() };
    let event_key = line.event_key();
    if session.sim.state().events.contains_key(&event_key) {
        return Err("native time append key already names an earlier event".into());
    }
    session.command(Command::Assert(Assertion::History(line)))?;
    let witness = Witness { entry:session.entries.last().ok_or("native time entry is absent")?.id.clone(),
        event:EventHandle(key_handle(&event_key)), event_key, tick };
    witness.validate(session, record)?;
    Ok(witness)
}

impl Proposal {
    pub fn validate(&self, session: &Session, key: &str) -> Result<(), String> {
        self.declaration.roster.validate(session)?;
        if key != self.witness.event_key || self.declaration.tick != self.witness.tick
            || self.declaration.roster.players.contains(&self.declaration.proposer)
            || !session.sim.state().population.get(self.declaration.proposer)
                .is_some_and(isocosm::directing::is_participant) {
            return Err("downtime declaration differs from native participant/roster/tick".into());
        }
        self.witness.validate(session, &Record::Proposed(self.declaration.clone()))?;
        let declared = session.entries.iter().find(|entry| entry.id == self.witness.entry)
            .ok_or("downtime declaration entry absent")?.order;
        let mut prior = declared;
        for consent in &self.consents {
            consent.roster.validate(session)?;
            if !consent.roster.players.contains(&consent.participant)
                || consent.roster.players.contains(&self.declaration.proposer) {
                return Err("consent participant is absent from its authenticated roster".into());
            }
            consent.witness.validate(session, &Record::Consented { participant:consent.participant,
                proposal:self.witness.event_key.clone(), event:self.witness.event, roster:consent.roster.clone() })?;
            let order = session.entries.iter().find(|entry| entry.id == consent.witness.entry)
                .ok_or("native consent entry absent")?.order;
            if order <= prior { return Err("native consent records are out of order".into()); }
            prior = order;
        }
        if self.completed.is_some() { return Err("downtime completion awaits its ruled activation policy".into()); }
        Ok(())
    }
}

/// Records only: no clock advance, inferred roster, expiry or reset policy.
/// Actual callers supply the same admitted seat/participant and roster source.
pub fn submit_time(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess, roster: &Roster,
    envelope: &VttIntentEnvelope) -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    if envelope.participant != access.participant || envelope.tick.0 != session.sim.state().tick {
        return Err("native time input names another participant or tick".into());
    }
    roster.validate(session)?;
    let Intent::Game(VttIntent::Time(intent)) = &envelope.intent else {
        return Err("native time requires its typed table time input".into());
    };
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut candidate = session.clone();
    let accepted = match intent {
        TimeIntent::Downtime { ticks } => {
            if access.seat != Seat::Dm || roster.players.contains(&access.participant.0) {
                return Err("the distinct DM declares downtime; it is not a player's consent".into());
            }
            let declaration = Declaration { proposer:access.participant.0, tick:envelope.tick.0,
                ticks:*ticks, roster:roster.clone() };
            let witness = append(&mut candidate, &Record::Proposed(declaration.clone()))?;
            let key = witness.event_key.clone();
            let handle = witness.event.0;
            bindings.record_downtime(key.clone(), Proposal { declaration, witness, consents:Vec::new(), completed:None });
            format!("downtime proposed: {key} ({handle}); activation awaits roster/time policy")
        },
        TimeIntent::Consent { downtime } => {
            if access.seat != Seat::Player || !roster.players.contains(&access.participant.0) {
                return Err("only that authenticated native player may give its yes".into());
            }
            let key = key_for_handle(candidate.sim.state().events.keys().map(String::as_str), downtime.0)
                .ok_or("downtime handle has no unique actual native event")?.to_owned();
            let mut proposal = bindings.downtime().get(&key).ok_or("event is not a recorded downtime proposal")?.clone();
            if proposal.completed.is_some() { return Err("downtime already completed".into()); }
            if roster.players.contains(&proposal.declaration.proposer) {
                return Err("the proposing DM is distinct from consenting players".into());
            }
            let record = Record::Consented { participant:access.participant.0, proposal:key.clone(),
                event:*downtime, roster:roster.clone() };
            proposal.consents.push(Consent { participant:access.participant.0, roster:roster.clone(),
                witness:append(&mut candidate, &record)? });
            bindings.record_downtime(key, proposal);
            "individual consent recorded; activation awaits roster/time policy".into()
        },
        TimeIntent::SimOff | TimeIntent::SimOn => return Err("simulation-mode changes use the separate founding/history boundary".into()),
    };
    bindings.validate(&candidate)?;
    let game = bindings.encode()?;
    *session = candidate;
    Ok((accepted, game))
}

/// The caller identity of the one typed change admitted beside native history.
/// Transport/seat admission remains the host's responsibility, not an authored
/// history label or a discovery pass over arbitrary assertions.
pub struct Change { pub participant: Id, pub seat: Seat }

/// Verify the dedicated time publication, including its actual accepted event.
/// Until 851/852 settle activation, this boundary admits records at the same
/// native tick only. It never infers completion from history text or names.
pub fn validate_transition(before: &Session, before_game: &[u8], after: &Session, after_game: &[u8])
    -> Result<Change, String> {
    let old = before.save();
    let next = after.save();
    if old.genesis != next.genesis || old.branch != next.branch || old.movement != next.movement
        || old.tick != next.tick || old.checkpoints != next.checkpoints
        || next.entries.len() != old.entries.len() + 1 || !next.entries.starts_with(&old.entries)
    { return Err("native time result does not append one record at its actual current tick".into()); }
    let entry = next.entries.last().ok_or("native time result has no new entry")?;
    let Command::Assert(Assertion::History(line)) = &entry.command else {
        return Err("native time result does not append its typed history record".into());
    };
    if line.key != format!("vtt:time:{}:{}", old.entries.len(), old.tick)
        || entry.tick != old.tick || entry.order != old.entries.len() as u64 {
        return Err("native time record has another append position or tick".into());
    }
    let record: Record = serde_json::from_str(&line.text).map_err(|error| error.to_string())?;
    if before.sim.state().events.contains_key(&line.event_key()) {
        return Err("native time result reuses a pre-existing event".into());
    }
    let a = GameBindings::decode(before_game)?;
    let b = GameBindings::decode(after_game)?;
    a.validate(before)?;
    b.validate(after)?;
    if !a.same_except_time(&b) { return Err("native time result changes character, placement, control or payload".into()); }
    let witness = Witness { entry:entry.id.clone(), event_key:line.event_key(),
        event:EventHandle(key_handle(&line.event_key())), tick:entry.tick };
    witness.validate(after, &record)?;
    let (key, expected, change) = match record {
        Record::Proposed(declaration) => {
            let key = witness.event_key.clone();
            if a.downtime().contains_key(&key) || b.downtime().len() != a.downtime().len() + 1 {
                return Err("native downtime proposal replaces a prior identity".into());
            }
            let change = Change { participant:declaration.proposer, seat:Seat::Dm };
            (key, Proposal { declaration, witness, consents:Vec::new(), completed:None }, change)
        },
        Record::Consented { participant, proposal, event, roster } => {
            let mut expected = a.downtime().get(&proposal).ok_or("consent has no prior accepted proposal")?.clone();
            if expected.witness.event != event || expected.completed.is_some()
                || b.downtime().len() != a.downtime().len() {
                return Err("consent changes proposal identity or an already completed proposal".into());
            }
            expected.consents.push(Consent { participant, roster, witness });
            (proposal, expected, Change { participant, seat:Seat::Player })
        },
    };
    if b.downtime().get(&key) != Some(&expected) || a.downtime().iter().any(|(prior, proposal)|
        prior != &key && b.downtime().get(prior) != Some(proposal)) {
        return Err("native time result rewrites or drops earlier proposal/consent records".into());
    }
    Ok(change)
}
