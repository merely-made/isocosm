// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The table's explicit native-cell batch (244), with current native control.
//! This admits location only; it resolves no ruleset action, pace or collision.

use super::{GameBindings, placement::place_character};
use isocosm::Session;
use isocosm_overlay::{EntityHandle, Intent, OrdinaryAccess, Seat,
    vtt::{TableActKind, VttIntent, VttIntentEnvelope}};
use isometer_space::places::Places;

/// Recheck the trusted host's cut against current native control. A caller's
/// table owner, character metadata or proposed access never replaces it.
pub(crate) fn current_access(session: &Session, access: OrdinaryAccess) -> Result<(), String> {
    if !session.sim.state().population.get(access.participant.0)
        .is_some_and(isocosm::directing::is_participant)
    { return Err("table caller is not a native participant".into()); }
    if access.seat == Seat::CreativeObserver {
        return Err("creative observation admits no table movement or authoring".into());
    }
    if session.sim.plays(access.participant.0).map(EntityHandle) != access.subject {
        return Err("table access names a stale native subject".into());
    }
    Ok(())
}

/// The trusted host supplies the ordinary seat cut; this function publishes
/// native Patch entries and product positions together after the entire
/// supplied batch accepts. There is no token-to-body or tile-to-cell inference.
pub fn submit_table(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    envelope: &VttIntentEnvelope) -> Result<(String, Vec<u8>), String> {
    current_access(session, access)?;
    if envelope.tick.0 != session.sim.state().tick || envelope.participant != access.participant {
        return Err("table batch names another native participant or tick".into());
    }
    let Intent::Game(VttIntent::Table(batch)) = &envelope.intent else {
        return Err("this play input requires an explicit native table batch".into());
    };
    // Refuse a mixed unsupported batch before any accepted move is published.
    if batch.acts.iter().any(|act| matches!(&act.kind, TableActKind::Act { .. })) {
        return Err("table acts require an explicit ruleset-to-native resolution mapping".into());
    }
    if batch.acts.is_empty() { return Ok(("moved: 0".into(), game)); }
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    let mut candidate = session.clone();
    let mut accepted = Vec::new();
    for (index, act) in batch.acts.iter().enumerate() {
        let actor = act.actor.0;
        match access.seat {
            Seat::Player if candidate.sim.plays(access.participant.0) != Some(actor) => {
                return Err(format!("move {index}: a player may move only the character currently played"));
            },
            Seat::Dm if candidate.sim.state().relations.iter().any(|relation|
                relation.kind == isocosm::directing::PLAYS && relation.object == actor
                    && relation.subject != access.participant.0) =>
            { return Err(format!("move {index}: the DM may move only an unclaimed character")); },
            Seat::CreativeObserver => return Err("creative observation admits no table movement".into()),
            _ => {},
        }
        let (character, recipe) = bindings.characters().iter().find(|(_, recipe)| recipe.entity == actor)
            .ok_or_else(|| format!("move {index}: actor has no admitted character binding"))?;
        let character = character.clone();
        let recipe = recipe.clone();
        let previous = bindings.positions().get(&character)
            .ok_or_else(|| format!("move {index}: character has no explicit native position"))?;
        previous.current(&candidate)?;
        let TableActKind::Move { to } = &act.kind else {
            return Err("unsupported table action".into());
        };
        if to.site.0 != previous.placement.site {
            return Err(format!("move {index}: a table move cannot cross a native site"));
        }
        let mut placement = previous.placement.clone();
        placement.at = to.at.0.map(i64::from);
        let window = &placement.window;
        let volume = candidate.sim.volume(placement.site, window.min, window.max, window.depth)?;
        placement.patch = Places::derive(&volume, placement.places)?.place_at(placement.at)
            .ok_or_else(|| format!("move {index}: destination has no derived native patch"))?;
        let position = match &previous.spatial {
            Some(spatial)=>super::placement::place_spatial_character(&mut candidate,
                &super::placement::CharacterSpatialPlacement {placement,spatial:spatial.clone()},&recipe),
            None=>place_character(&mut candidate,&placement,&recipe),
        }
            .map_err(|reason| format!("move {index} ({character}): {reason}"))?;
        accepted.push(format!("{character} at {:?}",position.placement.at));
        bindings.record_position(character.clone(), position);
    }
    bindings.validate(&candidate)?;
    let next = bindings.encode()?;
    *session = candidate;
    Ok((format!("moved: {}", accepted.join(", ")), next))
}
