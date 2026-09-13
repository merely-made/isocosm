// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#[path = "support/combat_fixture.rs"]
mod fixture;

use mesocosm_core::snapshot;
use paredros_identity::{SubjectId, Tick};
use paredros_world::timed_action::{TimedActionError, TimedActionSave, TimedActionSession};
use paredros_world::{CombatRules, GameError, GameIntent, GameState};

fn charged() -> TimedActionSession {
    let (mut action, _) = fixture::prepared();
    let tick = action.action().unwrap().last_tick;
    action.charge(Tick(tick.0 + 1)).unwrap();
    action.charge(Tick(tick.0 + 2)).unwrap();
    action
}

#[test]
fn invalid_target_and_rules_preserve_paid_charge_and_preparation() {
    let mut action = charged();
    let before = action.clone();
    let tick = action.action().unwrap().last_tick;
    assert!(
        action
            .release_against(SubjectId(u64::MAX), tick, CombatRules::default())
            .is_err()
    );
    assert_eq!(action, before);
    let invalid = CombatRules {
        max_reach: 0,
        ..CombatRules::default()
    };
    assert!(
        action
            .release_against(fixture::TARGET, tick, invalid)
            .is_err()
    );
    assert_eq!(action, before);
}

#[test]
fn public_action_batch_cannot_inject_an_unpaid_volley() {
    let mut action = charged();
    let before = action.clone();
    let tick = action.action().unwrap().last_tick;
    let strikes = action.clone().release(tick).unwrap();
    let intent = GameIntent::ResolveVolley {
        tick: action.session().game().next_tick(),
        actor: fixture::ACTOR,
        target: fixture::TARGET,
        strikes,
        rules: CombatRules::default(),
    };
    assert_eq!(
        action.apply_game_batch(&[intent]),
        Err(TimedActionError::DirectVolleyForbidden)
    );
    assert_eq!(action, before);
}

#[test]
fn actual_pre_combat_native_save_restores_and_can_continue() {
    // Actual 2026-09-09 headed smoke output from b9ae9a2, preserved verbatim.
    let bytes = include_bytes!("fixtures/timed-action-v1-game-v3.save");
    let saved: TimedActionSave = snapshot::decode(bytes).unwrap();
    assert_eq!(saved.version, 1);
    assert_eq!(saved.session.game.version, 3);
    let mut restored = TimedActionSession::restore(bytes).unwrap();
    assert!(restored.action().is_some());
    let tick = restored.action().unwrap().last_tick;
    assert!(!restored.release(tick).unwrap().is_empty());
    let upgraded = restored.save().unwrap();
    let saved: TimedActionSave = snapshot::decode(&upgraded).unwrap();
    assert_eq!(saved.session.game.version, 4);
    assert_eq!(TimedActionSession::restore(&upgraded).unwrap(), restored);
}

#[test]
fn legacy_version_cannot_disguise_a_new_combat_intent() {
    let mut action = charged();
    let tick = action.action().unwrap().last_tick;
    action
        .release_against(fixture::TARGET, tick, CombatRules::default())
        .unwrap();
    let mut save = action.session().game().save_record().unwrap();
    assert!(
        save.intents
            .iter()
            .any(|i| matches!(i, GameIntent::ResolveVolley { .. }))
    );
    save.version = 3;
    assert_eq!(
        GameState::restore_record(save),
        Err(GameError::LegacyCombatIntent)
    );
}

#[test]
fn uncharged_release_clears_preparation_without_a_world_transition() {
    let (mut action, _) = fixture::prepared();
    let before = action.session().game().clone();
    let tick = action.action().unwrap().last_tick;
    assert!(
        action
            .release_against(fixture::TARGET, tick, CombatRules::default())
            .unwrap()
            .is_empty()
    );
    assert!(action.action().is_none());
    assert_eq!(action.session().game(), &before);
}

#[test]
fn invalid_recorded_sources_cannot_partially_damage_a_target() {
    let mut action = charged();
    let tick = action.action().unwrap().last_tick;
    let valid = action.release(tick).unwrap();
    assert_eq!(valid.len(), 2);
    let mut stale = valid.clone();
    stale[1].binding.revision.0 += 1;
    let mut empty = valid.clone();
    empty[1].charge = 0;
    for strikes in [vec![valid[0], valid[0]], stale, empty] {
        let mut game = action.session().game().clone();
        let before = game.clone();
        assert!(
            game.apply(GameIntent::ResolveVolley {
                tick: game.next_tick(),
                actor: fixture::ACTOR,
                target: fixture::TARGET,
                strikes,
                rules: CombatRules::default(),
            })
            .is_err()
        );
        assert_eq!(game, before);
    }
}
