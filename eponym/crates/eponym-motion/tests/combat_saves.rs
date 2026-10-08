// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A headed save restored with the motion it recorded, which only the game's
//! solver can replay (ruling 597).

use isocosm::legacy::eponym::world::timed_action::{TimedActionSave, TimedActionSession};
use isocosm::legacy::mesocosm::snapshot;

#[test]
fn actual_pre_combat_native_save_restores_and_can_continue() {
    // Actual 2026-09-09 headed smoke output from b9ae9a2, preserved verbatim.
    let bytes = include_bytes!("fixtures/timed-action-v1-game-v3.save");
    let saved: TimedActionSave = snapshot::decode(bytes).unwrap();
    assert_eq!(saved.version, 1);
    assert_eq!(saved.session.game.version, 3);
    let mut restored = TimedActionSession::restore_solving(bytes, eponym_motion::SOLVER).unwrap();
    assert!(restored.action().is_some());
    let tick = restored.action().unwrap().last_tick;
    assert!(!restored.release(tick).unwrap().is_empty());
    let upgraded = restored.save().unwrap();
    let saved: TimedActionSave = snapshot::decode(&upgraded).unwrap();
    assert_eq!(
        saved.session.game.version,
        isocosm::legacy::eponym::world::GAME_STATE_VERSION
    );
    assert_eq!(
        TimedActionSession::restore_solving(&upgraded, eponym_motion::SOLVER).unwrap(),
        restored
    );
}
