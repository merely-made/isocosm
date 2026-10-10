// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The identity a game keeps for its played subjects (wing rulings 152 and
//! 755): a subject is the game's handle on a sim entity, a body revision
//! moves when a severing changes the body (717), and control is who the
//! player is being, the overlay's `played`. Faction membership and offices
//! are the sim's, authored polities and their relations (760, 769).

pub mod control;

pub use control::{Control, ControlIntent};

use serde::{Deserialize, Serialize};

/// The continuing person, across bodies and control.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SubjectId(pub u64);

/// Which body, at which revision.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BodyRevisionId(pub u64);

/// The sim's clock, as the game stamps its intents.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Tick(pub u64);

impl Tick {
    pub fn after(self, ticks: u64) -> Self {
        Self(self.0.saturating_add(ticks))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityError {
    /// A control log began twice.
    AlreadyBegun,
    /// A control log that does not open with [`ControlIntent::Begin`].
    NotBegun,
    /// Tag-in while already tagged in, which would be two bodies at once.
    AlreadyTaggedIn,
    /// Tag-out while not tagged in.
    NotTaggedIn,
}
