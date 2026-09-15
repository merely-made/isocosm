// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Versioned `GameSave` writing and replay-checked restoration.
//!
//! Each archive version names the intent vocabulary it could have recorded,
//! so an older archive is admitted only when it cannot be hiding a newer
//! intent it never had a word for.

use mesocosm_core::snapshot;

use super::GameState;
use crate::{
    COMBAT_GAME_STATE_VERSION, GAME_STATE_VERSION, GameError, GameIntent, GameSave,
    LEGACY_GAME_STATE_VERSION, MOTION_GAME_STATE_VERSION, PROFILE_GAME_STATE_VERSION, World,
};

impl GameState {
    pub fn save_record(&self) -> Result<GameSave, GameError> {
        Ok(GameSave {
            version: GAME_STATE_VERSION,
            world: self.world.save_record(),
            expected_hash: self.state_hash()?,
            intents: self.intents.clone(),
        })
    }
    pub fn save(&self) -> Result<Vec<u8>, GameError> {
        snapshot::encode(&self.save_record()?).map_err(|_| GameError::Encode)
    }
    pub fn restore(bytes: &[u8]) -> Result<Self, GameError> {
        let save: GameSave = snapshot::decode(bytes).map_err(|_| GameError::Decode)?;
        Self::restore_record(save)
    }
    pub fn restore_record(save: GameSave) -> Result<Self, GameError> {
        if !(LEGACY_GAME_STATE_VERSION..=GAME_STATE_VERSION).contains(&save.version) {
            return Err(GameError::VersionDiverged {
                saved: save.version,
                current: GAME_STATE_VERSION,
            });
        }
        if save.version < COMBAT_GAME_STATE_VERSION
            && save
                .intents
                .iter()
                .any(|intent| matches!(intent, GameIntent::ResolveVolley { .. }))
        {
            return Err(GameError::LegacyCombatIntent);
        }
        if save.version < MOTION_GAME_STATE_VERSION
            && save
                .intents
                .iter()
                .any(|intent| matches!(intent, GameIntent::AdvanceMotion { .. }))
        {
            return Err(GameError::LegacyMotionIntent);
        }
        if save.version < PROFILE_GAME_STATE_VERSION
            && save.intents.iter().any(|intent| {
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
            return Err(GameError::LegacyMovementProfileIntent);
        }
        if save.version < GAME_STATE_VERSION
            && save
                .intents
                .iter()
                .any(|intent| matches!(intent, GameIntent::ReviseCanon { .. }))
        {
            return Err(GameError::LegacyCanonIntent);
        }
        let world = World::restore_record(save.world)?;
        let mut state = Self::new(world);
        for intent in save.intents {
            state.apply(intent)?;
        }
        let restored = state.state_hash()?;
        if restored != save.expected_hash {
            return Err(GameError::StateDiverged {
                saved: save.expected_hash,
                restored,
            });
        }
        Ok(state)
    }
}
