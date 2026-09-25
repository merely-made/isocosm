// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// Mesocosm's handoff outcome vocabulary: empty (overlay plan §3). Reading,
/// not ruled: Mesocosm has no ruleset beside the sim — metabolism,
/// incorporation and contests are the sim's own processes, and the shop is a
/// revision the sim admits at the boundary — so nothing reaches the handoff
/// unless a later rule adds a game-resolved outcome. An uninhabited enum
/// makes that checked rather than only asserted: no value of this type can
/// exist, so [`crate::mesocosm::MesocosmHandoffEnvelope`] can never be built.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MesocosmHandoff {}
