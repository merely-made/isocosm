// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit party opt-in and disclosure over the caller's one native Session.

use cosmograph::{Command, Session, party::{Change, SharedMap}, schema::Id};
use cosmograph_overlay::{Intent, OrdinaryAccess, Seat, key_for_handle,
    vtt::{PartyIntent, PartyAction, VttIntent, VttIntentEnvelope}};

mod statements;
pub use statements::{StatementInput, KnowledgeReading, statement_admission, submit_statement_source,
    LocationInput, submit_location, read_knowledge, detail_keys, NOTE_KINDS};

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
        PartyAction::Disclose { sites, events } | PartyAction::DiscloseDetails { sites, events, .. } => {
            let native_sites: std::collections::BTreeSet<_> = sites.iter().map(|site| site.0).collect();
            let native_events = events.iter().map(|event|
                key_for_handle(session.sim.state().events.keys().map(String::as_str), event.0)
                    .map(str::to_owned).ok_or("disclosed event handle has no unique native key".into()))
                .collect::<Result<std::collections::BTreeSet<_>, String>>()?;
            if native_sites.len() != sites.len() || native_events.len() != events.len() {
                return Err("duplicate native site/event in party disclosure".into());
            }
            if let PartyAction::DiscloseDetails { held, .. } = &input.action {
                let selected = held.iter().cloned().collect::<std::collections::BTreeSet<_>>();
                if selected.len() != held.len() { return Err("duplicate named held disclosure source".into()); }
                Change::DiscloseDetails { participant, party, member, sites: native_sites, events: native_events, held: selected }
            } else { Change::Disclose { participant, party, member, sites: native_sites, events: native_events } }
        },
        PartyAction::ObserveStatement { source_note } => Change::ObserveStatement { participant, member, source_note: source_note.clone() },
        PartyAction::Tell { hearer, held_note } => Change::Tell { participant, member, hearer: hearer.0, held_note: held_note.clone() },
        PartyAction::ObserveTerrain { name, site, min, max } => {
            let bindings = super::GameBindings::decode(&game)?;
            let observation = terrain_observation(session, &bindings, &game, member,
                cosmograph::knowing::terrain::NamedWindow { name: name.clone(), site: site.0, min: *min, max: *max })?;
            Change::ObserveTerrain { participant, member, observation }
        },
    };
    let mut candidate = session.clone();
    let accepted = candidate.command(Command::Party(change))?;
    *session = candidate;
    Ok((accepted, game))
}

/// The request supplies only its named target. Actual saved geometry supplies sight.
fn terrain_observation(session: &Session, bindings: &super::GameBindings, game: &[u8], member: Id,
    named: cosmograph::knowing::terrain::NamedWindow) -> Result<cosmograph::knowing::terrain::Observation, String> {
    bindings.validate(session)?;
    let mut characters = bindings.characters().iter().filter(|(_, character)| character.entity == member);
    let (key, _) = characters.next().ok_or("terrain observer has no admitted native character")?;
    if characters.next().is_some() { return Err("terrain observer character binding is ambiguous".into()); }
    let position = bindings.positions().get(key).ok_or("terrain observer has no admitted physical position")?;
    let (eye, range) = position.current_eye(session)?.ok_or("terrain observation needs the actual supplied eye and range")?;
    if position.placement.site != named.site { return Err("terrain target is outside the actual observer site".into()); }
    let native = session.sim.state().population.get(member).ok_or("terrain observer is absent")?;
    let window = &position.placement.window;
    Ok(cosmograph::knowing::terrain::Observation { named,
        eye: cosmograph::knowing::terrain::Eye { window: cosmograph::operations::Window {
            min: window.min, max: window.max, depth: window.depth }, places: position.placement.places,
            eye, range, patch: position.placement.patch, body_revision: native.body_revision,
            resident_since: position.resident_since.ok_or("terrain eye has no current native visit")?,
            terrain_revision: session.sim.terrain_revision(), position_entry: position.patch_entry.clone(),
            game_revision: cosmograph::digest(&game) } })
}

/// Shared map facts remain separate from the player's survival cut and event stream.
pub fn read_party(session: &Session, access: OrdinaryAccess, party: &str) -> Result<SharedMap, String> {
    if !session.sim.state().population.get(access.participant.0).is_some_and(cosmograph::directing::is_participant)
        || session.sim.plays(access.participant.0).map(cosmograph_overlay::EntityHandle) != access.subject {
        return Err("party map caller has stale native participant or subject".into());
    }
    let map = cosmograph::party::shared_map(session, party)?;
    if access.seat == Seat::Player && !map.members.iter().any(|m| m.participant == access.participant.0) {
        return Err("player is not opted into this shared party map".into());
    }
    Ok(map)
}

/// A deliberate share request; the caller still signs/stamps and submits it normally.
pub fn disclose_known(session: &Session, participant: Id, party: String) -> Result<PartyIntent, String> {
    let member = session.sim.plays(participant).ok_or("map disclosure needs the currently played body")?;
    let known = cosmograph::party::knowledge(&session.sim, member)?;
    Ok(PartyIntent { party, member: cosmograph_overlay::EntityHandle(member),
        action: PartyAction::Disclose { sites: known.sites.into_iter().map(cosmograph_overlay::PlaceHandle).collect(),
            events: known.events.iter().map(|key| cosmograph_overlay::EventHandle(cosmograph_overlay::key_handle(key))).collect() } })
}
