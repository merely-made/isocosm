// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Individual native downtime, persistent self-released holds, and explicit
//! DM selection (246,851,852,855,856). Native history owns every accepted input.

use super::GameBindings;
use isocosm::{Command, Session, asserted::{Assertion, HistoryLine}, schema::{Id, Key, Tick}};
use isocosm_overlay::{EventHandle, Intent, OrdinaryAccess, Seat,
    key_for_handle, key_handle, vtt::{TimeIntent, VttIntent, VttIntentEnvelope}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod validation;
pub use validation::{ready, validate_bindings};
mod transition;
pub use transition::validate_transition;

/// The trusted host supplies this authenticated roster, never request bytes or
/// player names. Its preimage identifies the actual declared roster source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Roster {
    pub players: BTreeSet<Id>, pub preimage: String,
    #[serde(default, skip_serializing_if="zero")]
    pub revision: u64,
}

fn zero(value: &u64) -> bool { *value == 0 }

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
pub struct Completion {
    pub from: Tick, pub to: Tick, pub roster: Roster, pub witness: Witness,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub allowances: Vec<Allowance>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Allowance {
    Yes { participant:Id, witness:Witness },
    Released { participant:Id, witness:Witness },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hold { pub participant:Id, pub released:bool, pub roster:Roster, pub witness:Witness }

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
    Held { participant:Id, released:bool, roster:Roster },
    Completed { participant:Id, proposal:Key, event:EventHandle, from:Tick, to:Tick,
        roster:Roster, allowances:Vec<Allowance> },
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

/// The host supplies actual access and the accepted saved-player roster.
/// Only the DM's explicit Advance input selects a ready span to run.
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
    if roster.revision != bindings.roster_revision() || roster.preimage != isocosm::digest(&roster.players) {
        return Err("native time roster differs from its accepted revision or player-ID preimage".into());
    }
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
            format!("downtime proposed: {key} ({handle})")
        },
        TimeIntent::Consent { downtime } => {
            if access.seat != Seat::Player || !roster.players.contains(&access.participant.0) {
                return Err("only that authenticated native player may give its yes".into());
            }
            let key = key_for_handle(candidate.sim.state().events.keys().map(String::as_str), downtime.0)
                .ok_or("downtime handle has no unique actual native event")?.to_owned();
            let mut proposal = bindings.downtime().get(&key).ok_or("event is not a recorded downtime proposal")?.clone();
            if proposal.completed.is_some() { return Err("downtime already completed".into()); }
            proposal.require_current(envelope.tick.0, roster)?;
            if roster.players.contains(&proposal.declaration.proposer) {
                return Err("the proposing DM is distinct from consenting players".into());
            }
            let record = Record::Consented { participant:access.participant.0, proposal:key.clone(),
                event:*downtime, roster:roster.clone() };
            proposal.consents.push(Consent { participant:access.participant.0, roster:roster.clone(),
                witness:append(&mut candidate, &record)? });
            bindings.record_downtime(key, proposal);
            "individual consent recorded".into()
        },
        TimeIntent::ReleaseHold | TimeIntent::TakeHold => {
            if access.seat != Seat::Player || !roster.players.contains(&access.participant.0) {
                return Err("only a bound player may change their own time hold".into());
            }
            let released = matches!(intent, TimeIntent::ReleaseHold);
            let record = Record::Held { participant:access.participant.0, released, roster:roster.clone() };
            bindings.record_hold(Hold { participant:access.participant.0, released, roster:roster.clone(),
                witness:append(&mut candidate, &record)? });
            (if released { "your hold is released for current and future proposals" }
                else { "your hold is taken back; accepted yeses remain" }).into()
        },
        TimeIntent::Advance { downtime } => {
            if access.seat != Seat::Dm || roster.players.contains(&access.participant.0) {
                return Err("only the distinct DM may choose a ready downtime".into());
            }
            let key = key_for_handle(candidate.sim.state().events.keys().map(String::as_str), downtime.0)
                .ok_or("downtime handle has no unique actual native event")?.to_owned();
            let mut proposal = bindings.downtime().get(&key).ok_or("event is not a recorded downtime proposal")?.clone();
            let from = candidate.sim.state().tick;
            proposal.require_current(from, roster)?;
            let allowances = ready(&candidate, &proposal, roster, bindings.time_holds())?
                .ok_or("each bound player must give their own yes or release their hold")?;
            let to = from.checked_add(proposal.declaration.ticks).ok_or("clock overflow")?;
            let record = Record::Completed { participant:access.participant.0, proposal:key.clone(),
                event:*downtime, from, to, roster:roster.clone(), allowances:allowances.clone() };
            let witness = append(&mut candidate, &record)?;
            candidate.advance(proposal.declaration.ticks)?;
            proposal.completed = Some(Completion { from, to, roster:roster.clone(), witness, allowances });
            bindings.record_downtime(key, proposal);
            format!("native downtime advanced from {from} to {to}")
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
pub struct Change { pub participant: Id, pub seat: Seat, pub roster: Roster }
