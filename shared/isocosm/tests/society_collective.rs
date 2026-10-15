// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A polity owns stock and grants project work under one authored strategy (stock/title plan T5;
//! rulings 839, 844): governance decision, current-member unanimity or declared representatives.
//! Values are explicit test-world inputs, not production defaults.
#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;
#[path = "society_institutions/world.rs"]
mod charter_world;
pub use charter_world::*;
#[path = "society_collective/world.rs"]
mod collective_world;
pub use collective_world::*;

#[path = "society_collective/strategy.rs"]
mod strategy;
#[path = "society_collective/assertion.rs"]
mod assertion;
