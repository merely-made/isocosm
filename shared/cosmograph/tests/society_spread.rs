// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Titles through spread placement (stock/title plan T6; rulings 846, 849, 850): owned site stock
//! placed into a native identity keeps exact claims. Overflow delivery is C11's own test.
//! Values are explicit test-world inputs, not production defaults.
#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;
#[path = "society_spread/world.rs"]
mod spread_world;
pub use spread_world::*;

#[path = "society_spread/placement.rs"]
mod placement;
