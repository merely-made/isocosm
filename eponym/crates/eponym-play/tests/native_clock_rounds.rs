// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The borrowed mode's clock in rounds (cases 1095, 1107, 1108), driven
//! through the real action envelope with the keeper's own access and view.

use cosmograph::{Command, Execution, Session};
use cosmograph_host::drive::{Pace, Pacer, Speed};
use eponym_play::MotionInput;
use eponym_play::contract::mode::actions::{Action, Run};
use eponym_play::fixtures::native_clock::{NativeClock, native_clock_in_rounds};

const EAST: MotionInput = MotionInput { move_x: 32767, move_z: 0 };

/// Plays `minutes` of rounds at `speed` as a host would: each frame's
/// fixed steps are stepped locally and held as runs; at each minute the
/// runs go to the mode as one action, then the minute ticks (cases 1097,
/// 1113, 1117).
fn play(c: &mut NativeClock, speed: Speed, minutes: u64) {
    let mut pacer = Pacer::new(Pace::Round { speed });
    let mut runs: Vec<Run> = Vec::new();
    let mut ticked = 0;
    while ticked < minutes {
        let due = pacer.advance(16_667);
        let n = (due.steps * speed.factor()) as u32;
        match runs.last_mut() {
            Some(run) if run.input == EAST => run.steps += n,
            _ if n > 0 => runs.push(Run { input: EAST, steps: n }),
            _ => {},
        }
        for _ in 0..due.ticks {
            let batch: Vec<Run> = std::mem::take(&mut runs);
            if !batch.is_empty() { c.steps(batch).expect("the minute's motion"); }
            c.minute().expect("the minute");
            ticked += 1;
        }
    }
}

/// One and eight times speed both reach the sim once a minute, and the
/// native save replays to its hash with the clock's records valid.
#[test]
fn a_borrowed_round_at_one_and_eight_times_replays() {
    for speed in [Speed::X1, Speed::X8] {
        let started = std::time::Instant::now();
        let mut c = native_clock_in_rounds();
        let (tick, records, step) = (c.native.sim.state().tick, c.mode().actions.unwrap().records.len(), c.binding().pose.step);
        play(&mut c, speed, 2);
        let mode = c.mode();
        assert_eq!(c.native.sim.state().tick, tick + 2, "{speed:?}: two minutes ticked");
        // One motion and one advance a minute, not 3,600 records.
        assert_eq!(mode.actions.as_ref().unwrap().records.len(), records + 4, "{speed:?}");
        assert_eq!(c.binding().pose.step, step + 2 * 3600, "{speed:?}: every fixed step was solved");
        assert!(c.native.sim.state().spatial_poses.contains_key(&c.entity), "{speed:?}: the handback posed the keeper");
        let restored = Session::load(c.native.save(), Execution::Individuals).expect("replays");
        assert_eq!(restored.sim.state_hash(), c.native.sim.state_hash(), "{speed:?}: replay");
        mode.validate_native(&restored).expect("the clock's records replay");
        eprintln!("{speed:?}: two minutes in {:?}, mode {} bytes", started.elapsed(), c.game.len());
    }
}

/// A minute whose claimed end pose is not where its inputs lead is refused.
#[test]
fn a_minute_ending_elsewhere_is_refused() {
    let mut c = native_clock_in_rounds();
    let runs = vec![Run { input: EAST, steps: 600 }];
    let mut end = c.local(&runs);
    end.step += 1;
    let before = c.game.clone();
    let refused = c.act(Action::Steps { runs, end, falls: Vec::new() }).unwrap_err();
    assert!(refused.contains("claimed pose"), "{refused}");
    assert_eq!(c.game, before);
    let over = vec![Run { input: EAST, steps: 3601 }];
    let end = c.local(&over);
    assert!(c.act(Action::Steps { runs: over, end, falls: Vec::new() }).unwrap_err().contains("3600"));
}

/// A handback the sim refuses leaves the binding where the game had it, the
/// minute still runs, and the refusal is recorded and replays.
#[test]
fn a_refused_handback_leaves_the_binding() {
    let mut c = native_clock_in_rounds();
    c.step(MotionInput { move_x: 32767, move_z: 0 }).expect("a motion step");
    let before = c.binding();
    // Fill the keeper's eye cell, so its pose is no longer clear.
    let volume = c.native.sim.volume(before.site, before.window.min, before.window.max, before.window.depth).unwrap();
    let root = eponym_play::walking::to_site(&volume, before.pose.cell().unwrap());
    let ground = volume.material([root[0], root[1] - 1, root[2]]).unwrap();
    let eye = [root[0], root[1] + 1, root[2]];
    c.native.command(Command::Edit { site: before.site, by: None, edit: isometer_space::Edit {
        op: isometer_space::Op::Fill(ground),
        shape: isometer_space::edit::Shape::Box { min: eye, max: [eye[0] + 1, eye[1] + 1, eye[2] + 1] } } })
        .expect("the eye is filled");
    let tick = c.native.sim.state().tick;
    c.minute().expect("the minute runs");
    assert_eq!(c.native.sim.state().tick, tick + 1);
    let mode = c.mode();
    let record = mode.clock.as_ref().unwrap().records.last().unwrap();
    assert!(matches!(record.handback_reports.as_slice(), [r] if r.actor == c.entity && r.patch.is_err()),
        "the keeper's handback is refused: {:?}", record.handback_reports);
    assert!(record.changes.is_empty(), "a refused handback re-poses no one");
    assert_eq!(c.binding(), before, "the binding stands");
    assert!(!c.native.sim.state().spatial_poses.contains_key(&c.entity), "the sim did not take the pose");
    let restored = Session::load(c.native.save(), Execution::Individuals).expect("replays");
    assert_eq!(restored.sim.state_hash(), c.native.sim.state_hash());
    mode.validate_native(&restored).expect("the clock's records replay");
}
