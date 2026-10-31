// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native goal notes and their derived threads (stock/title plan T4; ruling 866): an accepted act
//! records participation in every goal it actually served. Values are explicit test-world inputs.
#![cfg(feature = "society")]

#[path = "owned_stock/fixture.rs"]
mod fixture;
pub use fixture::*;
#[path = "society_goals/world.rs"]
mod goal_world;
pub use goal_world::*;

#[path = "society_goals/directive.rs"]
mod directive;
#[path = "society_goals/command.rs"]
mod command;
