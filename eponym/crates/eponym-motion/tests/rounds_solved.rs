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
