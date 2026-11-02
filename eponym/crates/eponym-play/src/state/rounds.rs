// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A game played in rounds (cases 1095, 1100, 1102): motion is the game's,
//! and at each minute intent every embodied subject's position is handed
//! back to the sim, which ticks once. Nothing is recorded but the intent;
//! replay rebuilds the same handbacks from the same motion.

use super::*;

impl GameState {
    pub(super) fn minute(&mut self) -> Result<Vec<GameEvent>, GameError> {
        if !self.world.rounds() {
            return Err(crate::WorldError::Refused.into());
        }
        let subjects: Vec<_> = self.world.subjects().collect();
        let mut handbacks = Vec::new();
        for (subject, entity) in subjects {
            let alive = self.world.sim().state().population.get(entity).is_some_and(|e| e.alive && e.body.is_some());
            let Some(position) = self.movement.position(subject) else { continue };
            if !alive {
                continue;
            }
            let at = crate::walking::to_site(self.world.volume(), position);
            if let Some(h) = self.world.handback(entity, at) {
                handbacks.push(h);
            }
        }
        self.world.minute(&handbacks)?;
        Ok(Vec::new())
    }

    /// In a round the sim is told a body's patch once, when it first has
    /// none at the site; after that its pose is handed back each minute.
    pub(super) fn patched_in_round(&self, entity: cosmograph::schema::Id) -> bool {
        self.world.rounds() && self.world.sim().state().population.get(entity)
            .and_then(|e| e.patch).is_some_and(|p| p.site == self.world.site())
    }
}
