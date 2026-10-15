// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The site-grain contract adapter. Native Location IDs are a distinct noun;
//! this profile's PlaceHandle resolves a native Site, as in the VTT adapter.

use std::collections::BTreeSet;
use isocosm::{Session, attention::{self, Region, Target}};
use isocosm_overlay::{AttentionSet, EntityHandle, EventHandle, FactionHandle,
    LineageHandle, ParticipantHandle, PlaceHandle, PlayerAttention, Pointable,
    key_for_handle, key_handle};

pub fn lower(session: &Session, value: &AttentionSet) -> Result<attention::WindowAttention, String> {
    let state = session.sim.state();
    let mut pinned = BTreeSet::new();
    for point in &value.pinned {
        pinned.insert(match point {
            Pointable::Entity(id) => Target::Entity(id.0),
            Pointable::Place(id) => Target::Site(id.0),
            Pointable::Faction(id) => Target::Faction(id.0),
            Pointable::Lineage(id) => Target::Lineage(key_for_handle(
                state.lineages.keys().map(String::as_str), id.0)
                .ok_or("lineage handle is unknown or ambiguous")?.into()),
            Pointable::Event(id) => Target::Event(key_for_handle(
                state.events.keys().map(String::as_str), id.0)
                .ok_or("event handle is unknown or ambiguous")?.into()),
            _ => return Err("unsupported pointable kind in the site-grain adapter".into()),
        });
    }
    Ok(attention::WindowAttention { pinned, examined: value.examined.map(|p| Region::Site(p.0)) })
}

fn raise_target(session: &Session, target: Target) -> Result<Pointable, String> {
    Ok(match target {
        Target::Entity(id) => Pointable::Entity(EntityHandle(id)),
        Target::Site(id) => Pointable::Place(PlaceHandle(id)),
        Target::Location(_) => return Err("native Location requires its place-graph adapter".into()),
        Target::Faction(id) => Pointable::Faction(FactionHandle(id)),
        Target::Lineage(key) => {
            let handle = key_handle(&key);
            if key_for_handle(session.sim.state().lineages.keys().map(String::as_str), handle) != Some(key.as_str()) {
                return Err("lineage handle is ambiguous".into());
            }
            Pointable::Lineage(LineageHandle(handle))
        },
        Target::Event(key) => {
            let handle = key_handle(&key);
            if key_for_handle(session.sim.state().events.keys().map(String::as_str), handle) != Some(key.as_str()) {
                return Err("event handle is ambiguous".into());
            }
            Pointable::Event(EventHandle(handle))
        },
    })
}

pub fn care(session: &Session, participant: ParticipantHandle, profile: &str)
    -> Result<BTreeSet<Pointable>, String> {
    session.sim.profile_care(participant.0, profile)?.into_iter()
        .map(|target| raise_target(session, target)).collect()
}

pub fn effective(session: &Session, participant: ParticipantHandle) -> Result<PlayerAttention, String> {
    let value = session.sim.player_attention(participant.0)?;
    let raise = |values: BTreeSet<Target>| values.into_iter()
        .map(|target| raise_target(session, target)).collect::<Result<BTreeSet<_>, _>>();
    let examined = value.examined.into_iter().map(|region| match region {
        Region::Site(id) => Ok(PlaceHandle(id)),
        Region::Location(_) => Err("native Location requires its place-graph adapter".into()),
    }).collect::<Result<_, String>>()?;
    Ok(PlayerAttention { participant, played: value.played.into_iter().map(EntityHandle).collect(),
        pinned: raise(value.pinned)?, examined, care: raise(value.care)? })
}

pub fn restored(session: &Session, participant: ParticipantHandle,
    handle: isocosm_overlay::WindowHandle, expected_profile: &str) -> Result<AttentionSet, String> {
    let (_, profile, held) = session.sim.attention_window(participant.0, handle.0)?;
    if profile != expected_profile { return Err("saved window belongs to another product mode".into()); }
    let pinned = held.pinned.into_iter().map(|p| raise_target(session, p)).collect::<Result<_, _>>()?;
    let examined = match held.examined {
        Some(Region::Site(site)) => Some(PlaceHandle(site)),
        Some(Region::Location(_)) => return Err("native Location requires its place-graph adapter".into()),
        None => None,
    };
    Ok(AttentionSet { pinned, examined, ..AttentionSet::default() })
}
