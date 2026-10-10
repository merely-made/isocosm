// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What is asked and agreed: crafts, work, terms, offers and standing
//! agreements (rulings 63 and 67). An agreement is still not a command.

use crate::schema::{Entity, Id, Tick};
use serde::{Deserialize, Serialize};

/// A craft a peer may hold, kept as the skill `craft:<name>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Craft {
    Scouting,
    Smithing,
    Healing,
    Hauling,
    Watching,
}

impl Craft {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scouting => "scouting",
            Self::Smithing => "smithing",
            Self::Healing => "healing",
            Self::Hauling => "hauling",
            Self::Watching => "watching",
        }
    }
    /// The skill a body keeps it under.
    pub fn skill(&self) -> String {
        format!("craft:{}", self.name())
    }
    /// The grade `e` holds in it, nought where it holds none.
    pub fn grade_of(&self, e: &Entity) -> u8 {
        e.skills.get(&self.skill()).map_or(0, |g| (*g).min(255) as u8)
    }
}

/// Where a peer's caution is kept among its disposition's axes. *Reading,
/// not ruled:* the native disposition names no axes yet.
pub const CAUTION: usize = 0;

pub fn caution_of(e: &Entity) -> i16 {
    e.disposition[CAUTION]
}

/// A peer's read of whether work is within its craft.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confidence {
    pub craft: Craft,
    pub demanded: u8,
    pub held: u8,
}

impl Confidence {
    pub fn read(e: &Entity, work: &Work) -> Self {
        Self {
            craft: work.craft,
            demanded: work.grade,
            held: work.craft.grade_of(e),
        }
    }
    pub fn margin(&self) -> i16 {
        i16::from(self.held) - i16::from(self.demanded)
    }
    pub fn sufficient(&self) -> bool {
        self.margin() >= 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub craft: Craft,
    pub grade: u8,
    pub danger: u8,
}

impl Work {
    pub fn new(craft: Craft, grade: u8, danger: u8) -> Self {
        Self { craft, grade, danger }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terms {
    pub share: u8,
    pub danger_cap: u8,
}

impl Terms {
    pub fn new(share: u8, danger_cap: u8) -> Self {
        Self { share, danger_cap }
    }
}

/// An ask put by one sophont to another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub asked_by: Id,
    pub asked_of: Id,
    pub work: Work,
    pub terms: Terms,
}

impl Offer {
    pub fn new(asked_by: Id, asked_of: Id, work: Work, terms: Terms) -> Self {
        Self { asked_by, asked_of, work, terms }
    }
    pub fn to(&self, asked_of: Id) -> Self {
        Self { asked_of, ..*self }
    }
    pub fn on(&self, terms: Terms) -> Self {
        Self { terms, ..*self }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    Withdrawn,
    Resigned,
    WorkDone,
    PremisesChanged,
}

impl EndReason {
    pub fn phrase(&self) -> &'static str {
        match self {
            Self::Withdrawn => "withdrawn",
            Self::Resigned => "resigned",
            Self::WorkDone => "the work is done",
            Self::PremisesChanged => "what it rested on changed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementState {
    Standing,
    Ended { at: Tick, why: EndReason },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementChange {
    Formed,
    Exercised,
    Renegotiated { from: Terms, to: Terms },
    Ended(EndReason),
}

/// One change to an agreement and the deed that recorded it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgreementEvent {
    pub at: Tick,
    pub deed: crate::schema::Key,
    pub what: AgreementChange,
}

/// A standing agreement (ruling 67), kept in the world's state by id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agreement {
    pub id: Id,
    pub asker: Id,
    pub holder: Id,
    pub work: Work,
    pub terms: Terms,
    pub formed_at: Tick,
    pub state: AgreementState,
    pub history: Vec<AgreementEvent>,
}

impl Agreement {
    pub fn standing(&self) -> bool {
        matches!(self.state, AgreementState::Standing)
    }
    pub fn party(&self, who: Id) -> bool {
        who == self.asker || who == self.holder
    }
    pub fn other(&self, who: Id) -> Id {
        if who == self.holder { self.asker } else { self.holder }
    }
    pub fn covers(&self, work: &Work) -> bool {
        work.craft == self.work.craft
            && work.grade <= self.work.grade
            && work.danger <= self.terms.danger_cap
    }
    pub fn exercises(&self) -> usize {
        let done = |e: &&AgreementEvent| e.what == AgreementChange::Exercised;
        self.history.iter().filter(done).count()
    }
}
