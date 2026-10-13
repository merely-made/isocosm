// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;
use crate::native_table::{NativeTable, NativeTableAdmissionInput, NativeTableBindingInput, NativeTableResolved, NativeTokenMove,
    TableSender, table_digest};
use isocosm::Execution;
use isocosm_overlay::{EntityHandle, Intent, IntentEnvelope, OrdinaryAccess, ParticipantHandle, Seat, Tick,
    vtt::{TableAct, TableActKind, TableBatch, VttIntent}};

fn preserve_time(old: Option<&NativeTable>, next: &NativeTable, carry_config:bool) -> Result<(), String> {
    let before = isometry_campaign::native::GameBindings::decode(
        old.map_or(&[][..], |table| table.game.as_slice()))?;
    let after = isometry_campaign::native::GameBindings::decode(&next.game)?;
    if before.downtime() != after.downtime() || before.time_holds() != after.time_holds()
        || before.shared_players() != after.shared_players() || before.journeys()!=after.journeys()
        || before.hooks()!=after.hooks() || before.catalogue()!=after.catalogue()
        || before.physical_edits()!=after.physical_edits() || before.item_records()!=after.item_records() {
        return Err("table authoring/movement cannot change native time, journey, hook, pack or physical edit records".into());
    }
    if before.project_carry().iter().any(|(key,policy)|after.project_carry().get(key)!=Some(policy))
        || !carry_config && before.project_carry()!=after.project_carry() {
        return Err("table movement/binding cannot replace its saved project carry policies".into());
    }
    if before.composed_storylets() != after.composed_storylets() {
        return Err("table movement/binding cannot replace its accepted storylet composition records".into());
    }
    if after.roster_revision() != roster_revision(old, next)? {
        return Err("table result differs from its actual accepted player-roster revision".into());
    }
    Ok(())
}

fn roster_revision(old: Option<&NativeTable>, next: &NativeTable) -> Result<u64, String> {
    let Some(old) = old else { return Ok(0); };
    let revision = isometry_campaign::native::GameBindings::decode(&old.game)?.roster_revision();
    if old.bound_players() == next.bound_players() { Ok(revision) }
    else { revision.checked_add(1).ok_or("native time roster revision overflow".into()) }
}

fn validate_binding_change(state: &GameSnapshot, session: &isocosm::Session,
    old: Option<&NativeTable>, next: &NativeTable) -> Result<(), String> {
    if let Some(old) = old {
        let a = isometry_campaign::native::GameBindings::decode(&old.game)?;
        let b = isometry_campaign::native::GameBindings::decode(&next.game)?;
        if old.boards == next.boards && a.characters() == b.characters()
            && a.positions() == b.positions() && a.controls() == b.controls() {
            // Only metadata changes: departed/stale poses remain history.
            // This grants no new placement or renewed physical availability.
            return next.validate_history(session);
        }
    }
    next.validate(state, session)
}

pub fn resolve_native_move(state: &GameSnapshot, sender: TableSender, token: TokenId, to: TileCoord)
    -> Result<NativeTableResolved, String> {
    let world = state.native.as_ref().ok_or("table has no native campaign")?;
    let table = state.native_table.as_ref().ok_or("native table needs explicit character, peer and cell bindings")?;
    let campaign = world.load(Execution::Grouped)?;
    table.validate(state, campaign.table_session())?;
    let board = table.active(state)?;
    let character = board.tokens.iter().find(|held| held.token == token)
        .ok_or("token has no explicit native character binding")?;
    let destination = board.cells.iter().find(|cell| cell.tile == to)
        .ok_or("destination tile has no authored native cell")?;
    let game = isometry_campaign::native::GameBindings::decode(&table.game)?;
    let actor = game.characters().get(&character.character).ok_or("native recipe binding is absent")?.entity;
    let participant = table.participant(sender)?;
    let session = campaign.table_session();
    let access = OrdinaryAccess { participant: ParticipantHandle(participant),
        seat: if matches!(sender, TableSender::Host) { Seat::Dm } else { Seat::Player },
        subject: session.sim.plays(participant).map(EntityHandle) };
    let envelope = IntentEnvelope { participant: access.participant, tick: Tick(session.sim.state().tick),
        intent: Intent::Game(VttIntent::Table(TableBatch { acts: vec![TableAct {
            actor: EntityHandle(actor), kind: TableActKind::Move { to: destination.to },
        }] })) };
    let bytes = serde_json::to_vec(&envelope).map_err(|error| error.to_string())?;
    let (admitted, game, _) = campaign.with_table_input(table.game.clone(), access, &bytes)?;
    let mut next = table.clone();
    next.game = game;
    Ok(NativeTableResolved { expected_table: table_digest(Some(table)),
        publication: NativePublication::update(world, &admitted)?, table: next,
        moves: vec![NativeTokenMove { map: board.map.clone(), token,
            from: state.map.token(token).ok_or("native table token is absent")?.at, to }] })
}

