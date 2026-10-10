// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Key-to-envelope policy: what a keypress means, and whether it is let into
//! the queue.
//!
//! The player directs and never drives (wing rulings 671, 679), so no key
//! moves, eats or digs. Play keys send a nudge toward the played site or the
//! picked thing, or speciate; at a checkpoint the keys narrow to its answers;
//! the board has its own two keys. Everything here builds contract envelopes
//! ([`isocosm_overlay::mesocosm::MesocosmIntent`]); whether one is admitted is
//! the runtime's and the sim's to say, through [`mesocosm_runtime::Refusal`].
//!
//! Auto-repeat is dropped by the window handler, and [`admits`] caps the
//! backlog so mashing a key cannot build a session-long debt. A replay never
//! comes through here.
//!
//! # Dev keys (DT1, DT2, DT3)
//!
//! Live only while `--dev` is set: `P` pause, `.` step, `,` step ten, `[`/`]`
//! speed, `N`/`B` follow the next or previous living critter, `M` follow the
//! played one, and four that queue dev intents: `X` end the epoch, `F` force a
//! birth, `K` kill, `G` place matter. The runtime refuses `F` and `K` until
//! native has a command for them. The first eight never reach the queue.

use isocosm::schema::Id;
use isocosm_overlay::mesocosm::{
    BirthAnswer, CheckpointAnswer, CheckpointAnswerKind, DeathAnswer, MesocosmIntent, Nudge,
    NudgeMeaning, NudgeTarget, PlayerAct, PlayerActKind, RevisionAnswer,
};
use isocosm_overlay::{
    ActKey, CandidateHandle, EntityHandle, Intent, ParticipantHandle, PlaceHandle, Tick,
};
use mesocosm_runtime::{Checkpoint, Envelope, Occasion, Runtime};
use winit::keyboard::{Key, NamedKey};

/// Envelopes of backlog a new key tolerates before it is dropped.
pub const QUEUE_CAP: usize = 10;

/// The act key a host-side act nudge carries; at site grain native reads none (690).
pub const ACT_KEY: &str = "act";

/// Whether a new envelope should be queued, given the current backlog.
pub fn admits(queued_len: usize) -> bool {
    queued_len < QUEUE_CAP
}

/// Stamps an intent for the runtime's current tick and participant.
pub fn envelope(runtime: &Runtime, intent: MesocosmIntent) -> Envelope {
    Envelope {
        tick: Tick(runtime.tick()),
        participant: ParticipantHandle(runtime.interim().participant),
        intent: Intent::Game(intent),
    }
}

/// What a nudge is aimed at: the picked thing, else the played critter's site.
pub fn nudge(critter: Id, site: Id, picked: Option<Id>, meaning: NudgeMeaning) -> MesocosmIntent {
    let target = match picked.filter(|thing| *thing != critter) {
        Some(thing) => NudgeTarget::Thing(EntityHandle(thing)),
        None => NudgeTarget::Place(PlaceHandle(site)),
    };
    MesocosmIntent::Nudge(Nudge {
        entity: EntityHandle(critter),
        target,
        meaning,
    })
}

/// A play key's intent while no question stands. `None` for every other key.
pub fn intent_for(runtime: &Runtime, picked: Option<Id>, key: &Key) -> Option<MesocosmIntent> {
    let critter = runtime.critter()?;
    let site = runtime.played()?.place;
    let letter = match key {
        Key::Named(NamedKey::Space) => "e".to_owned(),
        Key::Character(c) => c.to_lowercase(),
        _ => return None,
    };
    match letter.as_str() {
        "e" => Some(nudge(critter, site, picked, NudgeMeaning::Attend)),
        "q" => Some(nudge(
            critter,
            site,
            picked,
            NudgeMeaning::Act(ActKey(ACT_KEY.into())),
        )),
        "s" => Some(MesocosmIntent::Act(PlayerAct {
            entity: EntityHandle(critter),
            kind: PlayerActKind::Speciate {
                name: format!("line-{}", runtime.tick()),
            },
        })),
        _ => None,
    }
}

