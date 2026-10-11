// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::Replay;
use crate::contract::{self, Refusal};
use isocosm::Command;
use isocosm_overlay::vtt::Assertion;
use std::fmt;

/// Where a table assertion failed to enter the native session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContractError {
    Lowering { index: usize, refusal: Refusal },
    Native { index: usize, reason: String },
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lowering { index, refusal } => {
                write!(f, "assertion {index}: {refusal:?}")
            },
            Self::Native { index, reason } => write!(f, "assertion {index}: {reason}"),
        }
    }
}

impl std::error::Error for ContractError {}

impl Replay {
    /// Admit a trusted caller's whole table batch through the contract.
    pub fn with_contract_assertions(&self, assertions: &[Assertion]) -> Result<Self, ContractError> {
        let mut session = self.session.clone();
        for (index, assertion) in assertions.iter().enumerate() {
            let entry = contract::lower(assertion, &session.sim)
                .map_err(|refusal| ContractError::Lowering { index, refusal })?;
            session
                .command(Command::Assert(entry))
                .map_err(|reason| ContractError::Native { index, reason })?;
        }
        Ok(Self { session })
    }
}
