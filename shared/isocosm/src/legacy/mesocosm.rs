// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What is left of Mesocosm's legacy core after its world moved onto native
//! Isocosm (wing rulings 681 and 755): the in-site places Eponym's legacy
//! world still stands on, waiting for the places family (670), the codec
//! its saves use, and the body re-exports at the paths Eponym names them by.
//! Each leaves with Eponym's world move.

pub mod places;
pub mod snapshot;

/// The seeded stream, native since the world move.
pub use crate::rng;

pub use crate::lineage::LineageBody as BodyDocument;
pub use crate::lineage::{Lineal, Origin, Provenance, SpeciesId};
pub use isometer_core::{Aabb, AttachError, Attachment, Part, PartId, VolumeRef, Yaw};
pub use isometer_core::{BodyPlan, Facing, Role, Symmetry, classify};
pub use isometer_core::{WireError, anatomy, frame, plan, unframe, wire};

/// isometer-core's body module with the lineage it no longer holds (756).
pub mod body {
    pub use crate::lineage::LineageBody as BodyDocument;
    pub use crate::lineage::{Lineal, Origin, Provenance, SpeciesId};
    pub use isometer_core::body::*;
}

pub use places::{Place, PlaceId, Places};
pub use rng::Rng;
