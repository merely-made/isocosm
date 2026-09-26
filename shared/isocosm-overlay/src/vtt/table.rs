// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{ActKey, EntityHandle, PlaceHandle, WorldPoint};

/// The table's acts for one tick: one batch, never a call per token (the
/// record's §5.2 point 1). Stances and emotes are not here; they stay the
/// table's, a beat being representational by the protocol's own word (VTT
/// overlay plan §3, a reading).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableBatch {
    pub acts: Vec<TableAct>,
}

/// One act at the table, by a character a participant plays: a player's own
/// (ruling 152), two players able to share one, or any unclaimed one the DM
/// takes up, both on by default (rulings 153, 156, 245).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableAct {
    pub actor: EntityHandle,
    pub kind: TableActKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableActKind {
    /// An adjudicated action the ruleset resolves; its outcome returns
    /// through the handoff (rulings 114, 154). `isometry-system`'s targeted
    /// actions.
    Act {
        action: ActKey,
        target: Option<EntityHandle>,
    },
    /// A move within the battlemap, so the character's place in the site
    /// follows its token (ruling 244). `SessionEvent::TokenMoved`.
    Move { to: Cell },
}

/// A position in a site's volume, in the sim's voxel grid; the table's tile
/// is the game's grid projected over that volume (ruling 18), and the
/// battlemap it draws is projected from the generated volume with the DM's
/// map an edit over it (ruling 243).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// The site, a place-graph node (ruling 205).
    pub site: PlaceHandle,
    pub at: WorldPoint,
}
