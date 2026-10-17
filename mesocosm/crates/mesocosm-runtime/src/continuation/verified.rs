// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Process-local evidence, never decoded from a product section.
use super::*;

pub(crate) struct VerifiedQuestion { continuation: String }
impl VerifiedQuestion {
    /// Called only after the actual native round or full historical validation.
    pub(super) fn after_validation(game: &Game) -> Option<Self> {
        let saved = game.continuation.as_ref()?;
        if saved.checkpoint.is_none() || saved.world_end.is_some() { return None; }
        Some(Self { continuation: isocosm::digest(saved) })
    }

    pub(super) fn matches(&self, game: &Game) -> bool {
        game.continuation.as_ref().is_some_and(|saved|
            self.continuation == isocosm::digest(saved))
    }
}

/// Native load/replay remains mandatory before minting this receipt.
pub(crate) fn validate_replayed_question(session: &Session, game: &Game)
    -> Result<Option<VerifiedQuestion>, String> {
    game.validate_replay(session)?;
    Ok(VerifiedQuestion::after_validation(game))
}
