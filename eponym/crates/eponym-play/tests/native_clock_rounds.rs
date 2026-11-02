// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The borrowed mode's clock in rounds (cases 1095, 1107, 1108), driven
//! through the real action envelope with the keeper's own access and view.

use cosmograph::{Command, Execution, Session};
use eponym_play::MotionInput;
use eponym_play::fixtures::native_clock::native_clock_in_rounds;

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
