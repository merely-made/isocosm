// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native party membership and disclosed maps (stock/title plan T3; rulings 853, 854, 857, 858).
//! Values are explicit test-world inputs, not production defaults.
#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;
#[path = "society_party/world.rs"]
mod party_world;
pub use party_world::*;

#[path = "society_party/membership.rs"]
mod membership;
#[path = "society_party/disclosure.rs"]
mod disclosure;
