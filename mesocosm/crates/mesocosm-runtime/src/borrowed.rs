// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Product operations over the host's named native Session; no owned world.

use isocosm::{Session, schema::Id, directing::readings::View};
use isocosm_overlay::{Intent, mesocosm::MesocosmIntentEnvelope};
use crate::{Refusal, runtime::translate::direct};

pub struct Reading<'a> { session: &'a Session, visible: &'a View }

impl<'a> Reading<'a> {
    pub fn new(session: &'a Session, visible: &'a View) -> Self { Self { session, visible } }
    pub fn things(&self) -> Vec<(Id, u64, String)> {
        self.visible.things.iter().filter_map(|(id, count)| {
            self.session.sim.state().population.get(*id)
                .map(|entity| (*id, *count, entity.lineage.clone()))
        }).collect()
    }
    pub fn events(&self) -> Vec<String> { self.visible.events.iter().cloned().collect() }
}

/// The host admits attention through its native window token, separately.
/// Checkpoints require the existing runtime's active question and review.
pub fn submit(session: &mut Session, participant: Id, round: u64,
    envelope: &MesocosmIntentEnvelope) -> Result<String, Refusal> {
    if envelope.participant.0 != participant || envelope.tick.0 != session.sim.state().tick {
        return Err(Refusal::Sim("envelope names another participant or tick".into()));
    }
    let Intent::Game(intent) = &envelope.intent else {
        return Err(Refusal::Unbuilt("attention requires the native window door".into()));
    };
    let mut candidate = session.clone();
    let accepted = direct::apply(&mut candidate, participant, round, intent)?;
    *session = candidate;
    Ok(accepted)
}

/// The shared host resolves operations against its named native Session.
pub fn advance_geometry<R>(session: &mut Session, access: isocosm_overlay::OrdinaryAccess,
    resolver: &mut R)
    -> Result<isocosm::flows::FlowResult<isocosm::simulation::Work>, Refusal>
where R: FnMut(&isocosm::operations::Pending)
    -> Result<isocosm_overlay::mesocosm::TerrainGeometry, String> + ?Sized
{
    crate::terrain::advance(session, access, resolver)
}

/// The caller publishes these native and product candidates together.
pub fn submit_game(session: &mut Session, access: isocosm_overlay::OrdinaryAccess,
    round: u64, input: &crate::spatial::Input, mut game: crate::spatial::Game)
    -> Result<(String, crate::spatial::Game), Refusal> {
    use crate::spatial::Input;
    if access.seat == isocosm_overlay::Seat::CreativeObserver
        || session.sim.plays(access.participant.0) != access.subject.map(|id| id.0)
    { return Err(Refusal::NotPlayed); }
    game.validate(session).map_err(Refusal::Sim)?;
    let mut candidate = session.clone();
    let accepted = match input {
        Input::Graft(input) => crate::runtime::graft::submit(
            &mut candidate, access, &mut game, input).map_err(Refusal::Sim)?,
        Input::Envelope(envelope) => {
            if matches!(&envelope.intent, Intent::Game(intent) if intent.is_dev()) {
                return Err(Refusal::Sim("dev intents require the separate authored-run door".into()));
            }
            submit(&mut candidate, access.participant.0, round, envelope)?
        },
        Input::Game(input) => {
            if access.seat != isocosm_overlay::Seat::Player
                || access.subject.is_none()
                || candidate.sim.plays(access.participant.0) != access.subject.map(|id| id.0)
                || input.participant != access.participant || input.tick.0 != candidate.sim.state().tick
            { return Err(Refusal::NotPlayed); }
            let mut accepted = Vec::new();
            if let Some(envelope) = &input.envelope {
                if matches!(&envelope.intent, Intent::Game(intent) if intent.is_dev()) {
                    return Err(Refusal::Sim("dev geometry needs the separate authored-run door".into()));
                }
                accepted.push(submit(&mut candidate, access.participant.0, round, envelope)?);
            }
            if input.advance {
                game.advance(&mut candidate, access)?;
                accepted.push("advanced 1".into());
            }
            accepted.join("\n")
        },
    };
    game.refresh(&mut candidate).map_err(Refusal::Sim)?;
    *session = candidate;
    Ok((accepted, game))
}
