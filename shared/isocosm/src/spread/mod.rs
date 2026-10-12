// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native territory and surface stocks, conservative travel and local reads.
//! Capacity, automatic routing and mutation-trigger policies stay caller-owned
//! until ruled. This module never substitutes a part tree or a second ledger
//! for matter already on a site, finer PlaceId, or host.

pub use crate::schema::spread::{Identity, PlaceLedger, Spreads, Substrate, Transfer};

mod stock;
mod transport;
mod strain;
mod native;

pub use stock::{functions, held, ledger_mass, provenance};
pub(crate) use stock::validate;
pub use native::Command;
pub use transport::{Dispatch, overflow, settle};
pub use strain::{fork_strain, strain_key};

#[cfg(test)]
mod tests;
