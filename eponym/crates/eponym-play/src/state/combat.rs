// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::identity::{SubjectId, Tick};

use crate::combat::{resolve as resolve_combat, precision::resolve as resolve_precision};
use crate::{COMBAT_RULES_REVISION, DeathCause, GameError, GameEvent, MovementError};

use super::GameState;

impl GameState {
    pub(super) fn resolve_volley(
        &mut self,
        tick: Tick,
        actor: SubjectId,
        target: SubjectId,
        strikes: &[crate::timed_action::StrikeReceipt],
        rules: crate::CombatRules,
    ) -> Result<Vec<GameEvent>, GameError> {
        let actor_record = self.current_anatomy(actor)?;
        let actor_revision = actor_record.revision;
        let actor_document = actor_record.document.clone();
        let actor_at = self
            .movement
            .position(actor)
            .ok_or(MovementError::MissingSubject(actor))?;
        let target_record = self.current_anatomy(target)?;
        let target_revision = target_record.revision;
        let target_document = target_record.document.clone();
        let target_at = self
            .movement
            .position(target)
            .ok_or(MovementError::MissingSubject(target))?;
        let resolution = if rules.revision == COMBAT_RULES_REVISION {
            let actor_pose = self
                .movement
                .pose(actor)
                .ok_or(MovementError::MissingSubject(actor))?;
            let target_pose = self
                .movement
                .pose(target)
                .ok_or(MovementError::MissingSubject(target))?;
            resolve_precision(
                actor,
                actor_revision,
                &actor_document,
                actor_pose,
                target,
                &target_document,
                target_pose,
                self.world.ground(),
                strikes,
                rules,
            )?
        } else {
            resolve_combat(
                actor,
                actor_revision,
                &actor_document,
                actor_at,
                target,
                &target_document,
                target_at,
                self.world.ground(),
                strikes,
                rules,
            )?
        };
        if resolution.harm == 0 {
            return Ok(vec![GameEvent::VolleyResolved {
                tick,
                actor,
                target,
                rules,
                strikes: resolution.strikes,
                harm: 0,
                revision: target_revision,
            }]);
        }
        // The game resolved the blow; the sim takes its wounds (669).
        let (revision, died, reconciled) =
            self.harm(tick, target, resolution.harm, &resolution.severed_parts)?;
        let mut events = vec![GameEvent::VolleyResolved {
            tick,
            actor,
            target,
            rules,
            strikes: resolution.strikes,
            harm: resolution.harm,
            revision,
        }];
        events.extend(reconciled);
        let _ = target_at;
        if died {
            events.push(GameEvent::Died {
                tick,
                subject: target,
                cause: DeathCause::Strike,
            });
        }
        Ok(events)
    }
}
