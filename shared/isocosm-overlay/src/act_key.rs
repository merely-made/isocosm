// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// An act named by opaque key. The vocabulary is a ruleset's, the way the
/// sim names processes by string key (`isocosm::schema::Key`, not depended
/// on here): a critter's act toward a nudge's target in Mesocosm, a timed act
/// of the body in Eponym, an adjudicated action's key at the VTT's table.
/// Lifted to the core at Mark's word on 2026-09-26 from the three game
/// modules that had each defined it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ActKey(pub String);
