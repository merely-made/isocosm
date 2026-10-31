// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, ParticipantHandle};

/// Knowledge access is independent of the active product mode (445).
/// A seat describes access; carrying this value grants no editing authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Seat {
    Player,
    CreativeObserver,
    Dm,
}

/// The ordinary access identity resolved by a trusted native adapter.
/// Divine awareness remains parked (384); this value carries no such grant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdinaryAccess {
    pub participant: ParticipantHandle,
    pub seat: Seat,
    pub subject: Option<EntityHandle>,
}
