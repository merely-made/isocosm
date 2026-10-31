// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native item custody (stock/title plan T2): an inert item rides with its carrier
//! through journeys, burden counts carried items and projects, and titled stock keeps
//! its owners. Values are explicit test-world inputs, not production defaults.
#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;
#[path = "society_custody/world.rs"]
mod custody_world;
pub use custody_world::*;

#[path = "society_custody/carry.rs"]
mod carry;
#[path = "society_custody/refusal.rs"]
mod refusal;
#[path = "society_custody/title.rs"]
mod title;
