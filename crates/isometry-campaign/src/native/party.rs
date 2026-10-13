// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit party opt-in and disclosure over the caller's one native Session.

use isocosm::{Command, Session, party::{Change, SharedMap}, schema::Id};
use isocosm_overlay::{Intent, OrdinaryAccess, Seat, key_for_handle,
    vtt::{PartyIntent, PartyAction, VttIntent, VttIntentEnvelope}};

/// Native command and product bytes are published by the same existing candidate door.
pub fn submit_party(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    envelope: &VttIntentEnvelope) -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    if envelope.participant != access.participant || envelope.tick.0 != session.sim.state().tick {
        return Err("party request names another authenticated participant or tick".into());
    }
    let Intent::Game(VttIntent::Party(input)) = &envelope.intent else { return Err("party input required".into()); };
    let participant = access.participant.0;
    let member = input.member.0;
    let party = input.party.clone();
    let change = match &input.action {
        PartyAction::Join => Change::Join { participant, party, member },
        PartyAction::Leave => Change::Leave { participant, party, member },
        PartyAction::Disclose { sites, events } => {
            let native_sites: std::collections::BTreeSet<_> = sites.iter().map(|site| site.0).collect();
            let native_events = events.iter().map(|event|
                key_for_handle(session.sim.state().events.keys().map(String::as_str), event.0)
                    .map(str::to_owned).ok_or("disclosed event handle has no unique native key".into()))
                .collect::<Result<std::collections::BTreeSet<_>, String>>()?;
            if native_sites.len() != sites.len() || native_events.len() != events.len() {
                return Err("duplicate native site/event in party disclosure".into());
            }
            Change::Disclose { participant, party, member, sites: native_sites, events: native_events }
        },
    };
    let mut candidate = session.clone();
    let accepted = candidate.command(Command::Party(change))?;
    *session = candidate;
    Ok((accepted, game))
}

/// Shared map facts remain separate from the player's survival cut and event stream.
pub fn read_party(session: &Session, access: OrdinaryAccess, party: &str) -> Result<SharedMap, String> {
    if !session.sim.state().population.get(access.participant.0).is_some_and(isocosm::directing::is_participant)
        || session.sim.plays(access.participant.0).map(isocosm_overlay::EntityHandle) != access.subject {
        return Err("party map caller has stale native participant or subject".into());
    }
    let map = isocosm::party::shared_map(session, party)?;
    if access.seat == Seat::Player && !map.members.iter().any(|m| m.participant == access.participant.0) {
        return Err("player is not opted into this shared party map".into());
    }
    Ok(map)
}

/// A deliberate share request; the caller still signs/stamps and submits it normally.
pub fn disclose_known(session: &Session, participant: Id, party: String) -> Result<PartyIntent, String> {
    let member = session.sim.plays(participant).ok_or("map disclosure needs the currently played body")?;
    let known = isocosm::party::knowledge(&session.sim, member)?;
    Ok(PartyIntent { party, member: isocosm_overlay::EntityHandle(member),
        action: PartyAction::Disclose { sites: known.sites.into_iter().map(isocosm_overlay::PlaceHandle).collect(),
            events: known.events.iter().map(|key| isocosm_overlay::EventHandle(isocosm_overlay::key_handle(key))).collect() } })
}
