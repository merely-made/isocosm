// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native institutions and polities (stock/title plan T4; rulings 844, 850, 867): constitutional
//! operations by actual bodies, support, amendment, dissolution and the readings of a polity.
//! Values are explicit test-world inputs, not production defaults.
#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;
#[path = "society_institutions/world.rs"]
mod charter_world;
pub use charter_world::*;

#[path = "society_institutions/operation.rs"]
mod operation;
#[path = "society_institutions/constitution.rs"]
mod constitution;
#[path = "society_institutions/polity.rs"]
mod polity;
