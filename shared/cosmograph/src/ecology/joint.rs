// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored roles and exact accepted evidence from one native trajectory.

use crate::{history::Saved, schema::*, simulation::Execution};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JointSpec {
    pub version: u32,
    pub key: Key,
    pub horizon: Tick,
    pub capture: CaptureRequirement,
    pub returned: ReturnRequirement,
    pub reproduction: BirthRequirement,
    pub counterplay: CounterRequirement,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub independence: Option<super::IndependenceSpec>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub process: Key,
    pub actor_lineage: Key,
    /// None requires an actual untargeted act; it is not a wildcard.
    pub target_lineage: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptureStock { EnergyAccount(Key), OwnReserve(Key) }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureRequirement {
    pub selection: Selection,
    pub stock: CaptureStock,
    pub at_least: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReturnRequirement {
    pub selection: Selection,
    pub account: Key,
    pub at_least: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthRequirement { pub selection: Selection, pub at_least: u64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CounterRequirement { pub selection: Selection, pub proof: CounterProof }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CounterProof {
    LivingCellsLost { at_least: u64 },
    TargetDied,
    TargetDebited { account: Key, at_least: u64 },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credit {
    pub holder: crate::flows::Holder,
    pub account: Key,
    /// Per member, from the successful native primitive, not optional flows.
    pub amount: u64,
    /// Excludes same-holder/account recirculation within this exact native effect.
    pub fresh: u64,
    pub parts: Vec<(PartId, u64)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiologicalBirth { pub child: Id, pub lineage: Key, pub matter: u128 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionProof {
    pub action: Key,
    pub tick: Tick,
    pub process: Key,
    pub actor: Id,
    pub actor_lineage: Key,
    pub target: Option<Id>,
    pub target_lineage: Option<Key>,
    pub count: u64,
    pub site: Option<Id>,
    pub credits: Vec<Credit>,
    pub births: Vec<BiologicalBirth>,
    /// Actual current target before this publication and after it, not a pass forecast.
    pub counter: u64,
}

/// Quantities accumulated over the one declared horizon. No ecology standard is inferred.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JointTotals { pub captured: u128, pub returned: u128, pub births: u128, pub counter: u128 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JointWitness {
    pub version: u32,
    pub spec: JointSpec,
    pub execution: Execution,
    pub native: Saved,
    pub acts: Vec<ActionProof>,
    pub totals: JointTotals,
}

impl Selection {
    pub(crate) fn matches(&self, act: &ActionProof) -> bool {
        self.process == act.process && self.actor_lineage == act.actor_lineage
            && self.target_lineage == act.target_lineage
    }
}

impl JointSpec {
    pub(crate) fn observes(&self, process: &str) -> bool {
        self.selections().iter().any(|role| role.process == process)
            || self.independence.as_ref().is_some_and(|spec|
                spec.extras.iter().any(|extra| extra.consumer.process == process))
    }

    pub(crate) fn selects(&self, act: &ActionProof) -> bool {
        self.selections().iter().any(|role| role.matches(act))
            || self.independence.as_ref().is_some_and(|spec|
                spec.extras.iter().any(|extra| extra.consumer.matches(act)))
    }

    pub(crate) fn selections(&self) -> [&Selection; 4] {
        [&self.capture.selection, &self.returned.selection,
            &self.reproduction.selection, &self.counterplay.selection]
    }
}
