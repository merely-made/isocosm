// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Native time records with unchanged authored table geometry. Activation is
//! held until the roster and final-consent timing policies (851/852) are ruled.

use super::*;
use crate::native_table::{NativeTable, TableSender, table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, vtt::VttIntentEnvelope};
use isometry_campaign::{founding::Campaign, native::time::{Roster, validate_transition}};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTimeResolved {
    pub expected_table: Option<String>,
    pub publication: NativePublication,
    pub table: NativeTable,
}

fn caller(table: &NativeTable, campaign: &Campaign, sender: TableSender,
    envelope: &VttIntentEnvelope) -> Result<OrdinaryAccess, String> {
    let session = campaign.table_session();
    table.validate_history(session)?;
    let participant = table.participant(sender)?;
    if envelope.participant.0 != participant || envelope.tick.0 != session.sim.state().tick {
        return Err("native time request names another authenticated participant or tick".into());
    }
    Ok(OrdinaryAccess { participant:ParticipantHandle(participant),
        seat:if matches!(sender, TableSender::Host) { Seat::Dm } else { Seat::Player },
        subject:session.sim.plays(participant).map(EntityHandle) })
}

pub(crate) fn apply_native_time(state: &mut GameSnapshot, result: &NativeTimeResolved) -> Result<(), GameError> {
    let fail = |reason: &str| GameError::Native(reason.into());
    let old = state.native_table.as_ref().ok_or_else(|| fail("native time has no admitted table"))?;
    if table_digest(Some(old)) != result.expected_table || old.host_participant != result.table.host_participant
        || old.peers != result.table.peers || old.boards != result.table.boards || old.version != result.table.version
        || !result.publication.boards.is_empty() || !result.publication.inventories.is_empty()
        || result.publication.activate.is_some() {
        return Err(fail("native time changes or replaces authored table mappings"));
    }
    let previous = state.native.as_ref().ok_or_else(|| fail("native time has no campaign"))?;
    let before = previous.load(Execution::Grouped).map_err(GameError::Native)?;
    let after = result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    old.validate_history(before.table_session()).map_err(GameError::Native)?;
    result.table.validate_history(after.table_session()).map_err(GameError::Native)?;
    let change = validate_transition(before.table_session(), &old.game,
        after.table_session(), &result.table.game).map_err(GameError::Native)?;
    match change.seat {
        Seat::Dm if change.participant == old.host_participant => {},
        Seat::Player if change.participant != old.host_participant
            && old.peers.iter().any(|peer| peer.participant == change.participant) => {},
        _ => return Err(fail("native time record has no admitted DM or player identity")),
    }
    let mut next = state.clone();
    super::native::apply_publication(&mut next, &result.publication)?;
    // Retain authentic poses/control as history. Their current() readers may
    // withdraw them after native effects; time never guesses replacement cells.
    next.native_table = Some(result.table.clone());
    *state = next;
    Ok(())
}

impl HostSession {
    pub(crate) fn native_time(&mut self, sender: TableSender, envelope: &VttIntentEnvelope)
        -> Result<Vec<Outbound>, String> {
        let world = self.state.native.as_ref().ok_or("no native campaign")?;
        let table = self.state.native_table.as_ref().ok_or("native time needs an admitted table")?;
        caller(table, &world.load(Execution::Grouped)?, sender, envelope)?;
        Err("native downtime activation awaits roster and final-consent timing rulings 851/852".into())
    }

    /// The trusted host supplies its roster independently of request bytes.
    /// This records proposals/individual yeses only; it cannot advance yet.
    pub fn commit_native_time(&mut self, sender: TableSender, envelope: &VttIntentEnvelope, roster: &Roster)
        -> Result<Vec<Outbound>, String> {
        let world = self.state.native.as_ref().ok_or("no native campaign")?;
        let table = self.state.native_table.as_ref().ok_or("native time needs an admitted table")?;
        let campaign = world.load(Execution::Grouped)?;
        let access = caller(table, &campaign, sender, envelope)?;
        if roster.players.contains(&table.host_participant) || roster.players.iter().any(|participant|
            !table.peers.iter().any(|peer| peer.participant == *participant)) {
            return Err("downtime player roster differs from admitted native player identities".into());
        }
        let (admitted, game, _) = campaign.with_table_time(table.game.clone(), access, roster, envelope)?;
        let mut next = table.clone();
        next.game = game;
        let result = NativeTimeResolved { expected_table:table_digest(Some(table)),
            publication:NativePublication::update(world, &admitted)?, table:next };
        self.try_commit(GameEvent::NativeTimeResolved(result))
    }
}
