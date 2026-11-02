// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The client's round (cases 1096, 1118, 1120, 1121): the played body is
//! stepped and drawn locally, each minute goes to a worker as one motion
//! intent and the minute, the frame adopts the answer, and the save replays.

use cosmograph_host::drive::{Speed, TickMode};
use eponym_client::producer::SceneModel;
use eponym_play::fixtures::session::{motion_rules, timed_action_world_in_rounds_solving};
use eponym_play::{MotionInput, Session};

const EAST: MotionInput = MotionInput { move_x: 32767, move_z: 0 };
const FRAME_US: u64 = 16_667;

#[test]
fn a_client_round_at_one_and_eight_times_replays() {
    for speed in [Speed::X1, Speed::X8] {
        let started = std::time::Instant::now();
        let f = timed_action_world_in_rounds_solving(eponym_motion::SOLVER);
        let keeper = f.keeper;
        let mut model = SceneModel::timed(f.action, keeper);
        let rules = motion_rules(model.game(), keeper);
        assert!(model.start_rounds(speed, TickMode::Worker), "the world was founded in rounds");
        let (ticks, intents) = (model.game().world().sim().state().tick, model.game().next_tick().0);
        let from = model.game().movement().position(keeper).unwrap();
        // Two world minutes of frames, walking east for the first.
        let frames = 2 * 60 * 60 / speed.factor();
        let mut walked_drawn = None;
        for frame in 0..frames {
            let walking = frame < frames / 2;
            model.advance_round(FRAME_US, rules, |_| walking.then_some(EAST)).expect("a frame");
            if frame == frames / 2 - 1 {
                walked_drawn = model.drawn_position(keeper);
            }
            let adopted = model.poll_round().expect("the worker answers");
            assert!(adopted.refusals.is_empty(), "{speed:?}: {:?}", adopted.refusals);
        }
        while model.minute_pending() {
            let adopted = model.drain_round().expect("the worker answers");
            assert!(adopted.refusals.is_empty(), "{speed:?}: {:?}", adopted.refusals);
        }
        let elapsed = started.elapsed();
        let game = model.game();
        assert_eq!(game.world().sim().state().tick, ticks + 2, "{speed:?}: two minutes");
        assert_eq!(game.next_tick().0, intents + 3, "{speed:?}: one motion intent and two minutes");
        let to = game.movement().position(keeper).unwrap();
        assert!(to[0] > from[0], "{speed:?}: the keeper walked east, {from:?} to {to:?}");
        assert_eq!(walked_drawn, Some(to), "{speed:?}: the drawn pose was where the game now has it");
        assert_eq!(model.drawn_position(keeper), Some(to));
        let saved = model.session().save().unwrap();
        let restored = Session::restore_solving(&saved, eponym_motion::SOLVER).unwrap();
        assert_eq!(restored.state_hash().unwrap(), model.session().state_hash().unwrap(), "{speed:?}: replay");
        eprintln!("{speed:?}: two client minutes in {elapsed:?}");
    }
}
