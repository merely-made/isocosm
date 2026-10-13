// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use isocosm::{asserted::{Assertion, HistoryLine}, projects::OwnedActed};
use isocosm_overlay::{key_handle, key_for_handle};

impl Witness {
    pub(super) fn order(&self, session: &Session) -> Result<u64, String> {
        session.entries.iter().find(|e| e.id == self.entry).map(|e| e.order)
            .ok_or("journey witness entry is absent".into())
    }
    pub(super) fn validate(&self, session: &Session, record: &Record) -> Result<(), String> {
        let entry = session.entries.iter().find(|e| e.id == self.entry).ok_or("journey witness entry is absent")?;
        let Command::Assert(Assertion::History(line)) = &entry.command else {
            return Err("journey witness is not an accepted native history record".into());
        };
        let event = session.sim.state().events.get(&self.event_key).ok_or("journey witness event is absent")?;
        if line.key != format!("vtt:journey:{}:{}", entry.order, self.tick)
            || line.event_key() != self.event_key || entry.outcome != self.event_key
            || key_handle(&self.event_key) != self.event.0
            || key_for_handle(session.sim.state().events.keys().map(String::as_str), self.event.0)
                != Some(self.event_key.as_str())
            || entry.tick != self.tick || event.tick != self.tick || event.authored.as_ref() != Some(line)
            || line.kind != "history" || !line.participants.is_empty() || line.place.is_some()
            || !line.tags.is_empty() || i64::try_from(self.tick).ok() != Some(line.time)
            || line.text != serde_json::to_string(record).map_err(|e| e.to_string())?
            || serde_json::from_str::<Record>(&line.text).map_err(|e| e.to_string())? != *record {
            return Err("journey native entry, event and typed record differ".into());
        }
        Ok(())
    }
}

pub(super) fn append(session: &mut Session, record: &Record) -> Result<Witness, String> {
    let tick = session.sim.state().tick;
    let line = HistoryLine { key:format!("vtt:journey:{}:{tick}", session.entries.len()),
        kind:"history".into(), time:i64::try_from(tick).map_err(|_| "journey tick exceeds history timestamp")?,
        text:serde_json::to_string(record).map_err(|e| e.to_string())?,
        participants:Vec::new(), place:None, tags:BTreeSet::new() };
    let event_key = line.event_key();
    if session.sim.state().events.contains_key(&event_key) { return Err("journey record key already exists".into()); }
    session.command(Command::Assert(Assertion::History(line)))?;
    let witness = Witness { entry:session.entries.last().ok_or("journey record entry is absent")?.id.clone(),
        event:EventHandle(key_handle(&event_key)), event_key, tick };
    witness.validate(session, record)?;
    Ok(witness)
}

