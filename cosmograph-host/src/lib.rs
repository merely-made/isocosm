// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The embeddable sim host (rulings 884 and 892): one native world, its
//! per-mode sections and saves, and the window attention bound to it. No
//! product crate is in this library's graph; products depend on it.

pub mod access;
pub mod binding;
pub mod drive;
#[cfg(feature = "endpoint")]
pub mod endpoint;
pub mod file_save;
pub mod founding;
pub mod native_attention;
pub mod save;
pub mod spatial;
pub mod stream;
pub mod window;
pub mod world;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;
