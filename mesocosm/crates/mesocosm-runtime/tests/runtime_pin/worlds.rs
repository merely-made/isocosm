// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The two worlds the pins run on, and envelope helpers. Both come from the
//! native ecology fixture's generated candidate: `offered` through the 879
//! offer gate, `held` founded through `Interim` with a six-tick epoch and
//! resumed, where a birth question opens at once.

use cosmograph::{Execution, directing::{interim::{Interim, OnCollapse, Pace, Start}, readings::Mode}};
use cosmograph_overlay::{EntityHandle, Intent, IntentEnvelope, ParticipantHandle, PlaceHandle, Tick,
    mesocosm::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, DeathAnswer, MesocosmIntent, Nudge,
        NudgeMeaning, NudgeTarget}};
use mesocosm_runtime::{Founded, Runtime, runtime::{Envelope, ecology::Setup}};

#[path = "../../../../../shared/cosmograph/tests/ecology_independence_879/fixture.rs"]
#[allow(dead_code)]
mod native_fixture;

pub const RATE: u32 = 30;
const PACE: Pace = Pace { round: 1, scoring: 1 };

/// A generated world through the offer gate; no question opens for 3,000 rounds.
pub fn offered() -> Runtime {
    let candidate = native_fixture::candidate_with_played("test:offered-cycle", 1);
    Setup { version: 1,
        founded: Founded { genesis: candidate.genesis, start: Start::default(), mode: Mode::Survival,
            pace: PACE, on_collapse: OnCollapse::Stay },
        specification: candidate.spec, execution: Execution::Grouped, spatial: None, generated_templates: false }.offer(RATE).unwrap()
}

/// The same candidate with a six-tick epoch, founded and resumed: a birth
/// to the played critter opens a question in the first round.
pub fn held() -> Runtime {
    let mut candidate = native_fixture::candidate_with_played("test:offered-cycle", 1);
    candidate.genesis.rules.epoch_ticks = 6;
    let interim = Interim::found(candidate.genesis, Start::default(), Mode::Survival, PACE, Execution::Grouped).unwrap();
    Runtime::resume(interim.session.save(), Mode::Survival, PACE, RATE).unwrap()
}

pub fn envelope(runtime: &Runtime, intent: Intent<MesocosmIntent>) -> Envelope {
    IntentEnvelope { tick: Tick(runtime.tick()), participant: ParticipantHandle(runtime.interim().participant), intent }
}

pub fn nudge(entity: u64) -> Intent<MesocosmIntent> {
    Intent::Game(MesocosmIntent::Nudge(Nudge { entity: EntityHandle(entity), target: NudgeTarget::Place(PlaceHandle(0)),
        meaning: NudgeMeaning::Attend }))
}

pub fn keep_parent(parent: u64) -> Intent<MesocosmIntent> {
    Intent::Game(MesocosmIntent::Checkpoint(CheckpointAnswer { entity: EntityHandle(parent),
        kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent) }))
}

pub fn death_answer(entity: u64, next: u64) -> Intent<MesocosmIntent> {
    Intent::Game(MesocosmIntent::Checkpoint(CheckpointAnswer { entity: EntityHandle(entity),
        kind: CheckpointAnswerKind::Death(DeathAnswer { next: EntityHandle(next) }) }))
}
