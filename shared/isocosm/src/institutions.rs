// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Constitutional operations use actual bodies; attribution keeps polity identity separate.

use crate::{Result, schema::*, simulation::Simulation};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod types;
mod policy;
mod admission;
mod decision;
mod reading;
pub use types::*;
pub use reading::{InstitutionReading, OwnAct};
pub use policy::validate;
pub(crate) use decision::{DecisionContext, DecisionStamp};

impl Simulation {
    pub(crate) fn institution_control(&self, participant: Id, actor: Id) -> Result<()> {
        if !self.state.population.get(participant).is_some_and(crate::directing::is_participant)
            || self.plays(participant) != Some(actor) {
            return Err("institution input requires actual native control of the acting body".into());
        }
        if !self.state.population.get(actor).is_some_and(|body| body.alive && !crate::directing::is_participant(body)) {
            return Err("institution input has no available native acting body".into());
        }
        Ok(())
    }
}
