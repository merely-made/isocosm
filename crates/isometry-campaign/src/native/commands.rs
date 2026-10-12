// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{Phase, Replay, ReplayError};
use isocosm::{Command, schema::Tick};

impl Replay {
    /// Admit ordered native commands and advance on a private copy.
    pub fn with_commands(
        &self,
        commands: &[Command],
        ticks: Tick,
    ) -> Result<Self, ReplayError> {
        let mut session = self.session.clone();
        for (index, command) in commands.iter().enumerate() {
            let outcome = session.command(command.clone()).map_err(|reason| ReplayError {
                phase: Phase::Command { index },
                reason,
            })?;
            if matches!(command, Command::Act { .. }) {
                let receipt: isocosm::simulation::Receipt = serde_json::from_str(&outcome)
                    .map_err(|e| ReplayError {
                        phase: Phase::Command { index }, reason: e.to_string(),
                    })?;
                if let isocosm::simulation::Outcome::Blocked(reason)
                    | isocosm::simulation::Outcome::Refused(reason) = receipt.outcome
                {
                    return Err(ReplayError { phase: Phase::Command { index }, reason });
                }
            }
        }
        session.advance(ticks).map_err(|reason| ReplayError {
            phase: Phase::Advance,
            reason,
        })?;
        Ok(Self { session })
    }
}