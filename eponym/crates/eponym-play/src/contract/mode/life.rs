// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Existing-life selection over the authenticated native participant, including
//! after death. This is not a body act and supplies no geometry or new identity.

use super::*;
use std::collections::BTreeSet;
use isocosm::{Command, schema::Method};
use isocosm_overlay::{OrdinaryAccess, Seat};
use isocosm_overlay::eponym::{LifeCheckpoint, LifeChoice, StartTime, Successor};

pub const KIND: &str = "eponym:life";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub kind: String, pub version: u32, pub expected_native: Key, pub expected_game: Key,
    /// An existing admitted product subject, not a number inferred from EntityId.
    pub subject: SubjectId, pub checkpoint: LifeCheckpoint,
    /// The exact latest accepted Take for this participant at a death cut.
    pub previous_take: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub input: Input, pub participant: Id, pub previous_take: Option<Key>,
    pub taken: Key, pub target: Id, pub outcome: String,
}

/// Actual native control evidence, independent of current PLAYS or product labels.
pub fn previous_take(session: &Session, participant: Id) -> Option<&isocosm::history::Entry> {
    session.entries.iter().rev().find(|entry|
        matches!(&entry.command, Command::Take { participant: p, .. } if *p == participant)
            && entry.outcome == "taken")
}

fn target(checkpoint: LifeCheckpoint) -> Result<Id, String> {
    match checkpoint {
        LifeCheckpoint::First(first) => {
            if first.start != StartTime::Society {
                return Err("first-life start time requires its separate authored founding input".into());
            }
            match first.begins_as {
                LifeChoice::Denizen(entity) => Ok(entity.0),
                _ => Err("outsider/birth selection requires its separate admitted founding door".into()),
            }
        },
        LifeCheckpoint::Death(death) => match death.next {
            Successor::Another(LifeChoice::Denizen(entity)) => Ok(entity.0),
            Successor::Companion(entity) => Ok(entity.0),
            _ => Err("a new world, outsider or birth requires its separate admitted founding door".into()),
        },
    }
}

fn shape(input: &Input) -> Result<Id, String> {
    if input.kind != KIND || input.version != 1 || input.expected_native.is_empty() || input.expected_game.is_empty() {
        return Err("unsupported or unbound Eponym life input".into());
    }
    match input.checkpoint {
        LifeCheckpoint::First(_) if input.previous_take.is_some() =>
            return Err("first life cannot name a previous accepted Take".into()),
        LifeCheckpoint::Death(_) if input.previous_take.as_ref().is_none_or(String::is_empty) =>
            return Err("succession needs its exact previous accepted native Take".into()),
        _ => {},
    }
    target(input.checkpoint)
}

/// Only the host's unpublished candidate is changed. Native Take remains the
/// admission door, seeding/preserving its bond without changing method or motives.
pub fn submit(session: &mut Session, bytes: Vec<u8>, input: &Input, access: OrdinaryAccess)
    -> Result<(String, Vec<u8>), String> {
    shape(input)?;
    if input.expected_native != source(session) || input.expected_game != actions::game_source(&bytes) {
        return Err("Eponym life input differs from its exact native/game cut".into());
    }
    let mut mode = ModeState::decode(&bytes)?.ok_or("Eponym life selection needs admitted product identities")?;
    mode.validate_native(session)?;
    let target = admit(session, &mode, input, access)?;
    let participant = access.participant.0;
    let outcome = session.command(Command::Take { participant, critter: target })?;
    if outcome != "taken" || session.sim.plays(participant) != Some(target) {
        return Err("native life selection did not accept the supplied target".into());
    }
    let taken = session.entries.last().ok_or("accepted native Take entry is absent")?.id.clone();
    mode.life.push(Record { input: input.clone(), participant, previous_take: input.previous_take.clone(),
        taken, target, outcome: outcome.clone() });
    if mode.life.len() > session.sim.genesis().rules.limits.history {
        return Err("Eponym life receipts exceed the native history limit".into());
    }
    // Historical poses, policies, paid work and deceased bodies remain intact.
    // Current physical availability is a separate native visit/revision reading.
    mode.version = mode.required_version(); mode.validate()?;
    mode.validate_native(session)?;
    Ok((outcome, serde_json::to_vec(&mode).map_err(|e| e.to_string())?))
}

