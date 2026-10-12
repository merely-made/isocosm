// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored material-world startup for the actual document host (835/836).
//! The historical demonstration stays its own unchanged founding profile.

use super::*;
use eponym_play::{GameIntent, GameState, MaterialProfile, MotionRules, Session, World, WorldConfig};
use eponym_play::glyphs::GlyphRules;
use eponym_play::timed_action::{TimedActionRules, TimedActionSession};
use serde::Deserialize;
use wing_functions::NetworkSnapshot;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    version: u32,
    seed: u64,
    config: WorldConfig,
    materials: MaterialProfile,
    /// Existing admitted game vocabulary, with authored seeds/positions.
    initial_intents: Vec<GameIntent>,
    played: SubjectId,
    target: SubjectId,
    target_item: ItemId,
    combat_rules: CombatRules,
    network: NetworkSnapshot,
    timed_rules: TimedActionRules,
    motion_rules: MotionRules,
    glyphs: Option<GlyphRules>,
}

pub(super) struct Start {
    pub model: SceneHandle,
    pub target: SubjectId,
    pub target_item: ItemId,
    pub combat_rules: CombatRules,
    pub glyphs: Option<GlyphReading>,
}

pub(super) fn start() -> Result<Start, String> {
    let Some(path) = std::env::var_os("EPONYM_MATERIAL_WORLD") else {
        let fixture = session_fixture::timed_action_world_solving(eponym_motion::SOLVER);
        let played = fixture.keeper;
        let model = SceneModel::timed(fixture.action, played).into_handle();
        let glyphs = match GlyphReading::new(session_fixture::demonstration_rules(played), model.borrow().game()) {
            Ok(reading) => Some(reading),
            Err(why) => { eprintln!("session: acquisition journal disabled: {why}"); None },
        };
        return Ok(Start { model, target:fixture.target, target_item:fixture.target_item,
            combat_rules:fixture.combat_rules, glyphs });
    };
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let input: Input = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if input.version != 1 { return Err("unsupported Eponym material-world startup version".into()); }
    let world = World::generate_materials(input.seed, input.config, input.materials)
        .map_err(|error| format!("material founding: {error:?}"))?;
    let mut game = GameState::new(world).with_motion_solver(eponym_motion::SOLVER);
    for (index, intent) in input.initial_intents.into_iter().enumerate() {
        game.apply(intent).map_err(|error| format!("initial intent {index}: {error:?}"))?;
    }
    if !game.bodies().get(input.target).is_some_and(|body| body.alive())
        || game.items().get(input.target_item).is_none()
    { return Err("startup target or target item is absent from admitted state".into()); }
    input.network.validate().map_err(|error| format!("startup network: {error:?}"))?;
    let glyphs = input.glyphs.map(|rules| GlyphReading::new(rules, &game)).transpose()
        .map_err(|error| format!("startup journal: {error}"))?;
    // The caller-selected living subject crosses the ordinary native
    // Session control door; recipe/material metadata grants no control.
    let session = Session::begin(game, input.played).map_err(|error| format!("startup control: {error:?}"))?;
    let action = TimedActionSession::begin(session, input.network.network, input.timed_rules)
        .map_err(|error| format!("startup action rules: {error:?}"))?;
    let mut model = SceneModel::timed(action, input.played);
    model.motion_rules = Some(input.motion_rules);
    Ok(Start { model:model.into_handle(), target:input.target, target_item:input.target_item,
        combat_rules:input.combat_rules, glyphs })
}
