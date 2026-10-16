// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Contract envelopes into native commands (ruling 686). A nudge is a
//! native `Nudge` (D1); the player's act is `Speciate`; a checkpoint answer
//! takes up a life or commits the review's offer; a dev intent is native
//! where native has a command, and refused where it has none.

use isocosm_host::world::World;
use isocosm_overlay::Intent;
use isocosm_overlay::mesocosm::{
    MesocosmIntent,
};
use serde::{Deserialize, Serialize};

use super::{Envelope, Mesocosm, commit, session};

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

impl Mesocosm {
    /// Applies one envelope; the core driver keeps it in the trace with what
    /// it came to.
    pub(super) fn apply_envelope(&mut self, world: &World, envelope: &Envelope) -> Result<String, Refusal> {
        let dev = matches!(&envelope.intent, Intent::Game(i) if i.is_dev());
        let tick = session(world).sim.state().tick;
        let outcome = match &envelope.intent {
            // The attention set is the host's to keep (204); nothing enters
            // the sim.
            Intent::Attention(_) => Ok("attention".to_owned()),
            Intent::Game(_) if envelope.participant.0 != self.lp.participant
                || envelope.tick.0 != tick =>
                Err(Refusal::Sim("envelope names another participant or tick".into())),
            Intent::Game(MesocosmIntent::Nudge(_) | MesocosmIntent::Act(_))
                if self.spatial.continuation.as_ref().is_some_and(|saved|
                    saved.phase != crate::continuation::LoopPhase::Running) =>
                Err(Refusal::Sim("ordinary game actions require the running played loop".into())),
            Intent::Game(intent) => self.translate(world, intent),
        };
        if dev && outcome.is_ok() {
            self.dev_intents += 1;
        }
        self.sync_loop(world);
        outcome
    }

    fn translate(&mut self, world: &World, intent: &MesocosmIntent) -> Result<String, Refusal> {
        match intent {
            MesocosmIntent::Checkpoint(answer) => self.answer_saved(world, *answer),
            MesocosmIntent::Dev(isocosm_overlay::mesocosm::DevIntent::ForceBirth { organism }) => {
                let mut game = self.spatial.clone();
                let out = commit(world, |session| game.force_birth(session, organism.0).map_err(Refusal::Sim))?;
                self.spatial = game;
                Ok(out)
            },
            MesocosmIntent::Dev(isocosm_overlay::mesocosm::DevIntent::EndEpoch) => {
                let ticks = {
                    let session = session(world);
                    let epoch = session.sim.genesis().rules.epoch_ticks.max(1);
                    let left = epoch - session.sim.state().tick % epoch;
                    left.saturating_sub(self.lp.pace.round)
                };
                // The existing authored dev clock uses the same saved spatial
                // driver; it cannot bypass chosen-step geometry or arrival data.
                let mut game = self.spatial.clone();
                commit(world, |session| {
                    for _ in 0..ticks { game.advance_clock(session).map_err(Refusal::Sim)?; }
                    Ok(())
                })?;
                self.spatial = game;
                Ok(format!("advanced {ticks}"))
            },
            _ => {
                let (participant, round) = (self.lp.participant, self.lp.pace.round);
                commit(world, |session| direct::apply(session, participant, round, intent))
            },
        }
    }
}
