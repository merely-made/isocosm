// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Contract envelopes into native commands (ruling 686). A nudge is a
//! native `Nudge` (D1); the player's act is `Speciate`; a checkpoint answer
//! takes up a life or commits the review's offer; a dev intent is native
//! where native has a command, and refused where it has none.

use isocosm_overlay::Intent;
use isocosm_overlay::mesocosm::{
    MesocosmIntent,
};
use serde::{Deserialize, Serialize};

use super::{Envelope, Runtime};

/// Why an envelope was not applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    /// The envelope names a critter the participant does not play.
    NotPlayed,
    /// No question stands that this answers.
    Unasked,
    /// Native has no command for it yet.
    Unbuilt(String),
    /// The sim refused the command.
    Sim(String),
}

pub(crate) mod direct;

impl Runtime {
    /// Applies one envelope and records it with what it came to.
    pub(super) fn apply(&mut self, envelope: Envelope) {
        let dev = matches!(&envelope.intent, Intent::Game(i) if i.is_dev());
        let outcome = match &envelope.intent {
            // The attention set is the host's to keep (204); nothing enters
            // the sim.
            Intent::Attention(_) => Ok("attention".to_owned()),
            Intent::Game(_) if envelope.participant.0 != self.interim.participant
                || envelope.tick.0 != self.tick() =>
                Err(Refusal::Sim("envelope names another participant or tick".into())),
            Intent::Game(MesocosmIntent::Nudge(_) | MesocosmIntent::Act(_))
                if self.spatial.continuation.as_ref().is_some_and(|saved|
                    saved.phase != crate::continuation::LoopPhase::Running) =>
                Err(Refusal::Sim("ordinary game actions require the running played loop".into())),
            Intent::Game(intent) => self.translate(intent),
        };
        if dev && outcome.is_ok() {
            self.dev_intents += 1;
        }
        self.sync_loop();
        self.trace.push((envelope, outcome));
    }

    fn translate(&mut self, intent: &MesocosmIntent) -> Result<String, Refusal> {
        match intent {
            MesocosmIntent::Checkpoint(answer) => self.answer_saved(*answer),
            MesocosmIntent::Dev(isocosm_overlay::mesocosm::DevIntent::ForceBirth { organism }) =>
                self.spatial.force_birth(&mut self.interim.session, organism.0).map_err(Refusal::Sim),
            MesocosmIntent::Dev(isocosm_overlay::mesocosm::DevIntent::EndEpoch) => {
                let epoch = self.interim.session.sim.genesis().rules.epoch_ticks.max(1);
                let left = epoch - self.interim.session.sim.state().tick % epoch;
                let ticks = left.saturating_sub(self.interim.pace.round);
                // The existing authored dev clock uses the same saved spatial
                // driver; it cannot bypass chosen-step geometry or arrival data.
                let mut session = self.interim.session.clone();
                let mut game = self.spatial.clone();
                for _ in 0..ticks { game.advance_clock(&mut session).map_err(Refusal::Sim)?; }
                self.interim.session = session;
                self.spatial = game;
                Ok(format!("advanced {ticks}"))
            },
            _ => direct::apply(&mut self.interim.session, self.interim.participant,
                self.interim.pace.round, intent),
        }
    }

}
