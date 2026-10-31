// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use cosmograph::asserted::Assertion;
use cosmograph_overlay::{EntityHandle, ParticipantHandle};

pub struct Change { pub participant: Id, pub activated: Option<Key> }

/// Public replicas verify the exact host-authored input and native successor.
/// Only the already witnessed required private IDs cross, never secret text.
pub fn validate_transition(before: &Session, before_game: &[u8], after: &Session, after_game: &[u8])
    -> Result<Change, String> {
    let (old,next) = (before.save(),after.save());
    if old.genesis != next.genesis || old.branch != next.branch || old.movement != next.movement
        || old.tick != next.tick || old.checkpoints != next.checkpoints
        || !next.entries.starts_with(&old.entries) || next.entries.len() <= old.entries.len() {
        return Err("adventure result changes founding facts, clock or accepted native prefix".into());
    }
    let entry = next.entries.last().ok_or("pack result has no new native record")?;
    let Command::Assert(Assertion::History(line)) = &entry.command else {
        return Err("pack result has no accepted typed native decision record".into());
    };
    let a = GameBindings::decode(before_game)?;
    let b = GameBindings::decode(after_game)?;
    a.validate(before)?;
    b.validate(after)?;
    if !a.same_except_catalogue(&b) { return Err("pack result changes unrelated game, control, roster or view data".into()); }
    let record: Record = serde_json::from_str(&line.text).map_err(|e| e.to_string())?;
    let (participant,intent,private,activated) = match record {
        Record::Content { participant,content } => (participant,PackInput::Content { content },BTreeSet::new(),None),
        Record::Registered { participant,definition } => (participant,PackInput::Register { definition },BTreeSet::new(),None),
        Record::Previewed { participant,key,prepared } => (participant,PackInput::Preview { key,force:prepared.forced },
            prepared.private_requirements,None),
        Record::Activated { participant,key,preview,forced } => {
            let selected = a.catalogue().and_then(|c| c.packs.get(&key)).and_then(|p| p.previews.last())
                .ok_or("activation has no previous accepted preview")?;
            let private = selected.prepared.private_requirements.clone();
            let intent = if forced { PackInput::Force { key:key.clone(),preview } }
                else { PackInput::Accept { key:key.clone(),preview } };
            (participant,intent,private,Some(key))
        },
    };
    let access = OrdinaryAccess { participant:ParticipantHandle(participant),seat:Seat::Dm,
        subject:before.sim.plays(participant).map(EntityHandle) };
    let mut candidate = before.clone();
    let (_,game) = submit_pack(&mut candidate,before_game.to_vec(),access,&private,
        &PackEnvelope { participant,tick:old.tick,intent })?;
    if game != after_game || candidate.save() != next {
        return Err("pack result differs from the exact native admission and catalogue bytes".into());
    }
    Ok(Change { participant,activated })
}
