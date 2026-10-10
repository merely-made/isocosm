// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Contract envelopes into native commands (ruling 686). A nudge is a
//! native `Nudge` (D1); the player's act is `Speciate`; a checkpoint answer
//! takes up a life or commits the review's offer; a dev intent is native
//! where native has a command, and refused where it has none.

use isocosm::directing::{Aim, Toward};
use isocosm::history::Command;
use isocosm_overlay::Intent;
use isocosm_overlay::mesocosm::{
    BirthAnswer, CheckpointAnswerKind, DevIntent, MesocosmIntent, NudgeMeaning, NudgeTarget,
    PlayerActKind,
};
use serde::{Deserialize, Serialize};

use super::{Envelope, Runtime};
use crate::succession::Occasion;

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

/// The account a dev's placed matter enters, as the interim's tests use.
const PLACED: &str = "world:soil";

impl Runtime {
    /// Applies one envelope and records it with what it came to.
    pub(super) fn apply(&mut self, envelope: Envelope) {
        let dev = matches!(&envelope.intent, Intent::Game(i) if i.is_dev());
        let outcome = match &envelope.intent {
            // The attention set is the host's to keep (204); nothing enters
            // the sim.
            Intent::Attention(_) => Ok("attention".to_owned()),
            Intent::Game(intent) => self.translate(intent),
        };
        if dev && outcome.is_ok() {
            self.dev_intents += 1;
        }
        self.trace.push((envelope, outcome));
    }

    fn translate(&mut self, intent: &MesocosmIntent) -> Result<String, Refusal> {
        let sim = |why: String| Refusal::Sim(why);
        match intent {
            MesocosmIntent::Nudge(nudge) => {
                self.playing(nudge.entity.0)?;
                let (aim, act) = match &nudge.meaning {
                    NudgeMeaning::Attend => (Aim::Attend, None),
                    // The act key names the process it asks for (784).
                    NudgeMeaning::Act(key) => (Aim::Act, Some(key.0.clone())),
                };
                let toward = match nudge.target {
                    NudgeTarget::Place(place) => Toward::Site(place.0),
                    NudgeTarget::Thing(thing) => Toward::Thing(thing.0),
                };
                self.interim.nudge(aim, toward, act).map_err(sim)
            },
            MesocosmIntent::Act(act) => {
                self.playing(act.entity.0)?;
                let PlayerActKind::Speciate { name } = &act.kind;
                let command = Command::Speciate {
                    founder: act.entity.0,
                    name: name.clone(),
                };
                self.interim.session.command(command).map_err(sim)
            },
            MesocosmIntent::Checkpoint(answer) => self.answer(answer.kind),
            MesocosmIntent::Dev(dev) => self.dev(*dev),
        }
    }

    fn playing(&self, entity: u64) -> Result<(), Refusal> {
        (self.critter() == Some(entity))
            .then_some(())
            .ok_or(Refusal::NotPlayed)
    }

    fn answer(&mut self, kind: CheckpointAnswerKind) -> Result<String, Refusal> {
        let Some(checkpoint) = self.checkpoint.clone() else {
            return Err(Refusal::Unasked);
        };
        let taken = match (kind, &checkpoint.occasion) {
            (CheckpointAnswerKind::Birth(BirthAnswer::KeepParent), Occasion::Birth(_)) => None,
            (CheckpointAnswerKind::Birth(BirthAnswer::TakeOffspring), Occasion::Birth(b)) => {
                Some(b.child)
            },
            (CheckpointAnswerKind::Death(death), Occasion::Loss(_)) => {
                if !checkpoint.heirs.contains(&death.next.0) {
                    return Err(Refusal::Unasked);
                }
                Some(death.next.0)
            },
            (CheckpointAnswerKind::EpochReview(answer), Occasion::Epoch(_)) => {
                let review = self.review.as_ref().ok_or(Refusal::Unasked)?;
                let index = usize::try_from(answer.revision.0).unwrap_or(usize::MAX);
                let offer = review.offers.get(index).ok_or(Refusal::Unasked)?;
                if let Some(why) = &offer.why_not {
                    return Err(Refusal::Sim(why.clone()));
                }
                // The status quo sends nothing and closes the turn.
                let mut commands = offer.commands.clone();
                // The first script's placement that held, sent after the
                // revision (787).
                let critter = self.critter().ok_or(Refusal::NotPlayed)?;
                let authored = self.proposed.iter().filter(|p| p.offer == index);
                if let Some(p) = authored.into_iter().find(|p| p.cells.is_ok()) {
                    commands.extend(p.commands(critter));
                }
                for command in commands {
                    self.interim
                        .session
                        .command(command)
                        .map_err(Refusal::Sim)?;
                }
                None
            },
            _ => return Err(Refusal::Unasked),
        };
        if let Some(critter) = taken {
            self.interim.take(critter).map_err(Refusal::Sim)?;
        }
        // One answer closes the question, a committed revision included.
        self.checkpoint = None;
        self.review = None;
        self.proposed.clear();
        Ok("answered".to_owned())
    }

    fn dev(&mut self, dev: DevIntent) -> Result<String, Refusal> {
        match dev {
            DevIntent::EndEpoch => {
                let s = &self.interim.session.sim;
                let epoch = s.genesis().rules.epoch_ticks.max(1);
                let left = epoch - s.state().tick % epoch;
                // The round that follows crosses the boundary.
                let ticks = left.saturating_sub(self.interim.pace.round);
                self.interim
                    .session
                    .advance(ticks)
                    .map(|_| format!("advanced {ticks}"))
                    .map_err(Refusal::Sim)
            },
            DevIntent::PlaceMatter { site, mass_mg } => {
                let command = Command::PlaceMatter {
                    site: site.0,
                    account: PLACED.into(),
                    amount: mass_mg,
                };
                self.interim.session.command(command).map_err(Refusal::Sim)
            },
            DevIntent::ForceBirth { organism } => {
                let command = Command::ForceBirth { parent: organism.0 };
                self.interim.session.command(command).map_err(Refusal::Sim)
            },
            DevIntent::Kill { organism } => {
                let command = Command::Kill { entity: organism.0 };
                self.interim.session.command(command).map_err(Refusal::Sim)
            },
        }
    }
}
