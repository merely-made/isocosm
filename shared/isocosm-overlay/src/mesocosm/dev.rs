// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

use crate::{EntityHandle, PlaceHandle};

/// A dev tool, never play (dev tools plan §2). Applied, refused and recorded
/// like any other intent; a receipt labels a run that used one as assisted.
/// `PlaceMatter` names a site by its place handle, as a nudge names a place
/// (rulings 205 and 784).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevIntent {
    EndEpoch,
    ForceBirth { organism: EntityHandle },
    Kill { organism: EntityHandle },
    PlaceMatter { site: PlaceHandle, mass_mg: u64 },
}
