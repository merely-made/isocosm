// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Today's `Runtime`, pinned before the sim-host core takes over its driving
//! (sim-host plan H3, ruling 928): queue order and trace outcomes, the step
//! ceiling, holds and answer draining, save and resume, and two recorded
//! envelope traces. These pass unchanged before and after H3.

#[path = "runtime_pin/worlds.rs"]
mod worlds;
#[path = "runtime_pin/trace.rs"]
mod trace;

use isocosm_overlay::{AttentionChange, Intent, Tick};
use mesocosm_runtime::{GameSave, Occasion, Refusal, Runtime};
use worlds::{RATE, death_answer, envelope, held, keep_parent, nudge, offered};

#[test]
fn queued_envelopes_apply_in_order_before_the_round_and_the_trace_keeps_each_outcome() {
    let mut runtime = offered();
    let (tick, critter) = (runtime.tick(), runtime.critter().unwrap());
    let mut late = envelope(&runtime, nudge(critter));
    late.tick = Tick(tick + 5);
    for queued in [envelope(&runtime, nudge(critter)), envelope(&runtime, nudge(critter + 1000)),
        envelope(&runtime, Intent::Attention(AttentionChange::StopExamining)), late,
        envelope(&runtime, death_answer(critter, critter))] {
        runtime.queue(queued);
    }
    assert_eq!(runtime.queued_len(), 5);
    assert_eq!(runtime.step(1), 1);
    assert_eq!((runtime.tick(), runtime.queued_len()), (tick + 1, 0), "one round, after every queued envelope");
    let outcomes: Vec<_> = runtime.trace().iter().map(|(_, outcome)| outcome.clone()).collect();
    assert_eq!(outcomes, [Ok("nudged".to_string()), Err(Refusal::NotPlayed), Ok("attention".to_string()),
        Err(Refusal::Sim("envelope names another participant or tick".into())), Err(Refusal::Unasked)]);
    assert_eq!(runtime.trace()[3].0.tick, Tick(tick + 5), "the trace keeps each envelope as queued");
    assert_eq!(runtime.receipt().envelopes, 5);
}

#[test]
fn the_step_ceiling_defers_steps_and_never_drops_one() {
    let mut runtime = offered().with_max_steps(3);
    let tick = runtime.tick();
    let mut calls = vec![runtime.advance(1_000_000)];
    calls.extend((0..10).map(|_| runtime.advance(0)));
    assert_eq!(calls, [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0], "a second at {RATE} Hz arrives three rounds a call");
    assert_eq!(runtime.tick(), tick + 30);
    assert_eq!(offered().advance(1_000_000), mesocosm_runtime::runtime::DEFAULT_MAX_STEPS_PER_ADVANCE);
    let mut uneven = offered();
    let taken: u64 = [400_000u64, 33, 250_000, 349_967].iter().map(|elapsed| uneven.advance(*elapsed)).sum::<u64>()
        + (0..4).map(|_| uneven.advance(0)).sum::<u64>();
    assert_eq!(taken, 30, "uneven delivery authorises the same rounds");
}

#[test]
fn a_held_run_takes_no_round_and_banks_no_time() {
    let mut runtime = held();
    assert_eq!(runtime.step(1), 1);
    let question = runtime.checkpoint().cloned().expect("a birth to the played critter opens a question");
    let Occasion::Birth(birth) = question.occasion else { panic!("the first question is a birth") };
    assert_eq!(runtime.critter(), Some(birth.parent));
    let tick = runtime.tick();
    assert_eq!(runtime.step(3), 0);
    assert_eq!(runtime.advance(5_000_000), 0);
    assert_eq!(runtime.tick(), tick, "nothing runs while a question holds");
    runtime.queue(envelope(&runtime, keep_parent(birth.parent)));
    assert_eq!(runtime.advance(0), 0, "an answer drains in a held call, which takes no round");
    assert!(runtime.checkpoint().is_none());
    assert_eq!(runtime.trace().last().unwrap().1.as_ref().map(|_| ()), Ok(()), "the answer was accepted");
    assert_eq!(runtime.advance(0), 0, "the held seconds were never banked");
    assert_eq!(runtime.step(1), 1);
}

#[test]
fn answers_drain_from_the_front_while_held_and_anything_else_waits() {
    let open = |answer_first: bool| {
        let mut runtime = held();
        runtime.step(1);
        let Occasion::Birth(birth) = runtime.checkpoint().unwrap().occasion.clone() else { panic!("a birth") };
        let (answer, other) = (envelope(&runtime, keep_parent(birth.parent)), envelope(&runtime, nudge(birth.parent)));
        if answer_first { runtime.queue(answer); runtime.queue(other); } else { runtime.queue(other); runtime.queue(answer); }
        runtime
    };
    let mut first = open(true);
    let tick = first.tick();
    assert_eq!(first.step(1), 1, "the answer drains, then the rest applies and the round runs");
    assert_eq!((first.queued_len(), first.tick()), (0, tick + 1));
    assert!(first.trace().iter().all(|(_, outcome)| outcome.is_ok()));
    let mut behind = open(false);
    assert_eq!(behind.step(1), 0, "an answer behind another envelope waits with it");
    assert_eq!((behind.queued_len(), behind.trace().len()), (2, 0));
    assert!(behind.checkpoint().is_some());
    let mut wrong = held();
    wrong.step(1);
    let parent = wrong.critter().unwrap();
    wrong.queue(envelope(&wrong, death_answer(parent, parent)));
    assert_eq!(wrong.step(1), 0, "an answer of another kind does not drain");
    assert_eq!(wrong.queued_len(), 1);
}

#[test]
fn save_and_resume_keep_the_question_the_queue_and_the_world() {
    let mut asked = held();
    asked.step(1);
    let resumed = |runtime: &Runtime| {
        let bytes = serde_json::to_vec(&runtime.save_game()).unwrap();
        let saved: GameSave = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(Runtime::replayed_game(saved.clone()).unwrap().state_hash, runtime.state_hash());
        Runtime::resume_game_with_mode(saved, RATE, None).unwrap()
    };
    let mut again = resumed(&asked);
    assert_eq!((again.tick(), again.state_hash(), again.checkpoint()), (asked.tick(), asked.state_hash(), asked.checkpoint()));
    assert!(again.trace().is_empty(), "the trace is the driver's, not the save's");
    for runtime in [&mut asked, &mut again] {
        let parent = runtime.critter().unwrap();
        let answer = envelope(runtime, keep_parent(parent));
        runtime.queue(answer);
        runtime.step(4);
    }
    assert_eq!((again.tick(), again.state_hash()), (asked.tick(), asked.state_hash()));
    // A queue survives too.
    let mut running = offered();
    let critter = running.critter().unwrap();
    running.queue(envelope(&running, nudge(critter)));
    let mut restored = resumed(&running);
    assert_eq!(restored.queued_len(), 1);
    for runtime in [&mut running, &mut restored] { assert_eq!(runtime.step(1), 1); }
    assert_eq!(restored.trace().last().unwrap().1, Ok("nudged".to_string()));
    assert_eq!(restored.state_hash(), running.state_hash());
}

#[test]
fn the_recorded_traces_replay_to_their_recorded_hashes() {
    for (name, world) in [("offered", offered as fn() -> Runtime), ("held", held)] {
        trace::check(name, world);
    }
}
