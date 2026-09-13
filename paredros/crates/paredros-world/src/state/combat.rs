// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use paredros_identity::{BodyRevisionId, SubjectId, Tick};

use crate::combat::resolve as resolve_combat;
use crate::{DeathCause, GameError, GameEvent, MovementError};

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
        let resolution = resolve_combat(
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
        )?;
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
        let revision = BodyRevisionId(target_revision.0 + 1);
        let mut anatomies = self.anatomies.clone();
        anatomies.reconcile(target, target_revision, revision, &resolution.severed_parts)?;
        let lost = anatomies
            .get(target)
            .expect("validated candidate anatomy")
            .document
            .parts
            .iter()
            .filter(|part| part.severed)
            .map(|part| part.id)
            .collect::<Vec<_>>();
        let (actual_revision, died) = self
            .bodies
            .get_mut(target)
            .expect("current anatomy proved a live body")
            .injure(resolution.harm, tick);
        debug_assert_eq!(actual_revision, revision);
        self.anatomies = anatomies;
        let released = self.items.release_attached(target, &lost, target_at);
        let mut events = vec![
            GameEvent::VolleyResolved {
                tick,
                actor,
                target,
                rules,
                strikes: resolution.strikes,
                harm: resolution.harm,
                revision,
            },
            GameEvent::AnatomyReconciled {
                tick,
                subject: target,
                from_revision: target_revision,
                revision,
            },
        ];
        events.extend(released.into_iter().map(|item| GameEvent::ItemReleased {
            tick,
            subject: target,
            item,
            at: target_at,
        }));
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
