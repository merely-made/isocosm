// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, LineageHandle, Tick};

/// An answer to one of Eponym's two checkpoints: the first life, at founding
/// (rulings 234, 235), and a death (rulings 61, 186, 238).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeCheckpoint {
    First(FirstLife),
    Death(Succession),
}

/// The first life: when in the world's time it begins and how, both the
/// player's pick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FirstLife {
    pub start: StartTime,
    pub begins_as: LifeChoice,
}

/// Where in the world's time a first life begins (ruling 234): from the
/// world's habitability for the creature onward, once settlements stand,
/// which is the default, or at a tick of the player's own choosing. Eponym
/// needs sapient people and society (ruling 3), which is why society is the
/// default.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartTime {
    Habitability,
    #[default]
    Society,
    At(Tick),
}

/// How a life begins (ruling 235), any of three at the player's call: a
/// newly generated outsider arriving, a birth into one of the world's
/// lineages, or a named creature of the drawn world taken up.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeChoice {
    Outsider,
    Birth { lineage: LineageHandle },
    Denizen(EntityHandle),
}

/// At a death: the one who died and who the player becomes. Death is final
/// (ruling 61); the body and its consequences stay in the world, and the
/// world keeps what it remembers of the dead (ruling 129).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Succession {
    pub of: EntityHandle,
    pub next: Successor,
}

/// Who the player becomes (rulings 186, 238): a companion with a bond to the
/// dead; with none, another life in the same world, any of ruling 235's
/// three; or a new world, which is how a line ends in this one, a reading
/// recorded with ruling 238. No answer lets the line end otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Successor {
    Companion(EntityHandle),
    Another(LifeChoice),
    NewWorld,
}
