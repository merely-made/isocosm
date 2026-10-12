// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

impl Session {
    /// Consumer legacy replay pins its movement mode before any action or clock.
    pub fn new_with_movement(genesis: Genesis, mode: Execution, movement: Movement) -> Result<Self> {
        let mut session = Self::new(genesis, mode)?;
        session.sim.state.movement = movement;
        Ok(session)
    }
}
