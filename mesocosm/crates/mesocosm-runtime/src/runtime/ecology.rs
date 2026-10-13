// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit generated-world offer gate. Structural generation is only a candidate.

use super::{Founded, Runtime};
use crate::spatial::{Game, Setup as SpatialSetup};
use isocosm::{Execution, ecology::{JointSpec, JointWitness, run_joint}};
use serde::{Deserialize, Serialize};

mod offer;
mod seal;
pub use seal::Seal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Setup {
    pub version: u32,
    /// Exact generated candidate, including authored role declarations and lifecycle settings.
    pub founded: Founded,
    pub specification: JointSpec,
    pub execution: Execution,
    /// Actual physical inputs; absent means only ordinary native site-grain execution.
    pub spatial: Option<SpatialSetup>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Assessment {
    pub witness: JointWitness,
    pub spatial: Game,
}

impl Setup {
    pub fn check(&self) -> Result<(), String> {
        if self.version != 1 || self.founded.genesis.founding.is_none() {
            return Err("ecology offer setup must name an exact generated candidate at version 1".into());
        }
        if self.founded.pace.round == 0 || self.founded.pace.scoring == 0 {
            return Err("ecology offer requires explicitly positive lifecycle pace".into());
        }
        self.founded.genesis.validate()?;
        self.specification.validate_genesis(&self.founded.genesis)?;
        if self.founded.genesis.rules.ecology_witness.as_ref() != Some(&self.specification) {
            return Err("ecology offer and native rule specifications differ".into());
        }
        Ok(())
    }

    /// Runs the actual supplied clock provider over one native trajectory.
    /// The same trajectory supplies the native evidence required before admission.
    pub fn assess(&self) -> Result<Assessment, String> {
        self.check()?;
        let mut game = Game::default();
        let mut initialized = false;
        let witness = run_joint(&self.founded.genesis, &self.specification, self.execution, &mut |session| {
            if !initialized {
                if let Some(setup) = &self.spatial { game.setup(session, setup)?; }
                initialized = true;
            }
            if self.spatial.is_some() { game.advance_clock(session) }
            else { session.advance_tick_with_flows() }
        })?;
        if self.spatial.is_some() {
            let native = isocosm::Session::load(witness.native.clone(), self.execution)?;
            game.validate_replay(&native)?;
        }
        Ok(Assessment { witness, spatial: game })
    }

    pub fn offer(&self, ticks_per_second: u32) -> Result<Runtime, String> {
        let founding=self.founded.genesis.founding.as_ref().ok_or("generated candidate has no founding")?;
        // This producer captures its real basis; authored JSON cannot replace it.
        let basis=isocosm::ecology::GeneratedBasis::new(founding)?;
        let criterion=self.specification.independence.as_ref()
            .ok_or("generated-world offer needs explicit native mechanical-effect declarations879")?;
        let assessment = self.assess()?;
        let admission=isocosm::ecology::admit(&assessment.witness,criterion,&basis)?;
        offer::accepted(self,assessment,admission,ticks_per_second)
    }
}

/// An absent marker is compatible for saved loads, but not a fresh generated offer.
pub(super) fn fresh_offer(founded: &Founded) -> Result<(), String> {
    if founded.genesis.founding.is_some() {
        return Err(if founded.genesis.rules.ecology_witness.is_none() {
            "new generated world requires an explicit joint ecology witness before it can be offered".into()
        } else {
            "generated world must pass its joint witness and native mechanical-effect admission879".into()
        });
    }
    Ok(())
}
