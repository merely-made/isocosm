// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::*;

pub(super) fn validate_game_version(save: &SessionSave) -> Result<(), SessionError> {
    if !(LEGACY_GAME_STATE_VERSION..=GAME_STATE_VERSION).contains(&save.game.version) {
        return Err(SessionError::Game(GameError::VersionDiverged {
            saved: save.game.version,
            current: GAME_STATE_VERSION,
        }));
    }
    if save.game.version < COMBAT_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|intent| matches!(intent, GameIntent::ResolveVolley { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyCombatIntent));
    }
    if save.game.version < EXTERNAL_VOLLEY_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|intent| matches!(intent, GameIntent::ExternalVolley { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyExternalVolleyIntent));
    }
    if save.game.version < MOTION_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|intent| matches!(intent, GameIntent::AdvanceMotion { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyMotionIntent));
    }
    if save.game.version < PROFILE_GAME_STATE_VERSION
        && save.game.intents.iter().any(|intent| {
            matches!(
                intent,
                GameIntent::ConfigureMovementProfile { .. }
                    | GameIntent::AdvanceMotion {
                        rules: crate::MotionRules { revision: 2, .. },
                        ..
                    }
            )
        })
    {
        return Err(SessionError::Game(GameError::LegacyMovementProfileIntent));
    }
    if save.game.version < CANON_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|intent| matches!(intent, GameIntent::ReviseCanon { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyCanonIntent));
    }
    if save.game.version < NATIVE_RECIPE_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|i| matches!(i, GameIntent::GenerateNativeRecipe { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyNativeRecipeIntent));
    }
    if save.game.version < ITEM_PLACEMENT_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|intent| matches!(intent, GameIntent::Drop { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyItemPlacementIntent));
    }
    if save.game.version < PROJECT_GAME_STATE_VERSION
        && save
            .game
            .intents
            .iter()
            .any(|intent| matches!(intent, GameIntent::ProjectAct { .. }))
    {
        return Err(SessionError::Game(GameError::LegacyProjectIntent));
    }
    if save.game.version < PROJECT_CUSTODY_GAME_STATE_VERSION
        && save.game.intents.iter().any(|intent| {
            matches!(
                intent,
                GameIntent::ProjectCarry { .. }
                    | GameIntent::ProjectReserve { .. }
                    | GameIntent::StockOwnership { .. }
                    | GameIntent::ProjectConsent { .. }
            )
        })
    {
        return Err(SessionError::Game(GameError::LegacyProjectCustodyIntent));
    }
    if save.game.version < OWNED_STOCK_GAME_STATE_VERSION
        && save.game.intents.iter().any(|intent| {
            matches!(
                intent,
                GameIntent::StockAct { .. } | GameIntent::EatOwned { .. }
            )
        })
    {
        return Err(SessionError::Game(GameError::LegacyOwnedStockIntent));
    }
    if save.game.version < SPREAD_GAME_STATE_VERSION
        && (save.game.world.generator_version == crate::founding::spread::GENERATOR_VERSION
            || save.game.intents.iter().any(|intent| {
                matches!(
                    intent,
                    GameIntent::FoundSpreadActor { .. }
                        | GameIntent::SpreadOwnership { .. }
                        | GameIntent::AuthoredSpread { .. }
                )
            }))
    {
        return Err(SessionError::Game(GameError::LegacySpreadIntent));
    }
    Ok(())
}
