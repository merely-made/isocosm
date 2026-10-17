// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit standalone table bindings, never inferred from owners or IDs.

use crate::{GameSnapshot, NativePublication};
use isocosm::{Session, schema::{Id, Key}};
use isometry_campaign::CampaignMap;
use isocosm_vtt::{founding::NativeCursor, native::{AdmissionBatch, GameBindings, ParticipantInput}};
use isometry_core::{MapDocument, TileCoord, TokenId};
use isocosm_overlay::vtt::Cell;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeerParticipant { pub endpoint: [u8; 32], pub participant: Id }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenCharacter { pub token: TokenId, pub character: Key }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TileCell { pub tile: TileCoord, pub to: Cell }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoardBinding {
    pub map: String,
    pub ground_digest: String,
    pub tokens: Vec<TokenCharacter>,
    pub cells: Vec<TileCell>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTable {
    pub version: u32,
    /// Existing authenticated GameBindings; no second native Session/save.
    pub game: Vec<u8>,
    pub host_participant: Id,
    pub peers: Vec<PeerParticipant>,
    pub boards: Vec<BoardBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTableBindingInput {
    pub expected_native: NativeCursor,
    pub expected_table: Option<String>,
    pub table: NativeTable,
}

/// Explicit host authoring, including a deliberate Join where requested.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTableAdmissionInput {
    pub expected_native: NativeCursor,
    pub expected_table: Option<String>,
    pub participant: ParticipantInput,
    pub batch: AdmissionBatch,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTokenMove {
    pub map: String, pub token: TokenId, pub from: TileCoord, pub to: TileCoord,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTableResolved {
    pub expected_table: Option<String>,
    pub publication: NativePublication,
    pub table: NativeTable,
    pub moves: Vec<NativeTokenMove>,
}

#[derive(Clone, Copy, Debug)]
pub enum TableSender { Host, Endpoint([u8; 32]) }

pub fn ground_digest(document: &MapDocument) -> String { isocosm::digest(&document.ground) }
pub fn table_digest(table: Option<&NativeTable>) -> Option<String> { table.map(isocosm::digest) }

fn board<'a>(state: &'a GameSnapshot, map: &str) -> Result<&'a MapDocument, String> {
    let held: &CampaignMap = state.maps.get(map).ok_or("table binding names no saved map")?;
    if state.active_map.as_deref() == Some(map) { Ok(&state.map) } else { Ok(&held.document) }
}

impl NativeTable {
    /// Saved bindings, including disconnected players, define the time roster.
    /// Endpoint aliases cannot count the same native player more than once.
    pub fn bound_players(&self) -> BTreeSet<Id> {
        self.peers.iter().map(|peer| peer.participant)
            .filter(|participant| *participant != self.host_participant).collect()
    }

    pub fn time_roster(&self) -> Result<isocosm_vtt::native::time::Roster, String> {
        let players = self.bound_players();
        Ok(isocosm_vtt::native::time::Roster { preimage:isocosm::digest(&players), players,
            revision:GameBindings::decode(&self.game)?.roster_revision() })
    }

    pub fn participant(&self, sender: TableSender) -> Result<Id, String> {
        match sender {
            TableSender::Host => Ok(self.host_participant),
            TableSender::Endpoint(endpoint) => self.peers.iter().find(|peer| peer.endpoint == endpoint)
                .map(|peer| peer.participant).ok_or("authenticated endpoint has no native participant binding".into()),
        }
    }

    /// Stored metadata survives departure or an unknown historical pose. It
    /// authenticates accepted history but supplies no current physical picture.
    pub fn validate_history(&self, session: &Session) -> Result<(), String> {
        if self.version != 1 { return Err("unsupported native table binding version".into()); }
        let participant = |id| session.sim.state().population.get(id)
            .is_some_and(isocosm::directing::is_participant);
        if !participant(self.host_participant) { return Err("table host has no native participant".into()); }
        let mut endpoints = BTreeSet::new();
        for peer in &self.peers {
            if !endpoints.insert(peer.endpoint) || !participant(peer.participant) {
                return Err("duplicate endpoint or absent native table participant".into());
            }
        }
        let game = GameBindings::decode(&self.game)?;
        game.validate(session)?;
        Ok(())
    }

    /// Authentication/replay precedes this live admission check. Only the
    /// active board's current token cells are live native placement claims.
    pub fn validate(&self, state: &GameSnapshot, session: &Session) -> Result<(), String> {
        self.validate_history(session)?;
        let game = GameBindings::decode(&self.game)?;
        let mut names = BTreeSet::new();
        for binding in &self.boards {
            if !names.insert(&binding.map) { return Err("duplicate table board binding".into()); }
            let document = board(state, &binding.map)?;
            if ground_digest(document) != binding.ground_digest {
                return Err("authored tile mapping is stale after board ground changed".into());
            }
            let mut cells = BTreeSet::new();
            for cell in &binding.cells {
                if !cells.insert(cell.tile) || !document.ground.in_bounds(cell.tile.0, cell.tile.1)
                    || !session.sim.state().sites.contains_key(&cell.to.site.0)
                { return Err("duplicate, out-of-board or absent-site native cell mapping".into()); }
            }
            let mut tokens = BTreeSet::new();
            for token in &binding.tokens {
                if !tokens.insert(token.token) { return Err("duplicate token character binding".into()); }
                let held = document.token(token.token).ok_or("table binding names no token")?;
                if !game.characters().contains_key(&token.character) || !game.positions().contains_key(&token.character) {
                    return Err("table token has no explicit admitted character and position".into());
                }
                if state.active_map.as_deref() == Some(binding.map.as_str()) {
                    let position = &game.positions()[&token.character];
                    let to = binding.cells.iter().find(|cell| cell.tile == held.at)
                        .ok_or("current token tile has no authored native cell")?;
                    if to.to.site.0 != position.placement.site || to.to.at.0.map(i64::from) != position.current(session)? {
                        return Err("table token picture differs from current admitted native position".into());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn active(&self, state: &GameSnapshot) -> Result<&BoardBinding, String> {
        let active = state.active_map.as_deref().ok_or("native table movement needs an explicitly named active map")?;
        self.boards.iter().find(|board| board.map == active).ok_or("active map has no explicit native cell bindings".into())
    }
}
