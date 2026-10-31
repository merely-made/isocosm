// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Explicit authored source admission and this actual body's held readings.

use cosmograph::{Session, knowing::{statements, terrain}, party::Details, schema::{Id, Key}};
use cosmograph_overlay::{EntityHandle, OrdinaryAccess, Pointable, Seat, vtt::{Assertion, Fact}};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatementInput {
    pub kind: String, pub version: u32, pub expected_native: Key, pub expected_game: Key,
    pub holder: Id, pub key: Key, pub statement: statements::Statement,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocationInput {
    pub kind: String, pub version: u32, pub expected_native: Key, pub expected_game: Key,
    pub location: cosmograph::schema::Location,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeReading {
    pub holder: Id, pub sources: Vec<(Key, statements::Statement, statements::Source)>,
    pub held: Details, pub locations: BTreeMap<Id, statements::LocationKnowledge>,
    pub native: Key, pub game: Key,
}

/// Old assertion arrays remain the actual author door. This builds no new world.
pub fn statement_admission(session: &Session, game: &[u8], access: OrdinaryAccess,
    input: &StatementInput) -> Result<super::super::AdmissionBatch, String> {
    super::super::movement::current_access(session, access)?;
    if access.seat != Seat::Dm || input.kind != "vtt:statement-source" || input.version != 1
        || input.expected_native != super::super::physical_map::physical_source(session)
        || input.expected_game != cosmograph::digest(&game) {
        return Err("statement authoring requires the actual DM and exact native/game source".into());
    }
    if !session.sim.state().population.get(input.holder).is_some()
        || !session.sim.genesis().rules.note_kinds.contains(statements::SOURCE) {
        return Err("statement source needs its existing native holder and explicitly declared kind".into());
    }
    input.statement.validate(&session.sim)?;
    let fact = input.statement.fact(input.holder, input.key.clone())?;
    Ok(super::super::AdmissionBatch { version: 1,
        assertions: vec![Assertion::Fact(Fact { key: fact.key, kind: statements::SOURCE.into(),
            about: Some(Pointable::Entity(EntityHandle(input.holder))), text: fact.text, tags: Vec::new() })],
        characters: Vec::new(), placements: Vec::new(), controls: Vec::new(), project_carry: None, spatial: Vec::new() })
}

pub fn submit_statement_source(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &StatementInput) -> Result<(String, Vec<u8>), String> {
    let batch = statement_admission(session, &game, access, input)?;
    let bytes = serde_json::to_vec(&batch).map_err(|e| e.to_string())?;
    super::super::submit_game_access(session, game, &bytes, access)
}

/// Native allocates the real identity. Authoring reveals no identity to a body.
pub fn submit_location(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &LocationInput) -> Result<(String, Vec<u8>), String> {
    super::super::movement::current_access(session, access)?;
    if access.seat != Seat::Dm || input.kind != "vtt:location" || input.version != 1
        || input.expected_native != super::super::physical_map::physical_source(session)
        || input.expected_game != cosmograph::digest(&game) {
        return Err("Location authoring requires the actual DM and exact native/game source".into());
    }
    let mut candidate = session.clone();
    let accepted = candidate.command(cosmograph::Command::Location { location: input.location.clone() })?;
    *session = candidate;
    Ok((accepted, game))
}

pub fn read_knowledge(session: &Session, game: &[u8], access: OrdinaryAccess) -> Result<Option<KnowledgeReading>, String> {
    super::super::movement::current_access(session, access)?;
    let Some(holder) = access.subject.map(|body| body.0) else { return Ok(None); };
    Ok(Some(KnowledgeReading { holder, sources: session.sim.statement_sources(holder)?,
        held: Details { statements: session.sim.held_statements(holder)?, terrain: session.sim.held_terrain(holder)? },
        locations: session.sim.known_locations(holder)?,
        native: super::super::physical_map::physical_source(session), game: cosmograph::digest(&game) }))
}

/// Exact source keys for an explicit share; no implicit all-known disclosure.
pub fn detail_keys(held: &Details) -> Result<Vec<Key>, String> { Ok(held.ids()?.into_iter().collect()) }

/// The public kinds a newly authored grammar must explicitly admit.
pub const NOTE_KINDS: [&str; 3] = [statements::SOURCE, statements::HELD, terrain::HELD];