/// A UI supplies both existing subject and checkpoint; only actual source keys
/// and the caller's accepted predecessor command are filled by this read.
pub fn prepare(session: &Session, bytes: &[u8], subject: SubjectId, checkpoint: LifeCheckpoint,
    access: OrdinaryAccess) -> Result<Input, String> {
    let previous_take = match checkpoint {
        LifeCheckpoint::First(_) => None,
        LifeCheckpoint::Death(_) => previous_take(session, access.participant.0).map(|e| e.id.clone()),
    };
    let input = Input { kind: KIND.into(), version: 1, expected_native: source(session),
        expected_game: actions::game_source(bytes), subject, checkpoint, previous_take };
    let mode = ModeState::decode(bytes)?.ok_or("Eponym life selection needs admitted product identities")?;
    mode.validate_native(session)?;
    admit(session, &mode, &input, access)?;
    Ok(input)
}

/// Current admitted, visible subjects only. Complete native eligibility decides
/// fallback; a hidden or unbound companion never becomes an invented subject.
pub fn death_choices(session: &Session, bytes: &[u8], access: OrdinaryAccess,
    visible: &isocosm::directing::readings::View) -> Result<Vec<Input>, String> {
    let prior = previous_take(session, access.participant.0).ok_or("no native life to succeed")?;
    let Command::Take { critter: predecessor, .. } = &prior.command else { unreachable!() };
    let predecessor = *predecessor;
    let dead = session.sim.state().population.get(predecessor).ok_or("predecessor is absent")?;
    if dead.alive { return Err("death choices require actual native death".into()); }
    let eligible = eligible_companions(session, predecessor)?;
    let mode = ModeState::decode(bytes)?.ok_or("life choices need admitted product bindings")?;
    mode.validate_native(session)?;
    let mut choices = Vec::new();
    for binding in &mode.bindings {
        if !visible.shows(binding.entity) { continue; }
        let entity = isocosm_overlay::EntityHandle(binding.entity);
        let next = if eligible.is_empty() { Successor::Another(LifeChoice::Denizen(entity)) }
            else if eligible.contains(&binding.entity) { Successor::Companion(entity) } else { continue; };
        let checkpoint = LifeCheckpoint::Death(isocosm_overlay::eponym::Succession {
            of: isocosm_overlay::EntityHandle(predecessor), next });
        if let Ok(input) = prepare(session, bytes, binding.subject, checkpoint, access) { choices.push(input); }
    }
    Ok(choices)
}

fn admit(session: &Session, mode: &ModeState, input: &Input, access: OrdinaryAccess) -> Result<Id, String> {
    let target = shape(input)?;
    let participant = access.participant.0;
    if access.seat != Seat::Player || !session.sim.state().population.get(participant)
        .is_some_and(isocosm::directing::is_participant)
        || access.subject.map(|s| s.0) != session.sim.plays(participant) {
        return Err("Eponym life selection needs the actual current native participant".into());
    }
    let binding = mode.bindings.iter().find(|b| b.subject == input.subject)
        .ok_or("selected life has no explicitly admitted Eponym subject binding")?;
    if binding.entity != target { return Err("selected product subject and native life differ".into()); }
    let body = session.sim.state().population.get(target).ok_or("selected native life is absent")?;
    if !body.alive || body.method != Method::Normative {
        return Err("Eponym takes an existing living native sophont; non-sapient critters stay directed".into());
    }
    let previous = previous_take(session, participant);
    match input.checkpoint {
        LifeCheckpoint::First(_) => {
            if previous.is_some() || session.sim.plays(participant).is_some() {
                return Err("this native participant has already taken up a life".into());
            }
        },
        LifeCheckpoint::Death(death) => {
            let previous = previous.ok_or("the participant has no accepted native Take to succeed")?;
            if input.previous_take.as_ref() != Some(&previous.id)
                || !matches!(&previous.command, Command::Take { critter, .. } if *critter == death.of.0)
                || session.sim.plays(participant).is_some_and(|id| id != death.of.0) {
                return Err("death succession does not name this participant's latest native life".into());
            }
            let dead = session.sim.state().population.get(death.of.0)
                .ok_or("the predecessor's native death cannot be established from this cut")?;
            if dead.alive { return Err("the predecessor is still living; ordinary succession is unavailable".into()); }
            let eligible = eligible_companions(session, death.of.0)?;
            match death.next {
                Successor::Companion(entity) if eligible.contains(&entity.0) => {},
                Successor::Another(_) if eligible.is_empty() => {},
                Successor::Companion(_) => return Err("selected successor does not satisfy the world's native social companion predicate".into()),
                _ => return Err("another life requires the actual absence of eligible living companions".into()),
            }
        },
    }
    Ok(target)
}

