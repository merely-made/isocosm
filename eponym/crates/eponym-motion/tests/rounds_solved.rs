// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A borrowed round's minute re-solved with the real solver (case 1117).

use cosmograph::{Execution, Session};
use eponym_play::MotionInput;
use eponym_play::contract::mode::actions::{Action, Run};
use eponym_play::fixtures::native_clock::native_clock_in_rounds;

#[test]
fn a_solved_minute_walks_and_replays() {
    let mut c = native_clock_in_rounds();
    c.solver = eponym_motion::SOLVER;
    let from = c.binding().pose.cell().unwrap();
    let runs = vec![Run { input: MotionInput { move_x: 32767, move_z: 0 }, steps: 1800 },
        Run { input: MotionInput { move_x: 0, move_z: 0 }, steps: 1800 }];
    let end = c.local(&runs);
    let started = std::time::Instant::now();
    c.act(Action::Steps { runs, end, falls: Vec::new() }).expect("the minute's motion");
    let solved = started.elapsed();
    let started = std::time::Instant::now();
    c.minute().expect("the minute");
    let ticked = started.elapsed();
    let to = c.binding().pose.cell().unwrap();
    assert!(to[0] > from[0], "the keeper walked east: {from:?} to {to:?}");
    let restored = Session::load(c.native.save(), Execution::Individuals).expect("replays");
    assert_eq!(restored.sim.state_hash(), c.native.sim.state_hash());
    c.mode().validate_native(&restored).expect("the clock's records replay");
    eprintln!("a solved minute's motion action took {solved:?}, its advance {ticked:?}; {from:?} to {to:?}");
}

/// Case 1120: the standalone game records a walked minute as one intent,
/// stepped locally by `GameState::predict`, and replays it with the solver.
#[test]
fn a_standalone_minute_is_one_intent_and_replays() {
    use eponym_play::fixtures::session::{advance_minute, timed_action_world_in_rounds_solving};
    use eponym_play::{GameIntent, MotionRun};
    let f = timed_action_world_in_rounds_solving(eponym_motion::SOLVER);
    let mut session = f.action.session().clone();
    let from = session.game().movement().position(f.keeper).unwrap();
    let intents = session.game().next_tick().0;
    let started = std::time::Instant::now();
    advance_minute(&mut session, f.keeper, vec![
        MotionRun { input: MotionInput { move_x: 32767, move_z: 0 }, steps: 1800 },
        MotionRun { input: MotionInput { move_x: 0, move_z: 0 }, steps: 1800 }]);
    let stepped = started.elapsed();
    let tick = session.game().next_tick();
    session.apply_game(GameIntent::Minute { tick }).expect("the minute");
    let to = session.game().movement().position(f.keeper).unwrap();
    assert!(to[0] > from[0], "the keeper walked east: {from:?} to {to:?}");
    assert_eq!(session.game().next_tick().0, intents + 2);
    let saved = session.save().unwrap();
    let restored = eponym_play::Session::restore_solving(&saved, eponym_motion::SOLVER).unwrap();
    assert_eq!(restored.state_hash().unwrap(), session.state_hash().unwrap());
    eprintln!("a standalone minute stepped and recorded in {stepped:?}; {from:?} to {to:?}");
}