/// The answer a key gives while a checkpoint stands: Enter carries on (keep
/// the parent, take the first heir, close the boundary), T takes the body on
/// offer.
pub fn answer_for(
    checkpoint: &Checkpoint,
    critter: Option<Id>,
    key: &Key,
) -> Option<MesocosmIntent> {
    let take = match key {
        Key::Named(NamedKey::Enter) => false,
        Key::Character(c) if c.eq_ignore_ascii_case("t") => true,
        _ => return None,
    };
    let (entity, kind) = match &checkpoint.occasion {
        Occasion::Birth(birth) => (
            birth.parent,
            CheckpointAnswerKind::Birth(if take {
                BirthAnswer::TakeOffspring
            } else {
                BirthAnswer::KeepParent
            }),
        ),
        Occasion::Loss(loss) => (
            loss.critter,
            CheckpointAnswerKind::Death(DeathAnswer {
                next: EntityHandle(checkpoint.heir()?),
            }),
        ),
        Occasion::Epoch(_) if !take => (critter.unwrap_or_default(), status_quo()),
        Occasion::Epoch(_) => return None,
    };
    Some(MesocosmIntent::Checkpoint(CheckpointAnswer {
        entity: EntityHandle(entity),
        kind,
    }))
}

/// Offer `index` of the boundary's review as an answer; offer 0 is the status quo.
pub fn revision(critter: Option<Id>, index: usize) -> MesocosmIntent {
    MesocosmIntent::Checkpoint(CheckpointAnswer {
        entity: EntityHandle(critter.unwrap_or_default()),
        kind: CheckpointAnswerKind::EpochReview(RevisionAnswer {
            revision: CandidateHandle(index as u64),
        }),
    })
}

fn status_quo() -> CheckpointAnswerKind {
    CheckpointAnswerKind::EpochReview(RevisionAnswer {
        revision: CandidateHandle(0),
    })
}

/// What a key means while the trait board is standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKey {
    /// Move the selection to the next offer, wrapping.
    Next,
    /// Commit the selected offer.
    Commit,
}

pub fn board_key(key: &Key) -> Option<BoardKey> {
    match key {
        Key::Named(NamedKey::Tab) => Some(BoardKey::Next),
        Key::Character(c) if c.eq_ignore_ascii_case("r") => Some(BoardKey::Commit),
        _ => None,
    }
}

/// A dev-only action a key maps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevKey {
    TogglePause,
    Step,
    StepN,
    SlowDown,
    SpeedUp,
    FollowNext,
    FollowBack,
    FollowSelf,
    EndEpoch,
    ForceBirth,
    Kill,
    PlaceMatter,
}

impl DevKey {
    /// Whether this key queues a dev intent rather than moving host state.
    pub fn changes_the_world(self) -> bool {
        matches!(
            self,
            Self::EndEpoch | Self::ForceBirth | Self::Kill | Self::PlaceMatter
        )
    }
}

pub fn dev_key(key: &Key) -> Option<DevKey> {
    let Key::Character(c) = key else { return None };
    Some(match c.to_lowercase().as_str() {
        "p" => DevKey::TogglePause,
        "." => DevKey::Step,
        "," => DevKey::StepN,
        "[" => DevKey::SlowDown,
        "]" => DevKey::SpeedUp,
        "n" => DevKey::FollowNext,
        "b" => DevKey::FollowBack,
        "m" => DevKey::FollowSelf,
        "x" => DevKey::EndEpoch,
        "f" => DevKey::ForceBirth,
        "k" => DevKey::Kill,
        "g" => DevKey::PlaceMatter,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_admits_up_to_its_bound() {
        assert!(admits(QUEUE_CAP - 1));
        assert!(!admits(QUEUE_CAP));
    }

    #[test]
    fn dev_keys_never_collide_with_play_keys() {
        for key in ["e", "q", "s", "t", "r"] {
            assert_eq!(dev_key(&Key::Character(key.into())), None, "{key}");
        }
        assert_eq!(board_key(&Key::Named(NamedKey::Enter)), None);
    }
}
