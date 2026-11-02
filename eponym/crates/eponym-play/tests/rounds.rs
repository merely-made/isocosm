// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym in rounds (cases 1095, 1097, 1100, 1102): motion runs at sixty
//! steps a real second, `speed` fixed solver steps a frame, the sim ticks
//! only at the minute intent, where poses are handed back, and the save
//! replays to its recorded hash.

use cosmograph_host::drive::{Pace, Pacer, Speed};
use eponym_play::{GameIntent, MotionInput, Session, fixtures::session::{advance_motion, timed_action_world_in_rounds}};

/// One real minute of a round at `speed`, a frame at a time: the frame's
/// due steps run as fixed solver steps, and each due minute is a minute
/// intent. Returns the session and how many minutes it recorded.
fn round(speed: Speed, real_us: u64) -> (Session, u64) {
    let f = timed_action_world_in_rounds();
    let mut session = f.action.session().clone();
    let keeper = f.keeper;
    let ticks_before = session.game().world().sim().state().tick;
    let mut pacer = Pacer::new(Pace::Round { speed });
    let mut elapsed = 0;
    let mut minutes = 0;
    while elapsed < real_us {
        let due = pacer.advance(16_667);
        elapsed += 16_667;
        // Case 1097: at speed n, n fixed sixtieth-second steps per frame.
        for _ in 0..due.steps * speed.factor() {
            advance_motion(&mut session, keeper, MotionInput { move_x: 32767, move_z: 0 });
        }
        for _ in 0..due.ticks {
            let tick = session.game().next_tick();
            session.apply_game(GameIntent::Minute { tick }).expect("the minute");
            minutes += 1;
        }
    }
    assert_eq!(session.game().world().sim().state().tick, ticks_before + minutes,
        "the sim ticked only at the minute intents");
    (session, minutes)
}

#[test]
fn a_round_at_one_and_eight_times_replays_its_save() {
    for (speed, real_us) in [(Speed::X1, 61_000_000), (Speed::X8, 8_000_000)] {
        let (session, minutes) = round(speed, real_us);
        assert!(minutes >= 1, "{speed:?}: a minute fell due");
        eprintln!("{speed:?}: {minutes} minutes, {} intents", session.game().next_tick().0);
        let keeper = session.game().world().subjects().next().unwrap().1;
        assert!(session.game().world().sim().state().spatial_poses.contains_key(&keeper),
            "{speed:?}: the minute's handback posed the keeper");
        let saved = session.save().unwrap();
        let restored = Session::restore_solving(&saved, eponym_play::MotionSolver::STILL).unwrap();
        assert_eq!(restored.state_hash().unwrap(), session.state_hash().unwrap(), "{speed:?}: replay");
    }
}