pub(crate) fn apply_native_table(state: &mut GameSnapshot, result: &NativeTableResolved) -> Result<(), GameError> {
    let fail = |reason: &str| GameError::Native(reason.into());
    if table_digest(state.native_table.as_ref()) != result.expected_table {
        return Err(fail("native table result has stale binding metadata"));
    }
    preserve_time(state.native_table.as_ref(), &result.table, result.moves.is_empty()).map_err(GameError::Native)?;
    if !result.moves.is_empty() {
        let old = state.native_table.as_ref().ok_or_else(|| fail("native move has no original table binding"))?;
        if old.host_participant != result.table.host_participant || old.peers != result.table.peers
            || old.boards != result.table.boards || old.version != result.table.version
        { return Err(fail("native movement changes its authored bindings")); }
    }
    let campaign = result.publication.world.load(Execution::Grouped).map_err(GameError::Native)?;
    let mut next = state.clone();
    super::native::apply_publication(&mut next, &result.publication)?;
    let mut moved = std::collections::BTreeSet::new();
    for movement in &result.moves {
        if next.active_map.as_deref() != Some(movement.map.as_str()) || !moved.insert(movement.token)
            || next.map.token(movement.token).map(|token| token.at) != Some(movement.from)
        { return Err(fail("native table move names a stale map or token")); }
        apply(&mut next.map, &isometry_core::SessionEvent::TokenMoved { id: movement.token, to: movement.to })
            .map_err(GameError::Core)?;
    }
    if let Some(active) = &next.active_map {
        next.maps.get_mut(active).ok_or_else(|| fail("active native board is absent"))?.document = next.map.clone();
    }
    if result.moves.is_empty() {
        validate_binding_change(&next, campaign.table_session(), state.native_table.as_ref(), &result.table)
            .map_err(GameError::Native)?;
    } else {
        result.table.validate(&next, campaign.table_session()).map_err(GameError::Native)?;
    }
    next.native_table = Some(result.table.clone());
    *state = next;
    Ok(())
}

impl HostSession {
    pub fn authenticated_endpoints(&self) -> Vec<[u8; 32]> {
        let mut endpoints: Vec<_> = self.peer_endpoints.values().copied().collect();
        endpoints.sort();
        endpoints.dedup();
        endpoints
    }

    pub fn commit_native_table_admission(&mut self, input: &NativeTableAdmissionInput) -> Result<Vec<Outbound>, String> {
        let world = self.state.native.as_ref().ok_or("no native campaign")?;
        if world.cursor()? != input.expected_native || table_digest(self.state.native_table.as_ref()) != input.expected_table {
            return Err("table admission input names a stale native or table prefix".into());
        }
        let mut campaign = world.load(Execution::Grouped)?;
        let participant = match input.participant {
            isometry_campaign::native::ParticipantInput::Existing(id) => id,
            isometry_campaign::native::ParticipantInput::Join => {
                campaign = campaign.with_commands(&[isocosm::Command::Join], 0).map_err(|error| error.to_string())?;
                campaign.table_session().entries.last().ok_or("native join entry is absent")?.outcome
                    .strip_prefix("participant:").ok_or("native join returned no participant")?
                    .parse().map_err(|_| "native join returned an invalid participant")?
            },
        };
        let session = campaign.table_session();
        let access = OrdinaryAccess { participant: ParticipantHandle(participant), seat: Seat::Dm,
            subject: session.sim.plays(participant).map(EntityHandle) };
        let game = self.state.native_table.as_ref().map(|table| table.game.clone()).unwrap_or_default();
        let bytes = serde_json::to_vec(&input.batch).map_err(|error| error.to_string())?;
        let (admitted, game, _) = campaign.with_table_input(game, access, &bytes)?;
        let mut table = self.state.native_table.clone().unwrap_or(NativeTable {
            version: 1, game: Vec::new(), host_participant: participant, peers: vec![], boards: vec![],
        });
        table.game = game;
        preserve_time(self.state.native_table.as_ref(), &table, input.batch.project_carry.is_some())?;
        // Existing authored mappings never silently follow a changed pose.
        validate_binding_change(&self.state, admitted.table_session(), self.state.native_table.as_ref(), &table)?;
        let result = NativeTableResolved { expected_table: input.expected_table.clone(),
            publication: NativePublication::update(world, &admitted)?, table, moves: vec![] };
        self.try_commit(GameEvent::NativeTableResolved(result))
    }

    /// Trusted transport supplies the actual authenticated endpoint, not Hello.
    /// A routing-hash collision must never inherit another endpoint's grant.
    pub fn on_authenticated_connect(&mut self, peer: PeerId, endpoint: [u8; 32]) -> Result<Vec<Outbound>, String> {
        self.begin_authenticated_connection(peer,endpoint).map(|(_,out)| out)
    }

    pub fn commit_native_table_binding(&mut self, input: &NativeTableBindingInput) -> Result<Vec<Outbound>, String> {
        let world = self.state.native.as_ref().ok_or("no native campaign")?;
        if world.cursor()? != input.expected_native || table_digest(self.state.native_table.as_ref()) != input.expected_table {
            return Err("table binding input names a stale native or table prefix".into());
        }
        let campaign = world.load(Execution::Grouped)?;
        validate_binding_change(&self.state, campaign.table_session(), self.state.native_table.as_ref(), &input.table)?;
        let mut table = input.table.clone();
        let prior = self.state.native_table.as_ref().map(|old|
            isometry_campaign::native::GameBindings::decode(&old.game).map(|game| game.roster_revision()))
            .transpose()?.unwrap_or(0);
        let mut game = isometry_campaign::native::GameBindings::decode(&table.game)?;
        if game.roster_revision() != prior { return Err("binding input rewrites its accepted roster revision".into()); }
        let revision = roster_revision(self.state.native_table.as_ref(), &table)?;
        if revision != prior {
            game.record_roster_revision(revision);
            table.game = game.encode()?;
        }
        preserve_time(self.state.native_table.as_ref(), &table, false)?;
        let result = NativeTableResolved { expected_table: input.expected_table.clone(),
            publication: NativePublication::update(world, &campaign)?, table, moves: vec![] };
        self.try_commit(GameEvent::NativeTableResolved(result))
    }

    pub(crate) fn native_move(&mut self, sender: TableSender, token: TokenId, to: TileCoord)
        -> Result<Vec<Outbound>, String> {
        let result = resolve_native_move(&self.state, sender, token, to)?;
        self.try_commit(GameEvent::NativeTableResolved(result))
    }
}
