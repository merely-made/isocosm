// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A real wound after the sim has ticked keeps the game's anatomy revision
//! with the body (case 716): the sim draws the wounded part by living cells
//! at its tick, and the game bumps the revision for exactly what was cut.

use eponym_play::{GameEvent, GameIntent};
use eponym_play::fixtures::session::{timed_action_world, timed_action_world_in_rounds};

#[test]
fn a_fall_after_a_minute_keeps_the_revision_with_the_body() {
    for rounds in [false, true] {
        let f = if rounds { timed_action_world_in_rounds() } else { timed_action_world() };
        let (mut a, played) = (f.action, f.keeper);
        // The sim ticks: a minute in a round, any intent otherwise.
        let tick = a.session().game().next_tick();
        let advance = if rounds { GameIntent::Minute { tick } } else { GameIntent::Rest { tick, subject: played } };
        a.apply_game_batch(&[advance]).expect("the sim ticks");
        for distance in [5, 9, 14] {
            let mut b = a.clone();
            let before = b.session().game().current_anatomy(played).unwrap().revision;
            let tick = b.session().game().next_tick();
            let events = b.apply_game_batch(&[GameIntent::Fall { tick, subject: played, distance }])
                .unwrap_or_else(|e| panic!("rounds={rounds} fall {distance}: {e:?}"));
            let game = b.session().game();
            let cut = events.iter().any(|e| matches!(e, GameEvent::AnatomyReconciled { .. }));
            let revision = game.anatomies().get(played).unwrap().revision;
            assert_eq!(game.bodies().get(played).unwrap().revision, revision, "rounds={rounds} fall {distance}");
            assert_eq!(revision.0, before.0 + u64::from(cut), "rounds={rounds} fall {distance}: bumped for a cut only");
        }
    }
}

/// A named wound severs its part whatever tick the sim is at, and the game
/// reconciles the cut: the host's debug injury (the session acceptance's
/// `lost-parts`).
#[test]
fn a_named_wound_severs_its_part_after_a_minute() {
    for rounds in [false, true] {
        let f = if rounds { timed_action_world_in_rounds() } else { timed_action_world() };
        let (mut a, played) = (f.action, f.keeper);
        let tick = a.session().game().next_tick();
        let advance = if rounds { GameIntent::Minute { tick } } else { GameIntent::Rest { tick, subject: played } };
        a.apply_game_batch(&[advance]).expect("the sim ticks");
        let before = a.session().game().current_anatomy(played).unwrap().revision;
        let tick = a.session().game().next_tick();
        let part = isometer_core::PartId(1);
        let events = a.apply_game_batch(&[GameIntent::Wound { tick, subject: played, part }])
            .unwrap_or_else(|e| panic!("rounds={rounds}: {e:?}"));
        assert!(events.iter().any(|e| matches!(e, GameEvent::AnatomyReconciled { .. })), "rounds={rounds}: {events:?}");
        let game = a.session().game();
        let record = game.current_anatomy(played).unwrap();
        assert_eq!(record.revision.0, before.0 + 1, "rounds={rounds}");
        assert_eq!(game.bodies().get(played).unwrap().revision, record.revision, "rounds={rounds}");
        assert!(record.document.part(part).unwrap().severed, "rounds={rounds}: part 1 severed");
        let tick = game.next_tick();
        assert!(a.clone().apply_game_batch(&[GameIntent::Wound { tick, subject: played, part }]).is_err(),
            "rounds={rounds}: a severed part cannot be wounded again");
    }
}
