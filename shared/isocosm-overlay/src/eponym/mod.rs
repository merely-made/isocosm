// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Eponym's side of the overlay contract (Eponym overlay plan §3; wing
//! design record §5.6 and §9.14), the one game the contract opens actuation
//! to: a player drives the one body they play (rulings 60, 152), and what
//! crosses per tick is the accepted transition of a game-side solver, never
//! its input frames (ruling 233). Asks to peers are proposals a peer weighs
//! by its opinion of the asker (rulings 60, 63); a telling carries its manner
//! (ruling 241); naming and notes are the player's own acts (rulings 36,
//! 130, 168); the checkpoints are a first life (rulings 234, 235) and a death
//! (rulings 186, 238); tag-in lives in creative mode (ruling 187). Eponym's
//! blows come back through an inhabited handoff (ruling 232). There are no
//! dev intents: the crossing fixture's body presets are receipts, not play.
//! See the crate README for the table mapping `eponym-world`'s `GameIntent`
//! and `WorldIntent` and `eponym-identity`'s `ControlIntent` onto this
//! module.

mod act;
mod actuation;
mod ask;
mod checkpoint;
mod handoff;
mod telling;

pub use act::{PlayerAct, PlayerActKind};
pub use actuation::{ActTarget, Actuation, Motion, TimedAct};
pub use ask::{Proposal, Term, TermSide, WorkKey};
pub use checkpoint::{FirstLife, LifeCheckpoint, LifeChoice, StartTime, Succession, Successor};
pub use handoff::{Blow, EponymHandoff};
pub use telling::{Claim, Manner, Telling};

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, HandoffEnvelope, IntentEnvelope};

/// Everything an Eponym host may send the sim: the played body's actuation,
/// an ask, a telling, a player's own act, a checkpoint answer, or creative
/// mode's tag-in and tag-out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EponymIntent {
    Drive(Actuation),
    Ask(Proposal),
    Tell(Telling),
    Act(PlayerAct),
    Checkpoint(LifeCheckpoint),
    Creative(CreativeIntent),
}

/// Creative mode's intents (ruling 187), and nothing else's: tag in to a
/// companion for a while, home remembered, and tag out to home. The shape of
/// `eponym-identity`'s `ControlIntent::{TagIn, TagOut}`; its `Begin` became
/// who the participant plays and its `Succeed` the death checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreativeIntent {
    TagIn { to: EntityHandle },
    TagOut,
}

impl EponymIntent {
    /// Whether this is creative mode's alone (ruling 187), so a run that
    /// carried one is labelled, as Mesocosm labels a run that used a dev
    /// intent assisted.
    pub fn is_creative(&self) -> bool {
        matches!(self, Self::Creative(_))
    }
}

/// An Eponym intent, stamped for its tick and naming its participant.
pub type EponymIntentEnvelope = IntentEnvelope<EponymIntent>;

/// Eponym's handoff envelope, inhabited (ruling 232): a blow the foreground
/// resolved, handed back in the sim's terms.
pub type EponymHandoffEnvelope = HandoffEnvelope<EponymHandoff>;
