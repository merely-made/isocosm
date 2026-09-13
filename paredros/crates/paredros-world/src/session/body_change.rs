// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Controlled injury and recovery that preserves the detailed-body boundary.
//!
//! `GameState` still owns every durable transition. This adapter only makes a
//! controlled body's ordinary fall or rest atomic with the necessary anatomy
//! refresh. A fatal fall deliberately keeps its last admitted anatomy as
//! historical evidence: `ReconcileAnatomy` correctly refuses to mutate a dead
//! body.

use paredros_identity::SubjectId;

use crate::{GameEvent, GameIntent};

use super::{Session, SessionError};

impl Session {
    /// Applies the controlled body's ordinary fall transition and, when the
    /// body survives a revision change, refreshes its admitted anatomy at the
    /// same candidate-session boundary.
    pub fn fall(&mut self, distance: i32) -> Result<Vec<GameEvent>, SessionError> {
        self.apply_body_change(|session, subject| GameIntent::Fall {
            tick: session.game.next_tick(),
            subject,
            distance,
        })
    }

    /// Applies the controlled body's ordinary rest transition and refreshes a
    /// changed, surviving detailed body. Dressing consumption remains entirely
    /// in `GameState::Rest`.
    pub fn rest(&mut self) -> Result<Vec<GameEvent>, SessionError> {
        self.apply_body_change(|session, subject| GameIntent::Rest {
            tick: session.game.next_tick(),
            subject,
        })
    }

    fn apply_body_change(
        &mut self,
        make_intent: impl FnOnce(&Session, SubjectId) -> GameIntent,
    ) -> Result<Vec<GameEvent>, SessionError> {
        let mut candidate = self.clone();
        let subject = candidate.control.played();
        let before = candidate.game.current_anatomy(subject)?.revision;
        let intent = make_intent(&candidate, subject);
        let mut events = candidate.apply_game(intent)?;
        let body = candidate
            .game
            .bodies()
            .get(subject)
            .expect("current anatomy proved the controlled body exists");

        if body.alive() && body.revision != before {
            let revision = body.revision;
            events.extend(candidate.apply_game(GameIntent::ReconcileAnatomy {
                tick: candidate.game.next_tick(),
                subject,
                from_revision: before,
                revision,
                severed_parts: Vec::new(),
            })?);
        }
        *self = candidate;
        Ok(events)
    }
}
