// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

/// Bind exactly one authenticated input and its native tick result. Replay and
/// hashes are already checked by the native World reader before this boundary.
pub fn validate_transition(before: &Session, before_game: &[u8], after: &Session, after_game: &[u8])
    -> Result<Change, String> {
    let old = before.save();
    let next = after.save();
    if old.genesis != next.genesis || old.branch != next.branch || old.movement != next.movement
        || next.entries.len() != old.entries.len() + 1 || !next.entries.starts_with(&old.entries) {
        return Err("native time result does not append exactly one input to its accepted prefix".into());
    }
    let entry = next.entries.last().ok_or("native time result has no new entry")?;
    let Command::Assert(Assertion::History(line)) = &entry.command else {
        return Err("native time result does not append its typed history record".into());
    };
    if line.key != format!("vtt:time:{}:{}", old.entries.len(), old.tick)
        || entry.tick != old.tick || entry.order != old.entries.len() as u64
        || before.sim.state().events.contains_key(&line.event_key()) {
        return Err("native time record has another append position, old event or tick".into());
    }
    let record: Record = serde_json::from_str(&line.text).map_err(|error| error.to_string())?;
    let a = GameBindings::decode(before_game)?;
    let b = GameBindings::decode(after_game)?;
    a.validate(before)?;
    b.validate(after)?;
    if !a.same_except_time(&b) || a.roster_revision() != b.roster_revision() {
        return Err("native time result changes its character, placement, control, payload or roster revision".into());
    }
    let witness = Witness { entry:entry.id.clone(), event_key:line.event_key(),
        event:EventHandle(key_handle(&line.event_key())), tick:entry.tick };
    witness.validate(after, &record)?;
    let mut expected = a.clone();
    let mut tick = old.tick;
    let change = match record {
        Record::Proposed(declaration) => {
            let key = witness.event_key.clone();
            if a.downtime().contains_key(&key) || declaration.tick != old.tick {
                return Err("native downtime proposal replaces a prior identity or tick".into());
            }
            let change = Change { participant:declaration.proposer, seat:Seat::Dm, roster:declaration.roster.clone() };
            expected.record_downtime(key, Proposal { declaration, witness, consents:Vec::new(), completed:None });
            change
        },
        Record::Consented { participant, proposal, event, roster } => {
            let mut held = a.downtime().get(&proposal).ok_or("consent has no prior accepted proposal")?.clone();
            held.require_current(old.tick, &roster)?;
            if held.witness.event != event { return Err("consent names another proposal event".into()); }
            held.consents.push(Consent { participant, roster:roster.clone(), witness });
            expected.record_downtime(proposal, held);
            Change { participant, seat:Seat::Player, roster }
        },
        Record::Held { participant, released, roster } => {
            expected.record_hold(Hold { participant, released, roster:roster.clone(), witness });
            Change { participant, seat:Seat::Player, roster }
        },
        Record::Completed { participant, proposal, event, from, to, roster, allowances } => {
            let mut held = a.downtime().get(&proposal).ok_or("completion has no prior accepted proposal")?.clone();
            held.require_current(old.tick, &roster)?;
            if from != old.tick || held.witness.event != event
                || to != from.checked_add(held.declaration.ticks).ok_or("clock overflow")?
                || ready(before, &held, &roster, a.time_holds())?.as_ref() != Some(&allowances) {
                return Err("DM completion differs from the exact ready original proposal".into());
            }
            held.completed = Some(Completion { from, to, roster:roster.clone(), witness, allowances });
            expected.record_downtime(proposal, held);
            tick = to;
            Change { participant, seat:Seat::Dm, roster }
        },
    };
    if next.tick != tick || expected != b || tick == old.tick && next.checkpoints != old.checkpoints {
        return Err("native time result rewrites records or differs from its actual authorized tick result".into());
    }
    Ok(change)
}
