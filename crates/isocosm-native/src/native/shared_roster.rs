// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit saved local table players for the shared host (851). A window,
//! attention set, connected peer or played body is not a roster declaration.

use super::GameBindings;
use cosmograph::{Session, schema::Id};
use cosmograph_overlay::{OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedRosterInput {
    pub version: u32,
    pub expected_roster_revision: u64,
    pub players: BTreeSet<Id>,
}

/// Game metadata only: preserve every native body, accepted time record,
/// opaque payload and hold. The host publishes these bytes with the same
/// borrowed Session; this creates no participant or second native owner.
pub(super) fn submit(session: &Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &SharedRosterInput) -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    if access.seat != Seat::Dm { return Err("only the actual DM seat may author the shared table roster".into()); }
    let mut bindings = GameBindings::decode(&game)?;
    bindings.validate(session)?;
    if input.version != 1 || input.expected_roster_revision != bindings.roster_revision() {
        return Err("shared table roster input has another version or a stale accepted revision".into());
    }
    if input.players.contains(&access.participant.0) || input.players.iter().any(|id|
        !session.sim.state().population.get(*id).is_some_and(cosmograph::directing::is_participant)) {
        return Err("shared table players must be declared native participants distinct from this DM".into());
    }
    if bindings.shared_players() == Some(&input.players) {
        return Ok(("shared table player roster unchanged".into(), game));
    }
    bindings.record_shared_players(input.players.clone())?;
    bindings.validate(session)?;
    Ok((format!("shared table player roster accepted at revision {}", bindings.roster_revision()), bindings.encode()?))
}
