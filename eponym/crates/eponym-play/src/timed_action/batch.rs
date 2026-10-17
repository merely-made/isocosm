// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl TimedActionSession {
    /// Applies injury and reconciliation intents atomically, then repairs the
    /// open action against the accepted anatomy.
    pub fn apply_game_batch(
        &mut self,
        intents: &[GameIntent],
    ) -> Result<Vec<GameEvent>, TimedActionError> {
        if intents.iter().any(|intent| {
            matches!(
                intent,
                GameIntent::ResolveVolley { .. } | GameIntent::ExternalVolley { .. }
            )
        }) {
            return Err(TimedActionError::DirectVolleyForbidden);
        }
        let mut candidate = self.clone();
        let mut events = Vec::new();
        for intent in intents {
            events.extend(candidate.session.apply_game(intent.clone())?);
        }
        candidate.reconcile_action()?;
        *self = candidate;
        Ok(events)
    }

    /// Applies one recorded world-sourced volley atomically and repairs the
    /// open action only after the native result has been accepted.
    pub(crate) fn apply_external_volley(
        &mut self,
        actor: crate::identity::SubjectId,
        actor_revision: crate::identity::BodyRevisionId,
        target: crate::identity::SubjectId,
        strikes: Vec<super::StrikeReceipt>,
        rules: crate::CombatRules,
    ) -> Result<Vec<GameEvent>, TimedActionError> {
        let mut candidate = self.clone();
        let events = candidate.session.apply_external_volley(
            actor,
            actor_revision,
            target,
            strikes,
            rules,
        )?;
        candidate.reconcile_action()?;
        *self = candidate;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_batch_still_rejects_arbitrary_volley_atomically() {
        let mut fixture = crate::fixtures::session::timed_action_world();
        let before = fixture.action.clone();
        let tick = fixture.action.session().game().next_tick();
        let result = fixture
            .action
            .apply_game_batch(&[GameIntent::ResolveVolley {
                tick,
                actor: fixture.keeper,
                target: fixture.target,
                strikes: Vec::new(),
                rules: crate::CombatRules::default(),
            }]);
        assert!(matches!(
            result,
            Err(TimedActionError::DirectVolleyForbidden)
        ));
        assert_eq!(fixture.action, before);
    }

    #[test]
    fn public_batch_refuses_external_volley_and_session_api_refuses_marker() {
        let mut fixture = crate::fixtures::session::timed_action_world();
        let before = fixture.action.clone();
        let event = GameIntent::ExternalVolley {
            tick: fixture.action.session().game().next_tick(),
            actor: crate::fixtures::native_severing::STRIKER,
            actor_revision: crate::identity::BodyRevisionId(0),
            target: fixture.target,
            strikes: Vec::new(),
            rules: crate::CombatRules::default(),
        };
        assert!(matches!(
            fixture.action.apply_game_batch(&[event.clone()]),
            Err(TimedActionError::DirectVolleyForbidden)
        ));
        assert_eq!(fixture.action, before);
        let mut session =
            crate::Session::begin(fixture.action.session().game().clone(), fixture.keeper).unwrap();
        assert!(matches!(
            session.apply_game(event),
            Err(crate::SessionError::ExternalEventForbidden)
        ));
    }

    #[test]
    fn native_practice_volley_survives_session_replay_and_repeat_is_empty() {
        let mut fixture = crate::fixtures::session::timed_action_world_with_practice_striker();
        let events = crate::fixtures::native_severing::injure_played(&mut fixture.action).unwrap();
        assert!(!events.is_empty());
        let bytes = fixture.action.save().unwrap();
        let restored = TimedActionSession::restore(&bytes).unwrap();
        assert_eq!(restored, fixture.action);
        let mut old = fixture.action.save_record().unwrap();
        old.session.game.version = crate::EXTERNAL_VOLLEY_GAME_STATE_VERSION - 1;
        assert!(matches!(TimedActionSession::restore_record(old.clone()),
            Err(TimedActionError::Session(crate::SessionError::Game(crate::GameError::LegacyExternalVolleyIntent)))));
        assert!(matches!(crate::GameState::restore_record(old.session.game),
            Err(crate::GameError::LegacyExternalVolleyIntent)));
        assert!(
            crate::fixtures::native_severing::injure_played(&mut fixture.action)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn external_volley_failure_and_replayed_played_actor_leave_action_unchanged() {
        let mut fixture = crate::fixtures::session::timed_action_world_with_practice_striker();
        let before = fixture.action.clone();
        let result = fixture.action.apply_external_volley(
            fixture.keeper,
            crate::identity::BodyRevisionId(0),
            fixture.target,
            Vec::new(),
            crate::CombatRules::default(),
        );
        assert!(matches!(
            result,
            Err(TimedActionError::Session(
                crate::SessionError::ExternalActorIsPlayed(_)
            ))
        ));
        assert_eq!(fixture.action, before);

        crate::fixtures::native_severing::injure_played(&mut fixture.action).unwrap();
        let mut save = fixture.action.save_record().unwrap();
        for intent in &mut save.session.game.intents {
            if let GameIntent::ExternalVolley { actor, .. } = intent {
                *actor = fixture.keeper;
                break;
            }
        }
        assert!(matches!(
            TimedActionSession::restore_record(save),
            Err(TimedActionError::Session(
                crate::SessionError::ExternalActorIsPlayed(_)
            ))
        ));
    }
}
