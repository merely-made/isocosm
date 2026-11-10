// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The saved loop's input door over a plain native session: the borrowed
//! product door the endpoint uses, with the generated fixture's world.

use cosmograph::{Execution, Session, simulation::Genesis, directing::{interim::{Happening, Interim, OnCollapse, Pace, Start},
    readings::Mode}, schema::Tick};
use cosmograph_overlay::{CandidateHandle, EntityHandle, OrdinaryAccess, ParticipantHandle, Seat, Tick as Stamp,
    mesocosm::{BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, DeathAnswer, RevisionAnswer}};
use mesocosm_runtime::{Refusal, borrowed::submit_game, continuation::{self, LoopAction, LoopAnswer, LoopInput,
    LoopPhase, LoopReading, LoopSettings, Rounds, read_loop}, spatial::{Game, Input}};

#[path = "../../../../../shared/cosmograph/tests/ecology_independence_879/fixture.rs"]
#[allow(dead_code)]
mod native_fixture;

pub const EPOCH: Tick = 8;
const PACE: Pace = Pace { round: 1, scoring: 1 };

pub struct Door { pub session: Session, pub game: Game, pub participant: u64 }

/// A played world with an eight-tick epoch, not yet configured.
pub fn door() -> Door {
    let candidate = native_fixture::candidate_with_played("test:round-endings", 1);
    Door::over(candidate.genesis)
}

/// The same world with the systems sheet rule set (case 1215).
pub fn sheeted() -> Door {
    let mut genesis = native_fixture::candidate_with_played("test:round-endings", 1).genesis;
    genesis.rules.directing.get_or_insert_with(Default::default).sheet = true;
    Door::over(genesis)
}

pub fn settings(rounds: Rounds, condition: bool) -> LoopSettings {
    LoopSettings { mode: Mode::Survival, pace: PACE, on_collapse: OnCollapse::Stay, rounds, condition }
}

impl Door {
    fn over(mut genesis: Genesis) -> Self {
        genesis.rules.epoch_ticks = EPOCH;
        let interim = Interim::found(genesis, Start::default(), Mode::Survival, PACE, Execution::Grouped).unwrap();
        Door { participant: interim.participant, session: interim.session, game: Game::default() }
    }

    pub fn access(&self) -> OrdinaryAccess {
        OrdinaryAccess { participant: ParticipantHandle(self.participant), seat: Seat::Player,
            subject: self.session.sim.plays(self.participant).map(EntityHandle) }
    }

    pub fn tick(&self) -> Tick { self.session.sim.state().tick }

    pub fn saved(&self) -> &continuation::Continuation { self.game.continuation.as_ref().unwrap() }

    pub fn reading(&self) -> LoopReading {
        let access = self.access();
        let visible = cosmograph::directing::readings::view(&self.session.sim, access.subject.unwrap().0, Mode::Survival);
        read_loop(&self.session, &self.game, access, &visible).unwrap()
    }

    pub fn send(&mut self, action: LoopAction) -> Result<String, Refusal> {
        let input = LoopInput { kind: continuation::KIND.into(), version: continuation::VERSION,
            participant: ParticipantHandle(self.participant), tick: Stamp(self.tick()),
            expected: continuation::cut(&self.session, &self.game), action };
        let access = self.access();
        let (out, game) = submit_game(&mut self.session, access, 0, &Input::Loop(input), self.game.clone())?;
        self.game = game;
        // Every question the loop holds at must survive a save and a historical replay.
        if self.saved().checkpoint.is_some() { self.reloaded().expect("the held question validates after a restore"); }
        Ok(out)
    }

    /// The game as saved and read back, replayed against a session loaded from its save.
    pub fn reloaded(&self) -> Result<Game, String> {
        let game: Game = serde_json::from_slice(&serde_json::to_vec(&self.game).unwrap()).unwrap();
        let session = Session::load(self.session.save(), Execution::Grouped)?;
        game.validate_replay(&session)?;
        Ok(game)
    }

    pub fn configure(&mut self, rounds: Rounds, condition: bool) {
        self.send(LoopAction::Configure(settings(rounds, condition))).unwrap();
    }

    pub fn step(&mut self) -> Vec<Happening> {
        self.send(LoopAction::Step).unwrap();
        self.saved().happenings.clone()
    }

    pub fn answer(&mut self, answer: LoopAnswer) -> Result<String, Refusal> { self.send(LoopAction::Answer(answer)) }

    pub fn phase(&self) -> LoopPhase { self.saved().phase }

    /// Answers a birth or a loss that held the run, if one did; a hold on an epoch is left alone.
    pub fn settle(&mut self) {
        use mesocosm_runtime::Occasion;
        let Some(checkpoint) = self.saved().checkpoint.clone() else { return };
        let answer = match &checkpoint.occasion {
            Occasion::Birth(birth) => CheckpointAnswer { entity: EntityHandle(birth.parent),
                kind: CheckpointAnswerKind::Birth(BirthAnswer::KeepParent) },
            Occasion::Loss(loss) if !checkpoint.heirs.is_empty() => CheckpointAnswer { entity: EntityHandle(loss.critter),
                kind: CheckpointAnswerKind::Death(DeathAnswer { next: EntityHandle(checkpoint.heirs[0]) }) },
            _ => return,
        };
        self.answer(LoopAnswer::Checkpoint(answer)).unwrap();
    }

    /// Takes the first offer of the held epoch review, the status quo.
    pub fn review(&mut self) -> Result<String, Refusal> {
        let entity = EntityHandle(self.session.sim.plays(self.participant).unwrap());
        self.answer(LoopAnswer::Checkpoint(CheckpointAnswer { entity,
            kind: CheckpointAnswerKind::EpochReview(RevisionAnswer { revision: CandidateHandle(0) }) }))
    }
}
