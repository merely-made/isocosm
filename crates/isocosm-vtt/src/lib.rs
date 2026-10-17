// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Isocosm: VTT game layer over the Isocosm simulator.

pub mod contract;
pub mod founding;
pub mod native;

#[cfg(feature = "endpoint")]
#[path = "endpoint.rs"]
mod endpoint;
#[cfg(feature = "endpoint")]
pub use endpoint::*;
