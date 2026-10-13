// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! A campaign reading of a native session, with native assertion admission.

use isocosm::{
    Command, Execution, Session, asserted::Assertion, history::Saved, simulation::Genesis,
};
use std::fmt;

mod commands;
mod contract;
pub use contract::ContractError;

mod borrowed;
pub use borrowed::{WindowReading, submit_contract_assertions};

mod characters;
pub use characters::{CharacterAdmission, RecipeBinding, admit_character};
mod placement;
pub use placement::{CharacterControl, CharacterPlacement, ControlBinding, ParticipantInput,
    PlacementWindow, PositionBinding,CharacterSpatialPlacement,SpatialSense};
pub mod eye;
mod game;
pub use game::{AdmissionBatch, GameBindings, submit_game, submit_game_as, submit_game_access,
    submit_game_access_with_roster, submit_external_table_access};
mod shared_roster;
pub use shared_roster::SharedRosterInput;
mod movement;
pub use movement::submit_table;
pub mod party;
pub mod joint_journey;
mod table;
pub mod time;
pub use time::submit_time;
pub mod overmap;
pub mod hooks;
pub use hooks::{goal_hooks, submit_hook};
pub mod pack;
pub mod storylets;
pub mod physical_map;
pub mod institutions;
pub mod items;
pub use items::{ProjectCarryPolicy,ProjectCarryConfig,ProjectCustodyWitness,ProjectLocation,
    submit_project_carry_config,current_project};
pub mod graft;

mod reading;
pub use reading::{Geography, Reading, polity_readings};

/// Where the native operation refused an input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Founding,
    /// Zero-based index in the caller's assertion batch.
    Assertion {
        index: usize,
    },
    Load,
    Command { index: usize },
    Advance,
}

/// Native refusal text, retained verbatim beside its calling context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayError {
    pub phase: Phase,
    pub reason: String,
}

impl fmt::Display for ReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.phase, self.reason)
    }
}

impl std::error::Error for ReplayError {}

/// An immutable campaign reading whose history and state belong to Isocosm.
#[derive(Clone, Debug)]
pub struct Replay {
    session: Session,
}

impl Replay {
    /// Replay the supplied assertions in order, publishing only a complete session.
    pub fn from_assertions(
        genesis: Genesis,
        assertions: &[Assertion],
        mode: Execution,
    ) -> Result<Self, ReplayError> {
        let session = Session::new(genesis, mode).map_err(|reason| ReplayError {
            phase: Phase::Founding,
            reason,
        })?;
        Self::apply(session, assertions)
    }

    /// Load through native replay, digest and witness validation.
    pub fn load(saved: Saved, mode: Execution) -> Result<Self, ReplayError> {
        Session::load(saved, mode)
            .map(|session| Self { session })
            .map_err(|reason| ReplayError {
                phase: Phase::Load,
                reason,
            })
    }

    /// Return a new reading with the whole batch accepted; retain this reading on refusal.
    pub fn with_assertions(&self, assertions: &[Assertion]) -> Result<Self, ReplayError> {
        Self::apply(self.session.clone(), assertions)
    }

    fn apply(mut session: Session, assertions: &[Assertion]) -> Result<Self, ReplayError> {
        for (index, assertion) in assertions.iter().enumerate() {
            session
                .command(Command::Assert(assertion.clone()))
                .map_err(|reason| ReplayError {
                    phase: Phase::Assertion { index },
                    reason,
                })?;
        }
        Ok(Self { session })
    }

    pub fn reading(&self) -> Reading<'_> {
        Reading::new(&self.session)
    }

    pub fn save(&self) -> Saved {
        self.session.save()
    }
}
