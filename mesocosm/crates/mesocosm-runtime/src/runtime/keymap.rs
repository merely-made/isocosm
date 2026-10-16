// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's keymap (sim-host plan H4, rulings 954 and 955): what a press or
//! a current pick asks of the world. The player directs and never drives
//! (671, 679), so no key moves, eats or digs. E and Space nudge toward the
//! focused thing or the played site, S speciates; while a question stands the
//! keys narrow to its answers (Enter carries on, T takes the body on offer);
//! at the trait board R commits the selected offer; with dev keys live, X ends
//! the epoch, F forces a birth, K kills and G places matter. A primary click on
//! a current pick nudges toward it. Keys that only move host state (camera,
//! inspection, the dev clock, the board cursor, the hint menu) are the window's.

use isocosm::schema::Id;
use isocosm_host::drive::{Button, Context, Named, Pick, Press};
use isocosm_host::world::World;
use isocosm_overlay::mesocosm::{
    BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, DeathAnswer, DevIntent, MesocosmIntent, Nudge,
    NudgeMeaning, NudgeTarget, PlayerAct, PlayerActKind, RevisionAnswer,
};
use isocosm_overlay::{CandidateHandle, EntityHandle, Intent, ParticipantHandle, PlaceHandle, Tick};

use super::{Envelope, Mesocosm, session};
use crate::succession::{Checkpoint, Occasion};

/// Matter a dev G key places at the played site, in milligrams.
pub const DEV_PLACE_MG: u64 = 5_000;

impl Mesocosm {
    /// Stamps an intent for the world's current tick and the participant.
    pub(super) fn envelope(&self, world: &World, intent: MesocosmIntent) -> Envelope {
        Envelope {
            tick: Tick(session(world).sim.state().tick),
            participant: ParticipantHandle(self.lp.participant),
            intent: Intent::Game(intent),
        }
    }

    fn site(&self, world: &World) -> Option<Id> {
        let critter = self.critter(world)?;
        session(world).sim.state().population.get(critter)?.resident_site()
    }

    pub(super) fn keyed(&self, world: &World, press: &Press, context: &Context) -> Option<MesocosmIntent> {
        let critter = self.critter(world);
        if context.dev && let Some(dev) = self.dev(world, press, context) {
            return Some(MesocosmIntent::Dev(dev));
        }
        if let Some(review) = &self.review && *press == Press::Char('r') {
            return review.offers.get(context.row).filter(|offer| offer.takeable())
                .map(|_| revision(critter, context.row));
        }
        if let Some(checkpoint) = &self.checkpoint {
            return answer(checkpoint, critter, press);
        }
        let (critter, site) = (critter?, self.site(world)?);
        match press {
            Press::Char('e') | Press::Named(Named::Space) =>
                Some(nudge(critter, site, context.focus, NudgeMeaning::Attend)),
            Press::Char('s') => Some(MesocosmIntent::Act(PlayerAct {
                entity: EntityHandle(critter),
                kind: PlayerActKind::Speciate { name: format!("line-{}", session(world).sim.state().tick) },
            })),
            _ => None,
        }
    }

    fn dev(&self, world: &World, press: &Press, context: &Context) -> Option<DevIntent> {
        let followed = EntityHandle(context.focus.or_else(|| self.critter(world)).unwrap_or_default());
        Some(match press {
            Press::Char('x') => DevIntent::EndEpoch,
            Press::Char('f') => DevIntent::ForceBirth { organism: followed },
            Press::Char('k') => DevIntent::Kill { organism: followed },
            Press::Char('g') => DevIntent::PlaceMatter { site: PlaceHandle(self.site(world)?), mass_mg: DEV_PLACE_MG },
            _ => return None,
        })
    }

    pub(super) fn pointed(&self, world: &World, pick: Pick, button: Button) -> Option<MesocosmIntent> {
        if button != Button::Primary || self.checkpoint.is_some() { return None; }
        let target = match pick {
            Pick::Thing(id) => NudgeTarget::Thing(EntityHandle(id)),
            Pick::Place(id) => NudgeTarget::Place(PlaceHandle(id)),
            Pick::Nothing => return None,
        };
        Some(MesocosmIntent::Nudge(Nudge { entity: EntityHandle(self.critter(world)?), target, meaning: NudgeMeaning::Attend }))
    }
}

/// What a nudge is aimed at: the focused thing, else the played critter's site.
pub fn nudge(critter: Id, site: Id, focus: Option<Id>, meaning: NudgeMeaning) -> MesocosmIntent {
    let target = match focus.filter(|thing| *thing != critter) {
        Some(thing) => NudgeTarget::Thing(EntityHandle(thing)),
        None => NudgeTarget::Place(PlaceHandle(site)),
    };
    MesocosmIntent::Nudge(Nudge { entity: EntityHandle(critter), target, meaning })
}

/// The answer a press gives while a checkpoint stands: Enter carries on (keep
/// the parent, take the first heir, close the boundary), T takes the body on
/// offer.
fn answer(checkpoint: &Checkpoint, critter: Option<Id>, press: &Press) -> Option<MesocosmIntent> {
    let take = match press {
        Press::Named(Named::Enter) => false,
        Press::Char('t') => true,
        _ => return None,
    };
    let (entity, kind) = match &checkpoint.occasion {
        Occasion::Birth(birth) => (birth.parent, CheckpointAnswerKind::Birth(
            if take { BirthAnswer::TakeOffspring } else { BirthAnswer::KeepParent })),
        Occasion::Loss(loss) => (loss.critter,
            CheckpointAnswerKind::Death(DeathAnswer { next: EntityHandle(checkpoint.heir()?) })),
        Occasion::Epoch(_) if !take => (critter.unwrap_or_default(), review_answer(0)),
        Occasion::Epoch(_) => return None,
    };
    Some(MesocosmIntent::Checkpoint(CheckpointAnswer { entity: EntityHandle(entity), kind }))
}

/// Offer `index` of the boundary's review as an answer; offer 0 is the status quo.
pub fn revision(critter: Option<Id>, index: usize) -> MesocosmIntent {
    MesocosmIntent::Checkpoint(CheckpointAnswer {
        entity: EntityHandle(critter.unwrap_or_default()),
        kind: review_answer(index),
    })
}

fn review_answer(index: usize) -> CheckpointAnswerKind {
    CheckpointAnswerKind::EpochReview(RevisionAnswer { revision: CandidateHandle(index as u64) })
}
