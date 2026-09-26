// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use super::table::{Cell, WorldPoint};
use crate::{FactionHandle, ParticipantHandle, PlaceHandle, Pointable};

/// What the DM asserts from edit mode, which is the world editor's (rulings
/// 89, 156, 184): each an asserted fact in the record (the record's §1),
/// written whether the sim is on or off, so a sim-off campaign can switch it
/// on later over the same history (ruling 248). Host-committed at the table,
/// as `isonetry`'s `Fact`, `World`, `MapStored` and `CharacterCreated` are.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Assertion {
    /// A fact committed to the campaign's journal, or a secret revealed
    /// (rulings 80, 87, 156): a note on the thing it concerns.
    Fact(Fact),
    /// The DM's map as an edit over the site's generated volume (rulings 89,
    /// 243): tile kinds and heights per cell, in the open tile vocabulary.
    Edit(MapEdit),
    /// A character created (ruling 36): a denizen with a faction association,
    /// member or outsider, where they start being the DM's setup.
    Character(NewCharacter),
    /// A storylet's effects applied, its facts asserted together (the
    /// record's §3.9).
    Storylet {
        storylet: String,
        asserts: Vec<Assertion>,
    },
    /// A pack forced over generated content where the world did not meet its
    /// requirements (ruling 190).
    PackForced {
        pack: String,
        asserts: Vec<Assertion>,
    },
}

/// A public campaign fact: its kind in the table's vocabulary (`reveal`,
/// `narration`, `history`), what it concerns, its text and tags. The shape
/// of `isometry-campaign`'s `WorldFact` with its subject a handle.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    pub kind: String,
    pub about: Option<Pointable>,
    pub text: String,
    pub tags: Vec<String>,
}

/// The DM's edits to one site's volume.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapEdit {
    pub site: PlaceHandle,
    pub cells: Vec<CellEdit>,
}

/// One cell of the DM's map: its tile kind, from the open tile vocabulary,
/// and its height in the game's steps (`SessionEvent::TilePlaced` and
/// `ElevationSet`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellEdit {
    pub at: WorldPoint,
    pub kind: String,
    pub height: u32,
}

/// A character created at the table (`isonetry`'s `CharacterCreated`): a
/// name, a faction if any, where it starts, and the participant who owns it,
/// none meaning the DM's.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewCharacter {
    pub name: String,
    pub faction: Option<FactionHandle>,
    pub at: Cell,
    pub owner: Option<ParticipantHandle>,
}
