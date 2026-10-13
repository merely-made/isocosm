// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Player-authored native membership and explicit disclosed map contributions.

use crate::{EntityHandle, EventHandle, PlaceHandle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartyIntent {
    pub party: String,
    pub member: EntityHandle,
    pub action: PartyAction,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartyAction {
    Join,
    Leave,
    Disclose { sites: Vec<PlaceHandle>, events: Vec<EventHandle> },
}
