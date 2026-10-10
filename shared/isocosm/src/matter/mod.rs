// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's typed matter (TG2), moved from its legacy core with the matter
//! and processes family: exact stocks over four material channels, declared
//! conversions and reconciled receipts. A stock column diffuses through
//! [`crate::diffusion`], the kernel ledgers share.

pub mod receipt;
pub mod stock;

pub use receipt::Conversion;
pub use stock::{Material, Stock, StockError};
