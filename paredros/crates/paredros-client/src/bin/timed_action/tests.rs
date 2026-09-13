// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn native_handler_lifecycle_keeps_authoritative_receipt() {
    let mut app = App::new();
    app.save_path = std::env::temp_dir().join("paredros-timed-action-handler-test.save");
    app.smoke_sequence();
    assert!(app.status[0].contains("target anatomy"));
    let _ = std::fs::remove_file(app.save_path);
}

#[test]
fn wrong_direction_is_a_recorded_miss() {
    let mut app = App::new();
    app.action.prepare(Direction::Backward).unwrap();
    let start = app.action.action().unwrap().last_tick;
    for offset in 1..=4 {
        app.action.charge(Tick(start.0 + offset)).unwrap();
    }
    let events = app
        .action
        .release_against(app.target, Tick(start.0 + 4), app.combat_rules)
        .unwrap();
    let strikes = events
        .into_iter()
        .find_map(|event| match event {
            GameEvent::VolleyResolved { strikes, .. } => Some(strikes),
            _ => None,
        })
        .unwrap();
    assert!(
        strikes
            .iter()
            .all(|strike| matches!(strike.outcome, paredros_world::StrikeOutcome::Miss))
    );
}

#[test]
fn native_move_uses_one_step_toward_absolute_goal() {
    let mut app = App::new();
    let subject = app.action.session().control().played();
    let before = app
        .action
        .session()
        .game()
        .movement()
        .position(subject)
        .unwrap();
    app.move_player([1, 0, 0]);
    let after = app
        .action
        .session()
        .game()
        .movement()
        .position(subject)
        .unwrap();
    assert_eq!(after[0], before[0] + 1);
    assert_eq!(after[1..], before[1..]);
}
