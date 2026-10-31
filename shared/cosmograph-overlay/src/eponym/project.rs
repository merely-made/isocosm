// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Material work uses the world's named process and an existing native project.

use crate::{ActKey, EntityHandle, FactionHandle};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectHandle(pub u64);

/// Reservation and work share native process admission. Selecting a project
/// here requests resumption; it grants neither access nor craft eligibility.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectAct {
    pub subject: EntityHandle,
    pub process: ActKey,
    pub target: Option<EntityHandle>,
    pub project: Option<ProjectHandle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lots: Vec<String>,
}

/// Carrying requests native custody; the game pairs it with accepted position
/// and actual burden capacity before publishing either side (840).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectCarry {
    pub subject: EntityHandle,
    pub project: ProjectHandle,
    pub carried: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectOwner { Entity(EntityHandle), Polity(FactionHandle) }

/// Each genuine input owner supplies its own native decision (839/844).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConsent {
    pub subject: EntityHandle,
    pub project: ProjectHandle,
    pub owner: ProjectOwner,
    pub worker: EntityHandle,
    pub grant: bool,
    pub process: Option<ActKey>,
}
