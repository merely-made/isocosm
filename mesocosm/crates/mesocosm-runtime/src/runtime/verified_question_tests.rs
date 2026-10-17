// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm_overlay::{EntityHandle, Intent, IntentEnvelope, ParticipantHandle, Tick,
    mesocosm::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, MesocosmIntent}};

#[path = "../../../../../shared/isocosm/tests/ecology_independence_879/fixture.rs"]
#[allow(dead_code)]
mod native_fixture;

fn held() -> Runtime {
    let mut candidate = native_fixture::candidate_with_played("test:verified-question", 1);
    candidate.genesis.rules.epoch_ticks = 6;
    let interim = Interim::found(candidate.genesis, Start::default(), Mode::Survival,
        Pace { round: 1, scoring: 1 }, Execution::Grouped).unwrap();
    let mut runtime = Runtime::resume(interim.session.save(), Mode::Survival,
        Pace { round: 1, scoring: 1 }, 30).unwrap();
    assert_eq!(runtime.step(1), 1);
    assert!(runtime.product().verified_question.is_some());
    runtime
}

fn answer(runtime: &Runtime) -> isocosm_overlay::mesocosm::CheckpointAnswer {
    CheckpointAnswer { entity: EntityHandle(runtime.critter().unwrap()),
        kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent) }
}

fn queue_answer(runtime: &mut Runtime) {
    let answer = answer(runtime);
    runtime.queue(IntentEnvelope { tick: Tick(runtime.tick()),
        participant: ParticipantHandle(runtime.interim().participant),
        intent: Intent::Game(MesocosmIntent::Checkpoint(answer)) });
}

#[test]
fn live_and_reloaded_receipts_match_the_public_replayed_answer() {
    for reloaded in [false, true] {
        let runtime = held();
        let mut runtime = if reloaded {
            Runtime::resume_world(runtime.save_world().unwrap(), 30, None).unwrap()
        } else { runtime };
        assert!(runtime.product().verified_question.is_some());
        let mut session = runtime.session().clone();
        let mut game = runtime.product().loop_game(runtime.world(), Vec::new());
        let input = crate::continuation::LoopInput {
            kind: crate::continuation::KIND.into(), version: crate::continuation::VERSION,
            participant: ParticipantHandle(runtime.interim().participant), tick: Tick(runtime.tick()),
            expected: crate::continuation::cut(&session, &game),
            action: crate::continuation::LoopAction::Answer(
                crate::continuation::LoopAnswer::Checkpoint(answer(&runtime))),
        };
        let access = runtime.product().loop_access(runtime.world());
        crate::continuation::answer_loop(&mut session, &mut game, access, &input).unwrap();
        queue_answer(&mut runtime);
        assert_eq!(runtime.advance(0), 0);
        assert!(runtime.trace().last().unwrap().1.is_ok());
        assert_eq!(runtime.session().save(), session.save());
        assert_eq!(runtime.world().section(MODE).unwrap().unwrap(), serde_json::to_vec(&game).unwrap());
    }
}

#[test]
fn externally_changed_historical_source_cannot_reuse_a_receipt() {
    let mut runtime = held();
    let before = runtime.session().save();
    let mut game = runtime.spatial().clone();
    game.continuation.as_mut().unwrap().round.as_mut().unwrap().native = "forged prefix".into();
    // The current seal and the question still match; only historical replay
    // catches this forged past, and the old process-local receipt must miss.
    game.validate(&runtime.session()).unwrap();
    runtime.world().put_section(MODE, serde_json::to_vec(&game).unwrap()).unwrap();
    queue_answer(&mut runtime);
    assert_eq!(runtime.advance(0), 0);
    assert!(runtime.trace().last().unwrap().1.is_err());
    assert_eq!(runtime.session().save(), before);
    assert!(runtime.checkpoint().is_some());
}
