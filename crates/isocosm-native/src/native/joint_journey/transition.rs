// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use super::super::{GameBindings,time::Roster};
use cosmograph::asserted::Assertion;
use cosmograph_overlay::{EntityHandle,ParticipantHandle};

pub struct Change { pub participant: Id, pub seat: Seat }

/// Authenticate the finite input beside actual replayed native history. An
/// arbitrary history text or replacement product payload is not an approval.
pub fn validate_transition(before: &Session, before_game: &[u8], after: &Session, after_game: &[u8],
    roster: &Roster) -> Result<Change,String> {
    let (old,next) = (before.save(),after.save());
    if old.genesis != next.genesis || old.branch != next.branch || old.movement != next.movement
        || old.tick != next.tick || old.checkpoints != next.checkpoints
        || !next.entries.starts_with(&old.entries) || next.entries.len() <= old.entries.len() {
        return Err("joint journey changes founding facts, native clock or its accepted prefix".into());
    }
    let entry = next.entries.last().ok_or("joint journey result has no new native record")?;
    let Command::Assert(Assertion::History(line)) = &entry.command else {
        return Err("joint journey result lacks its accepted typed completion/input record".into());
    };
    let record: Record = serde_json::from_str(&line.text).map_err(|e| e.to_string())?;
    let (participant,seat,intent) = match record {
        Record::Proposed(declaration) => (declaration.proposer,Seat::Dm,
            JourneyInput::Propose { party:declaration.party,
                departures:declaration.departures.into_iter().map(|d| d.input).collect() }),
        Record::Approved { participant,event,.. } => (participant,Seat::Player,JourneyInput::Approve { proposal:event }),
        Record::Applied { participant,event,.. } => (participant,Seat::Dm,JourneyInput::Apply { proposal:event }),
    };
    let a = GameBindings::decode(before_game)?;
    let b = GameBindings::decode(after_game)?;
    a.validate(before)?;
    b.validate(after)?;
    if !a.same_except_journeys(&b) { return Err("joint journey result changes unrelated opaque product/time/roster data".into()); }
    let access = OrdinaryAccess { participant:ParticipantHandle(participant),seat,
        subject:before.sim.plays(participant).map(EntityHandle) };
    let mut candidate = before.clone();
    let (_,game) = submit_journey(&mut candidate,before_game.to_vec(),access,roster,
        &JourneyEnvelope { participant,tick:old.tick,intent })?;
    if game != after_game || candidate.save() != next {
        return Err("joint journey result differs from the exact admitted native input/actions".into());
    }
    Ok(Change { participant,seat })
}