impl Proposal {
    pub(super) fn require_current(&self, session: &Session, roster: &super::super::time::Roster) -> Result<(), String> {
        if self.completed.is_some() { return Err("joint journey already applied".into()); }
        if self.declaration.tick != session.sim.state().tick || &self.declaration.roster != roster {
            return Err("joint journey tick or authenticated player roster changed; propose again".into());
        }
        Ok(())
    }
    pub(super) fn validate(&self, session: &Session, key: &str) -> Result<(), String> {
        let declaration = &self.declaration;
        declaration.roster.validate(session)?;
        if key != self.witness.event_key || declaration.tick != self.witness.tick
            || declaration.roster.players.contains(&declaration.proposer)
            || !session.sim.state().population.get(declaration.proposer)
                .is_some_and(isocosm::directing::is_participant) || declaration.departures.is_empty() {
            return Err("joint journey declaration has no distinct native DM, group or original tick".into());
        }
        let mut players = BTreeSet::new();
        let mut bodies = BTreeSet::new();
        for departure in &declaration.departures {
            let input = &departure.input;
            if !players.insert(input.participant) || !bodies.insert(input.body)
                || !declaration.roster.players.contains(&input.participant)
                || departure.journey.from != input.from || departure.journey.to != input.to
                || departure.journey.departed != declaration.tick || departure.journey.arrives <= declaration.tick {
                return Err("joint journey has duplicate identities or a different quoted route/tick".into());
            }
        }
        self.witness.validate(session, &Record::Proposed(declaration.clone()))?;
        let mut previous = self.witness.order(session)?;
        let mut approved = BTreeSet::new();
        for approval in &self.approvals {
            if !approved.insert(approval.participant) || approval.witness.tick != declaration.tick
                || !declaration.departures.iter().any(|d|
                    d.input.participant == approval.participant && d.input.body == approval.body) {
                return Err("journey approval does not name one unique listed participant/body".into());
            }
            approval.witness.validate(session, &Record::Approved { participant:approval.participant,
                body:approval.body, proposal:key.into(), event:self.witness.event })?;
            let order = approval.witness.order(session)?;
            if order <= previous { return Err("journey approvals are not in native record order".into()); }
            previous = order;
        }
        if let Some(done) = &self.completed {
            if approved != players || done.participant != declaration.proposer
                || done.witness.tick != declaration.tick || done.departures.len() != declaration.departures.len() {
                return Err("joint departure lacks the exact listed approvals or original DM".into());
            }
            for (departure, result) in declaration.departures.iter().zip(&done.departures) {
                previous = validate_action(session, departure, result, previous)?;
            }
            if done.witness.order(session)? != previous.checked_add(1).ok_or("journey record order overflow")? {
                return Err("joint departure completion is not immediately after the accepted actions".into());
            }
            done.witness.validate(session, &Record::Applied { participant:done.participant,
                proposal:key.into(), event:self.witness.event, departures:done.departures.clone() })?;
        }
        Ok(())
    }
}

fn validate_action(session: &Session, departure: &Departure, result: &DepartureResult, previous: u64)
    -> Result<u64, String> {
    let input = &departure.input;
    let (participant, body, key) = match result {
        DepartureResult::Departed { participant,body,entry,.. } | DepartureResult::Goal { participant,body,entry } =>
            (*participant,*body,entry),
    };
    let entry = session.entries.iter().find(|e| &e.id == key).ok_or("joint departure native action is absent")?;
    if participant != input.participant || body != input.body || entry.tick != departure.journey.departed
        || entry.order <= previous {
        return Err("joint departure action differs from its listed player/body/order".into());
    }
    match (result, &entry.command) {
        (DepartureResult::Departed { journey,.. }, Command::ActOwned { participant,actor,target,process,cause,selections }) => {
            let acted: OwnedActed = serde_json::from_str(&entry.outcome).map_err(|e| e.to_string())?;
            if departure.kind != DepartureKind::Normative || *participant != input.participant
                || *actor != input.body || target != &input.target
                || process != &input.process || cause.is_some() || *journey != departure.journey
                || !acted.receipt.accepted() || acted.receipt.actor != input.body
                || acted.receipt.chosen != input.process || acted.receipt.target != input.target
                || acted.receipt.tick != entry.tick || selections.len() != input.debits.len()
                || acted.debits != selections.iter().map(|s| s.request.clone()).collect::<Vec<_>>()
                || selections.iter().zip(&input.debits).any(|(s,d)| !d.matches(&s.request)
                    || s.lots != d.lots || s.unowned != d.unowned) {
                return Err("actual native journey action differs from approved process/stock terms".into());
            }
        },
        (DepartureResult::Goal { .. }, Command::Nudge { participant,critter,aim,toward,act })
            if departure.kind == DepartureKind::Goal && *participant == input.participant && *critter == input.body
                && *aim == isocosm::directing::Aim::Act && *toward == isocosm::directing::Toward::Site(input.to)
                && act.as_deref() == Some(input.process.as_str()) && input.debits.is_empty()
                && entry.outcome == "nudged" => {},
        _ => return Err("joint journey result is not its actual native departure or accepted goal".into()),
    }
    Ok(entry.order)
}

pub(super) fn validate(session: &Session, proposals: &BTreeMap<Key, Proposal>) -> Result<(), String> {
    for (key, proposal) in proposals { proposal.validate(session,key)?; }
    Ok(())
}
