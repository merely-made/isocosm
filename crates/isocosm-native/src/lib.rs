// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Isocosm game layer over the Cosmograph simulator.

pub mod contract;
pub mod founding;
pub mod native;

#[cfg(feature = "endpoint")]
pub mod ruleset;

#[cfg(feature = "endpoint")]
#[path = "endpoint.rs"]
mod endpoint;
#[cfg(feature = "endpoint")]
pub use endpoint::*;
