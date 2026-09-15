// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn treatment_preserves_severance_and_paid_surviving_action_after_resume() {
    let mut app = App::new();
    app.take_dressing();
    let subject = app.action.session().control().played();
    let dressings = app
        .action
        .session()
        .game()
        .items()
        .carried_by(subject)
        .filter(|item| item.kind == ItemKind::Dressing)
        .count();
    assert_eq!(dressings, 1);
    app.action.prepare(Direction::Right).unwrap();
    app.action.join(isometer_core::PartId(2)).unwrap();
    let tick = app.action.action().unwrap().last_tick;
    app.action.charge(Tick(tick.0 + 1)).unwrap();
    app.injure();
    let before = app.action.action().unwrap().contributors[&isometer_core::PartId(2)].charge;
    assert!(before > 0);
    app.rest();
    let game = app.action.session().game();
    assert_eq!(game.bodies().get(subject).unwrap().wound, 0);
    assert_eq!(
        game.items()
            .carried_by(subject)
            .filter(|item| item.kind == ItemKind::Dressing)
            .count(),
        0
    );
    let anatomy = game.current_anatomy(subject).unwrap();
    assert!(
        anatomy
            .document
            .part(isometer_core::PartId(1))
            .unwrap()
            .severed
    );
    let action = app.action.action().unwrap();
    assert_eq!(
        action.contributors[&isometer_core::PartId(1)].state,
        paredros_world::timed_action::ContributionState::Cancelled
    );
    assert_eq!(
        action.contributors[&isometer_core::PartId(2)].charge,
        before
    );
    assert_eq!(
        action.contributors[&isometer_core::PartId(2)]
            .binding
            .revision,
        anatomy.revision
    );
    let mut restored = TimedActionSession::restore(&app.action.save().unwrap()).unwrap();
    assert_eq!(restored, app.action);
    let next = Tick(action.last_tick.0 + 1);
    assert_eq!(
        restored.charge(next).unwrap(),
        app.action.charge(next).unwrap()
    );
    assert_eq!(restored, app.action);
}

#[test]
fn ordinary_fall_refreshes_charged_bindings_and_death_cancels_them() {
    let mut app = App::new();
    app.action.prepare(Direction::Right).unwrap();
    let tick = app.action.action().unwrap().last_tick;
    app.action.charge(Tick(tick.0 + 1)).unwrap();
    app.action.fall(5).unwrap();
    let subject = app.action.session().control().played();
    let revision = app
        .action
        .session()
        .game()
        .current_anatomy(subject)
        .unwrap()
        .revision;
    assert!(
        app.action
            .action()
            .unwrap()
            .contributors
            .values()
            .all(|c| c.binding.revision == revision)
    );
    app.action.fall(100).unwrap();
    assert!(
        app.action
            .action()
            .unwrap()
            .contributors
            .values()
            .all(|c| c.state == paredros_world::timed_action::ContributionState::Cancelled)
    );
    let before = app.action.clone();
    assert!(app.action.rest().is_err());
    assert_eq!(app.action, before);
}

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
fn native_fractional_motion_resumes_exactly() {
    let mut app = App::new();
    let subject = app.action.session().control().played();
    let before = app
        .action
        .session()
        .game()
        .movement()
        .pose(subject)
        .unwrap();
    app.move_player([1, 0, 0]);
    let after = app
        .action
        .session()
        .game()
        .movement()
        .pose(subject)
        .unwrap();
    assert_eq!(after.step, 1);
    assert!(after.position[0] > before.position[0]);
    assert!(after.position[0] - before.position[0] < paredros_world::MOTION_SCALE);
    let mut resumed = App::new();
    resumed.action = TimedActionSession::restore(&app.action.save().unwrap()).unwrap();
    app.move_player([1, 0, 0]);
    resumed.move_player([1, 0, 0]);
    assert_eq!(app.action, resumed.action);
}

#[test]
fn native_support_loss_slows_movement_and_is_visible_after_resume() {
    use super::view::AppView;
    let mut app = App::new();
    let subject = app.action.session().control().played();
    let before = app
        .action
        .session()
        .game()
        .movement_projection(subject)
        .unwrap()
        .unwrap();
    assert_eq!(before.active_supports.len(), 4);
    app.injure();
    app.action = TimedActionSession::restore(&app.action.save().unwrap()).unwrap();
    let after = app
        .action
        .session()
        .game()
        .movement_projection(subject)
        .unwrap()
        .unwrap();
    assert_eq!(after.active_supports.len(), 3);
    assert_eq!(after.speed * 4, before.speed * 3);
    assert_eq!(after.envelope, before.envelope);
    assert!(
        app.display_lines()
            .iter()
            .any(|line| line.contains("supports 3/4"))
    );
    app.move_player([1, 0, 0]);
    assert_eq!(app.action.session().game().pose(subject).unwrap().step, 1);
}
