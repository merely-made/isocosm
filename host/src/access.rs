// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::Session;
use isocosm::directing::readings::{self, Mode, View};
use isocosm::schema::Id;
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat};

pub(crate) fn resolve(
    session: &Session,
    participant: ParticipantHandle,
    seat: Seat,
) -> Result<OrdinaryAccess, String> {
    let entity = session.sim.state().population.get(participant.0);
    if !entity.is_some_and(isocosm::directing::is_participant) {
        return Err("window participant is not native".into());
    }
    Ok(OrdinaryAccess {
        participant,
        seat,
        subject: session.sim.plays(participant.0).map(EntityHandle),
    })
}

pub(crate) fn permitted(session: &Session, access: OrdinaryAccess) -> View {
    let subject = access.subject.map_or(Id::MAX, |id| id.0);
    let visibility = match access.seat {
        Seat::Player => Mode::Survival,
        Seat::CreativeObserver | Seat::Dm => Mode::Creative,
    };
    readings::view(&session.sim, subject, visibility)
}

/// Filter ordinary attention publication as well as the native View. The
/// native union remains complete for replay, collection and foreground.
pub(crate) fn filter_attention(session: &Session, access: OrdinaryAccess, visible: &View,
    attention: &mut isocosm_overlay::PlayerAttention) {
    use isocosm_overlay::{Pointable, key_for_handle};
    if access.seat != Seat::Player { return; }
    let state = session.sim.state();
    let shown = |point: &Pointable| match point {
        Pointable::Entity(id) => access.subject == Some(*id) || visible.shows(id.0),
        Pointable::Place(id) => visible.sites.contains(&id.0),
        Pointable::Event(id) => key_for_handle(state.events.keys().map(String::as_str), id.0)
            .is_some_and(|key| visible.events.contains(key)),
        Pointable::Lineage(id) => key_for_handle(state.lineages.keys().map(String::as_str), id.0)
            .is_some_and(|key| visible.things.keys().any(|first|
                state.population.get(*first).is_some_and(|e| e.lineage == key))),
        Pointable::Faction(id) => state.polities.get(&id.0).is_some_and(|p|
            p.constitution.members.iter().any(|member| visible.shows(*member))),
        _ => false,
    };
    attention.pinned.retain(shown);
    attention.care.retain(shown);
    attention.examined.retain(|place| visible.sites.contains(&place.0));
}
