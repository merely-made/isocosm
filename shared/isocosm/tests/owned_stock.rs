// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native owned stock and projects (rulings 837 to 850), under the stock/title
//! plan. Values are explicit test-world inputs, not production defaults.
//! `owned_stock/controls.py` plants the source faults these tests must catch.

#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;

#[path = "owned_stock/ownership.rs"]
mod ownership;
#[path = "owned_stock/work.rs"]
mod work;
#[path = "owned_stock/consent.rs"]
mod consent;
#[path = "owned_stock/placement.rs"]
mod placement;
#[path = "owned_stock/debit.rs"]
mod debit;
