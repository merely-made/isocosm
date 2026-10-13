// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native territory and surface stocks, conservative travel and local reads.
//! Capacity reads authored trait mechanics and ordered modifier layers;
//! route overflow divides excess equally before transmission. It never substitutes a part tree or a second ledger
//! for matter already on a site, finer PlaceId, or host.

pub use crate::schema::spread::{CapacityModifier, CapacitySource, Identity, PlaceLedger,
    Spreads, Substrate, Transfer, TransitId, TransitMark};

mod stock;
mod transport;
mod strain;
mod native;
mod capacity;
mod owned;
mod placement;
pub use placement::{Placement, Supply, place_stock};

pub use stock::{functions, held, ledger_mass, provenance};
pub(crate) use stock::validate;
pub use native::Command;
pub use capacity::{CapacityLayer, CapacityRules, Stacking, capacity};
pub use transport::{Dispatch, dispatch_changed, overflow, overflow_at, resolved_overflow,
    resolved_overflow_at, settle};
pub use strain::{fork_strain, strain_key};

#[cfg(test)]
mod tests;
