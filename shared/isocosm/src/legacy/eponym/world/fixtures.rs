// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Authored test/demo data, not production generation or world authority.
//!
//! The three lives are deliberately public so native inspectors and examples
//! consume one authored source. They remain a fixture and must not become a
//! profession registry or a second body store.
//!
//! [`session`] is the seed-7 timed-action world the client's hosts and this
//! crate's glyph tests share. It is public and unconditional for the same
//! reason: one authored source, reachable from every consumer.

pub mod session;
pub mod three_lives;
