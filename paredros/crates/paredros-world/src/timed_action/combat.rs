// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl TimedActionSession {
    /// Release paid limb contributions against one world-owned target, atomically.
    /// Invalid contact input preserves both preparation and world state. An
    /// uncharged release clears preparation without recording a world intent.
    pub fn release_against(
        &mut self,
        target: SubjectId,
        action_tick: Tick,
        rules: crate::CombatRules,
    ) -> Result<Vec<GameEvent>, TimedActionError> {
        let mut candidate = self.clone();
        let strikes = candidate.release_inner(action_tick)?;
        if strikes.is_empty() {
            *self = candidate;
            return Ok(Vec::new());
        }
        let actor = candidate.session.control().played();
        let tick = candidate.session.game().next_tick();
        let events = candidate.session.apply_game(GameIntent::ResolveVolley {
            tick,
            actor,
            target,
            strikes,
            rules,
        })?;
        *self = candidate;
        Ok(events)
    }
}
