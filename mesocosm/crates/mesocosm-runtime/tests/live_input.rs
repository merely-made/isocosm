// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Live input through the core, headless (sim-host plan H4): the head's play,
//! checkpoint, board and dev keys and its clicks reach the session through
//! the core's driver and Mesocosm's keymap, a stale pick is refused, and the
//! backlog is capped.

#[path = "runtime_pin/worlds.rs"]
#[allow(dead_code)]
mod worlds;

use cosmograph_overlay::Intent;
use cosmograph_overlay::mesocosm::{CheckpointAnswerKind, DevIntent, MesocosmIntent, NudgeTarget, PlayerActKind};
use mesocosm_runtime::{Button, Context, InputRefusal, Named, Pick, Pointed, Press, Runtime};
use worlds::{held, offered};

fn game(runtime: &Runtime, at: usize) -> MesocosmIntent {
    match &runtime.trace()[at].0.intent { Intent::Game(intent) => intent.clone(), other => panic!("{other:?}") }
}

fn queued_then_applied(runtime: &mut Runtime, n: usize) -> Vec<MesocosmIntent> {
    assert_eq!(runtime.queued_len(), n);
    let from = runtime.trace().len();
    assert_eq!(runtime.step(1), 1);
    (from..runtime.trace().len()).map(|at| game(runtime, at)).collect()
}

#[test]
fn play_keys_reach_the_session_through_the_core() {
    let mut runtime = offered();
    let critter = runtime.critter().unwrap();
    let context = Context::default();
    assert_eq!(runtime.press(&Press::char('E'), &context), Ok(true));
    assert_eq!(runtime.press(&Press::Named(Named::Space), &context), Ok(true));
    assert_eq!(runtime.press(&Press::char('s'), &context), Ok(true));
    assert_eq!(runtime.press(&Press::char('q'), &context), Ok(false), "Q opens the window's menu");
    let applied = queued_then_applied(&mut runtime, 3);
    let site = runtime.played().unwrap().resident_site().unwrap();
    for nudge in &applied[..2] {
        let MesocosmIntent::Nudge(nudge) = nudge else { panic!("{nudge:?}") };
        assert_eq!((nudge.entity.0, nudge.target), (critter, NudgeTarget::Place(cosmograph_overlay::PlaceHandle(site))));
    }
    assert!(matches!(&applied[2], MesocosmIntent::Act(act) if matches!(act.kind, PlayerActKind::Speciate { .. })));
    let focused = Context { focus: Some(critter + 1), ..context };
    runtime.press(&Press::char('e'), &focused).unwrap();
    let MesocosmIntent::Nudge(nudge) = &queued_then_applied(&mut runtime, 1)[0] else { panic!() };
    assert_eq!(nudge.target, NudgeTarget::Thing(cosmograph_overlay::EntityHandle(critter + 1)), "toward the focused thing");
}

#[test]
fn a_question_narrows_the_keys_to_its_answers_and_the_board_commits() {
    let mut runtime = held();
    let context = Context::default();
    assert_eq!(runtime.step(1), 1);
    assert!(runtime.checkpoint().is_some(), "a birth question holds");
    assert_eq!(runtime.press(&Press::char('e'), &context), Ok(false), "play keys wait out the question");
    assert_eq!(runtime.press(&Press::Named(Named::Enter), &context), Ok(true));
    runtime.step(1);
    assert!(matches!(game(&runtime, runtime.trace().len() - 1),
        MesocosmIntent::Checkpoint(answer) if matches!(answer.kind, CheckpointAnswerKind::Birth(_))));
    let mut boards = 0;
    for _ in 0..60 {
        if let Some(review) = runtime.review() {
            let takeable = review.offers.iter().position(|offer| offer.takeable());
            let untakeable = review.offers.iter().position(|offer| !offer.takeable()).expect("the status quo");
            let at = |row| Context { row, ..context };
            assert_eq!(runtime.press(&Press::char('r'), &at(untakeable)), Ok(false), "R takes only a takeable offer");
            let press = match takeable {
                Some(row) => { assert_eq!(runtime.press(&Press::char('r'), &at(row)), Ok(true)); row }
                None => { assert_eq!(runtime.press(&Press::Named(Named::Enter), &context), Ok(true)); 0 }
            };
            runtime.step(1);
            let MesocosmIntent::Checkpoint(answer) = game(&runtime, runtime.trace().len() - 1) else { panic!() };
            assert!(matches!(answer.kind, CheckpointAnswerKind::EpochReview(r) if r.revision.0 == press as u64),
                "the board's answer reaches the session");
            assert!(runtime.review().is_none(), "and closes the boundary");
            boards += 1;
            break;
        }
        if runtime.checkpoint().is_some() { runtime.press(&Press::Named(Named::Enter), &context).unwrap(); }
        runtime.step(1);
    }
    assert_eq!(boards, 1, "the six-tick epoch reaches the board");
    assert_eq!(runtime.fault(), None);
}

#[test]
fn a_click_nudges_toward_a_current_pick_and_a_stale_one_is_refused() {
    let mut runtime = offered();
    let context = Context::default();
    let drawn = runtime.revision().unwrap();
    let thing = runtime.critter().unwrap() + 1;
    let click = Pointed { pick: Pick::Thing(thing), button: Button::Primary, revision: drawn };
    let menu = Pointed { button: Button::Secondary, ..click };
    assert_eq!(runtime.point(&menu, &context), Ok(false), "a right-click opens the window's menu");
    assert_eq!(runtime.current(&menu), Ok(Pick::Thing(thing)));
    assert_eq!(runtime.point(&click, &context), Ok(true));
    let MesocosmIntent::Nudge(nudge) = &queued_then_applied(&mut runtime, 1)[0] else { panic!() };
    assert_eq!(nudge.target, NudgeTarget::Thing(cosmograph_overlay::EntityHandle(thing)));
    let current = runtime.revision().unwrap();
    assert!(matches!(runtime.point(&click, &context), Err(InputRefusal::Stale { picked, .. }) if picked == drawn && current > drawn));
    assert_eq!(runtime.queued_len(), 0);
}

#[test]
fn mashing_stops_at_the_backlog_cap_and_dev_keys_need_dev() {
    let mut runtime = offered();
    let context = Context::default();
    for _ in 0..10 { assert_eq!(runtime.press(&Press::char('e'), &context), Ok(true)); }
    assert_eq!(runtime.press(&Press::char('e'), &context), Err(InputRefusal::Backlog { queued: 10 }));
    runtime.step(1);
    assert_eq!(runtime.press(&Press::char('g'), &context), Ok(false), "dev keys are off");
    let dev = Context { dev: true, ..context };
    // X ends the epoch, which on this world's stock epoch is a long run: mapped, not applied.
    assert!(matches!(runtime.keyed(&Press::char('x'), &dev), Some(MesocosmIntent::Dev(DevIntent::EndEpoch))));
    assert_eq!(runtime.press(&Press::char('g'), &dev), Ok(true));
    assert_eq!(runtime.press(&Press::char('k'), &dev), Ok(true));
    let applied = queued_then_applied(&mut runtime, 2);
    assert!(matches!(&applied[0], MesocosmIntent::Dev(DevIntent::PlaceMatter { .. })));
    assert!(matches!(&applied[1], MesocosmIntent::Dev(DevIntent::Kill { .. })));
}
