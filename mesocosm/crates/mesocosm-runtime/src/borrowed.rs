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
