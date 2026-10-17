// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use std::collections::BTreeMap;

fn order(session: &Session, witness: &Witness) -> Result<u64, String> {
    session.entries.iter().find(|entry| entry.id == witness.entry).map(|entry| entry.order)
        .ok_or("native time witness entry is absent".into())
}

impl Hold {
    fn validate(&self, session: &Session) -> Result<(), String> {
        self.roster.validate(session)?;
        if !self.roster.players.contains(&self.participant) {
            return Err("time hold was not changed by its bound native player".into());
        }
        self.witness.validate(session, &Record::Held { participant:self.participant,
            released:self.released, roster:self.roster.clone() })
    }
}

impl Proposal {
    pub fn require_current(&self, tick: Tick, roster: &Roster) -> Result<(), String> {
        if self.completed.is_some() { return Err("downtime already completed".into()); }
        if self.declaration.tick != tick || &self.declaration.roster != roster {
            return Err("downtime proposal is stale; preserve its records and declare a new proposal".into());
        }
        Ok(())
    }

    fn validate(&self, session: &Session, key: &str, holds: &[Hold]) -> Result<(), String> {
        self.declaration.roster.validate(session)?;
        if key != self.witness.event_key || self.declaration.tick != self.witness.tick
            || self.declaration.roster.players.contains(&self.declaration.proposer)
            || !session.sim.state().population.get(self.declaration.proposer)
                .is_some_and(isocosm::directing::is_participant) {
            return Err("downtime declaration differs from native participant/roster/tick".into());
        }
        self.witness.validate(session, &Record::Proposed(self.declaration.clone()))?;
        let mut prior = order(session, &self.witness)?;
        for consent in &self.consents {
            consent.roster.validate(session)?;
            if !consent.roster.players.contains(&consent.participant)
                || consent.roster.players.contains(&self.declaration.proposer) {
                return Err("consent participant differs from its authenticated player roster".into());
            }
            consent.witness.validate(session, &Record::Consented { participant:consent.participant,
                proposal:self.witness.event_key.clone(), event:self.witness.event, roster:consent.roster.clone() })?;
            let next = order(session, &consent.witness)?;
            if next <= prior { return Err("native consent records are out of order".into()); }
            prior = next;
        }
        if let Some(completion) = &self.completed {
            let completed = order(session, &completion.witness)?;
            let entry = session.entries.iter().find(|entry| entry.id == completion.witness.entry)
                .ok_or("downtime completion entry is absent")?;
            let Command::Assert(Assertion::History(line)) = &entry.command else {
                return Err("downtime completion has no actual native history record".into());
            };
            let Record::Completed { participant, .. } = serde_json::from_str::<Record>(&line.text)
                .map_err(|error| error.to_string())? else { return Err("downtime completion record differs".into()); };
            if completion.from != self.declaration.tick || completion.witness.tick != completion.from
                || completion.to != completion.from.checked_add(self.declaration.ticks).ok_or("clock overflow")?
                || completion.roster != self.declaration.roster || session.sim.state().tick < completion.to
                || completed <= prior || !session.sim.state().population.get(participant)
                    .is_some_and(isocosm::directing::is_participant)
                || completion.roster.players.contains(&participant) {
                return Err("downtime completion differs from its original span/roster".into());
            }
            let expected = allowances(session, self, holds, completed)?
                .ok_or("downtime completion lacks each original player's accepted permission")?;
            if expected != completion.allowances { return Err("downtime completion permissions differ".into()); }
            completion.witness.validate(session, &Record::Completed { participant, proposal:key.into(),
                event:self.witness.event, from:completion.from, to:completion.to,
                roster:completion.roster.clone(), allowances:completion.allowances.clone() })?;
        }
        Ok(())
    }
}

fn allowances(session: &Session, proposal: &Proposal, holds: &[Hold], before: u64)
    -> Result<Option<Vec<Allowance>>, String> {
    let mut allowed = Vec::new();
    for participant in &proposal.declaration.roster.players {
        if let Some(consent) = proposal.consents.iter().find(|consent|
            consent.participant == *participant && consent.roster == proposal.declaration.roster
                && consent.witness.tick == proposal.declaration.tick
                && order(session, &consent.witness).is_ok_and(|order| order < before)) {
            allowed.push(Allowance::Yes { participant:*participant, witness:consent.witness.clone() });
            continue;
        }
        let held = holds.iter().rev().find(|hold| hold.participant == *participant
            && order(session, &hold.witness).is_ok_and(|order| order < before));
        let Some(held) = held.filter(|hold| hold.released) else { return Ok(None); };
        allowed.push(Allowance::Released { participant:*participant, witness:held.witness.clone() });
    }
    Ok(Some(allowed))
}

/// Readiness never selects a proposal or advances native time.
pub fn ready(session: &Session, proposal: &Proposal, roster: &Roster, holds: &[Hold])
    -> Result<Option<Vec<Allowance>>, String> {
    proposal.require_current(session.sim.state().tick, roster)?;
    for hold in holds { hold.validate(session)?; }
    proposal.validate(session, &proposal.witness.event_key, holds)?;
    allowances(session, proposal, holds, u64::MAX)
}

/// Validate recorded history, including old stale consents. Staleness prevents
/// future activation; it never deletes those authentic historical records.
pub fn validate_bindings(session: &Session, proposals: &BTreeMap<Key, Proposal>, holds: &[Hold])
    -> Result<(), String> {
    let mut prior = None;
    for hold in holds {
        hold.validate(session)?;
        let next = order(session, &hold.witness)?;
        if prior.is_some_and(|prior| next <= prior) { return Err("time hold records are out of order".into()); }
        prior = Some(next);
    }
    for (key, proposal) in proposals { proposal.validate(session, key, holds)?; }
    Ok(())
}
