// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::{Phase, Replay, ReplayError};
use cosmograph::{Command, schema::Tick};

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
            let receipt = match command {
                Command::Act { .. } => serde_json::from_str::<cosmograph::simulation::Receipt>(&outcome).map(Some),
                Command::ActOwned { .. } => serde_json::from_str::<cosmograph::projects::OwnedActed>(&outcome)
                    .map(|acted| Some(acted.receipt)),
                _ => Ok(None),
            }.map_err(|error| ReplayError { phase:Phase::Command { index }, reason:error.to_string() })?;
            if let Some(receipt) = receipt {
                if let cosmograph::simulation::Outcome::Blocked(reason)
                    | cosmograph::simulation::Outcome::Refused(reason) = receipt.outcome
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
