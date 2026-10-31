// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The ladder's readings (doctrine World 10, Naming 2; cases 1023 to 1027):
//! named individuals, their kin and relations, households, a place's tier
//! and the factions and polities acting through its residents. Each is read
//! from the state on demand and never stored (Schema 12).

use super::*;
use crate::{Result, simulation::Simulation};
use std::collections::BTreeSet;

mod denizens;
mod households;
mod places;
mod tiers;
#[cfg(test)]
mod tests;

pub use denizens::{CHILD, Denizen, PARENT, Tie};
pub use households::Household;
pub use places::{Group, PlaceReading};
pub use tiers::{Tier, Tiers, tiers};
