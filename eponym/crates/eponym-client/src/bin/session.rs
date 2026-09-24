// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native entry point for the Eponym document host.
//!
//! `session` is a working name; the real one is a later naming round.

#[path = "session/model.rs"]
mod model;

fn main() {
    std::process::exit(model::run());
}