pub fn submit_bytes(session: &mut Session, bytes: Vec<u8>, input: &[u8], access: OrdinaryAccess)
    -> Result<(String, Vec<u8>), String> {
    let input: Input = serde_json::from_slice(input).map_err(|e| e.to_string())?;
    submit(session, bytes, &input, access)
}

pub fn validate(records: &[Record]) -> Result<(), String> {
    let mut keys = BTreeSet::new();
    for record in records {
        if shape(&record.input)? != record.target || record.previous_take != record.input.previous_take
            || record.outcome != "taken" || record.taken.is_empty() || !keys.insert(&record.taken) {
            return Err("invalid or duplicated Eponym life receipt".into());
        }
    }
    Ok(())
}

/// Links product observations to exact accepted native control commands. Death
/// was checked from the actual source at submission; this is not a new native
/// death command, a reconstructed world, or a claim of historical death replay.
pub fn validate_native(session: &Session, records: &[Record]) -> Result<(), String> {
    validate(records)?;
    let mut last = None;
    for record in records {
        let index = session.entries.iter().position(|e| e.id == record.taken).ok_or("life native Take entry is absent")?;
        let entry = &session.entries[index];
        if last.is_some_and(|i| index <= i) || entry.outcome != record.outcome
            || entry.command != (Command::Take { participant: record.participant, critter: record.target }) {
            return Err("life receipt differs from its ordered accepted native Take".into());
        }
        let previous = session.entries[..index].iter().rev().find(|e|
            matches!(&e.command, Command::Take { participant, .. } if *participant == record.participant)
                && e.outcome == "taken");
        match record.input.checkpoint {
            LifeCheckpoint::First(_) if previous.is_some() =>
                return Err("recorded first life follows an existing native life".into()),
            LifeCheckpoint::Death(death) => {
                let previous = previous.ok_or("recorded succession has no native predecessor Take")?;
                if Some(&previous.id) != record.previous_take.as_ref()
                    || !matches!(&previous.command, Command::Take { critter, .. } if *critter == death.of.0) {
                    return Err("recorded succession names another native predecessor".into());
                }
                let before = session.before_entry(&record.taken)?;
                if source(&before) != record.input.expected_native
                    || before.sim.plays(record.participant).is_some_and(|id| id != death.of.0) {
                    return Err("recorded succession differs from its authenticated native source".into());
                }
                let dead = before.sim.state().population.get(death.of.0)
                    .ok_or("recorded predecessor is absent at its exact source")?;
                if dead.alive { return Err("recorded succession preceded native death".into()); }
                if !before.sim.state().population.get(record.target)
                    .is_some_and(|body|body.alive && body.method==Method::Normative) {
                    return Err("recorded successor was not an existing living native sophont".into());
                }
                let eligible = eligible_companions(&before, death.of.0)?;
                match death.next {
                    Successor::Companion(entity) if eligible.contains(&entity.0) => {},
                    Successor::Another(_) if eligible.is_empty() => {},
                    _ => return Err("recorded successor lacks its source's native social eligibility or absence".into()),
                }
            },
            _ => {},
        }
        last = Some(index);
    }
    if records.len() > session.sim.genesis().rules.limits.history {
        return Err("Eponym life history exceeds the native limit".into());
    }
    Ok(())
}

/// The complete native population establishes absence, including companions
/// that have no current product binding or are outside a player's awareness.
/// Missing authored policy refuses; it cannot unlock another-life fallback.
fn eligible_companions(session: &Session, predecessor: Id) -> Result<BTreeSet<Id>, String> {
    let policy = session.sim.genesis().rules.companions.as_ref()
        .ok_or("succession needs an authored native social companion predicate")?;
    policy.validate(&session.sim.genesis().rules)?;
    let mut eligible = BTreeSet::new();
    for (&first, group) in &session.sim.state().population.groups {
        if !group.entity.alive || group.entity.method != Method::Normative { continue; }
        let end = first.checked_add(group.count).ok_or("companion identity interval overflows")?;
        for candidate in first..end {
            if candidate != predecessor && session.sim.companion(predecessor, candidate)?.eligible {
                eligible.insert(candidate);
            }
        }
    }
    Ok(eligible)
}
