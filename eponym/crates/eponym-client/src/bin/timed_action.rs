// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native host entry point for the timed action client.

#[path = "timed_action/model.rs"]
mod model;

fn main() {
    model::run();
}
