// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym in rounds (cases 1095, 1097, 1100, 1102): motion runs at sixty
//! steps a real second, `speed` fixed solver steps a frame, the sim ticks
//! only at the minute intent, where poses are handed back, and the save
//! replays to its recorded hash.

use cosmograph_host::drive::{Pace, Pacer, Speed};
use eponym_play::{GameIntent, MotionInput, MotionRun, Session, fixtures::session::{advance_minute, advance_motion, timed_action_world_in_rounds}};

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

/// Case 1120: the same rounds with each minute's motion one intent, stepped
/// locally a frame at a time and recorded at the minute, replaying to its
/// hash; a minute's intents are its motion and the minute itself.
#[test]
fn a_batched_round_at_one_and_eight_times_replays_its_save() {
    const EAST: MotionInput = MotionInput { move_x: 32767, move_z: 0 };
    for speed in [Speed::X1, Speed::X8] {
        let started = std::time::Instant::now();
        let f = timed_action_world_in_rounds();
        let mut session = f.action.session().clone();
        let (ticks, intents) = (session.game().world().sim().state().tick, session.game().next_tick().0);
        let mut pacer = Pacer::new(Pace::Round { speed });
        let (mut held, mut minutes) = (0u32, 0);
        while minutes < 2 {
            let due = pacer.advance(16_667);
            held += (due.steps * speed.factor()) as u32;
            for _ in 0..due.ticks {
                if held > 0 { advance_minute(&mut session, f.keeper, vec![MotionRun { input: EAST, steps: held }]); }
                held = 0;
                let tick = session.game().next_tick();
                session.apply_game(GameIntent::Minute { tick }).expect("the minute");
                minutes += 1;
            }
        }
        assert_eq!(session.game().world().sim().state().tick, ticks + 2, "{speed:?}");
        assert_eq!(session.game().next_tick().0, intents + 4, "{speed:?}: two intents a minute");
        let saved = session.save().unwrap();
        let bytes = serde_json::to_vec(&saved).unwrap().len();
        let restored = Session::restore_solving(&saved, eponym_play::MotionSolver::STILL).unwrap();
        assert_eq!(restored.state_hash().unwrap(), session.state_hash().unwrap(), "{speed:?}: replay");
        eprintln!("{speed:?}: two batched minutes in {:?}, save {bytes} bytes", started.elapsed());
    }
}
