// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Chosen native steps and clocked arrival placement, resolved by a game (841–842).

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use crate::{Result, rules::{Amount, Causation, Rules}, schema::*, Simulation};
use isometer_core::Yaw;
use isometer_space::places::{Body, PlaceId};
use crate::operations::Window;

mod geometry;
mod site_geometry;
pub use site_geometry::SiteGeometry;
pub(crate) use site_geometry::Cache as GeometryCache;
mod advance;
mod arrival;
mod birth;
mod birth_fallback;
pub use birth_fallback::BirthFallback;
mod birth_geometry;
mod birth_command;
mod templates;
mod tier;
pub mod embodiment;
pub mod foraging;
pub use templates::{Templates, Stand, Crossing, STANDS as STANDS_PER_SITE};
pub use birth::{BirthProfile, BirthMedium, BirthSupport, BirthRequest, BirthGeometry, BirthResolution, BirthReport, BirthPending, birth_pose};
pub use birth::rotate as rotate_offset;
pub use birth_command::{BirthPreparation, PreparedBirth};
pub(crate) use birth::BirthMark;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Publication { Step(Key), Birth(Id) }
pub use advance::{Preparation, Prepared, SpatialAdvance};
pub use geometry::{check_step, neighbours};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepProfile {
    pub process: Key,
    pub cells: Amount,
    pub places: isometer_space::places::Rules,
    pub body: Body,
    /// Root pivot to walking stance, in this site's base-cell axes.
    pub stance_offset: [i64; 3],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pose {
    pub at: [i64; 3], pub yaw: Yaw,
    /// Admitted sole centres in body-local fixed-point voxel units (1024).
    /// Empty preserves the rigid contract of existing saved worlds.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub feet: BTreeMap<isometer_core::PartId, [i64; 3]>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub action: Key, pub process: Key, pub profile: Key,
    pub tick: Tick, pub revision: Key, pub actor: Id, pub target: Option<Id>,
    pub site: Id, pub patch: PlaceId, pub body_revision: u64,
    pub terrain_revision: Key, pub cells: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geometry {
    pub window: Window, pub path: Vec<Pose>, pub eye: [i64; 3],
    /// One explicit gait per adjacent edge; empty preserves recorded legacy steps.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strides: Vec<Option<isometer_core::legs::Stride>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Resolution { pub request: Request, pub geometry: std::result::Result<Geometry, String> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerrainResolution {
    pub request: crate::operations::Request,
    pub geometry: std::result::Result<crate::operations::Geometry, String>,
}

#[derive(Clone, Debug)]
pub struct Pending { pub request: Request, pub sim: Box<Simulation> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalRequest {
    pub actor: Id, pub from: Id, pub to: Id, pub departed: Tick, pub arrives: Tick,
    pub body_revision: u64, pub terrain_revision: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalGeometry {
    pub window: Window, pub pose: Pose, pub eye: [i64; 3],
    pub places: isometer_space::places::Rules,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalResolution { pub request: ArrivalRequest, pub geometry: ArrivalGeometry }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrivalReport { pub actor: Id, pub patch: std::result::Result<PlaceId, String> }

#[derive(Clone, Debug)]
pub(crate) struct StagedStep { pub request: Request, pub resolved: bool }

#[derive(Clone, Debug, Default)]
pub(crate) struct Resolutions {
    pub(crate) inputs: BTreeMap<Key, Resolution>, used: BTreeSet<Key>,
    accepted: BTreeSet<Key>,
    pub pending: Option<Pending>,
    arrivals: BTreeMap<Id, ArrivalResolution>, arrived: BTreeSet<Id>,
    reports: Vec<ArrivalReport>, enabled: bool,
    births: BTreeMap<Id, BirthResolution>, born: BTreeSet<Id>,
    birth_reports: Vec<BirthReport>, birth_enabled: bool,
    birth_fallback: Option<BirthFallback>, auto_births: Vec<BirthResolution>,
    order: Vec<Publication>,
    pub birth_pending: Option<BirthPending>,
    command_outcome: Option<String>,
    work: Option<crate::simulation::Work>,
}

pub fn validate(rules: &Rules) -> Result<()> {
    for (key, profile) in &rules.birth_profiles {
        crate::validation::key(key)?;
        if !rules.processes.contains_key(&profile.process)
            || (0..3).any(|k| profile.clearance[0][k] >= profile.clearance[1][k]) || profile.places.climb.run == 0
            || matches!(profile.places.cap, isometer_space::places::Cap::Own(s) if s.contains(&0))
        { return Err("birth profile needs its native process and authored clearance".into()); }
        if profile.support == BirthSupport::BodyGround && (profile.medium != BirthMedium::Air
            || rules.spatial_templates.as_ref().is_none_or(|t| t.version < 2)) {
            return Err("part-floor birth support needs whole-body templates and air".into());
        }
    }
    for (key, profile) in &rules.step_profiles {
        crate::validation::key(key)?;
        let process = rules.processes.get(&profile.process).ok_or("step process absent")?;
        profile.cells.validate()?;
        if key.is_empty() || process.causation != Causation::Choice || process.period.is_none()
            || profile.cells == Amount::Fixed(0) || profile.body.width == 0
            || profile.body.height == 0 || profile.places.climb.run == 0
            || matches!(profile.places.cap, isometer_space::places::Cap::Own(s) if s.contains(&0))
        { return Err("step profile needs an authored chosen process, budget and clearance".into()); }
    }
    Ok(())
}

impl Simulation {
    pub(crate) fn capture_step(&mut self, step: &Option<StagedStep>) {
        let Some(step) = step else { return; };
        if step.resolved {
            self.spatial_resolutions.used.insert(step.request.action.clone());
            return;
        }
        if self.spatial_resolutions.pending.is_some() { return; }
        let mut sim = self.clone();
        sim.clear_owned_capture();
        sim.spatial_resolutions = Resolutions { accepted: self.spatial_resolutions.accepted.clone(),
            reports: self.spatial_resolutions.reports.clone(),
            birth_reports: self.spatial_resolutions.birth_reports.clone(), ..Default::default() };
        sim.spatial_resolutions.order = self.spatial_resolutions.order.clone();
        sim.geometry_resolutions = crate::operations::Resolutions::default();
        sim.journal = None;
        self.spatial_resolutions.pending = Some(Pending { request: step.request.clone(), sim: Box::new(sim) });
    }
    pub(crate) fn commit_step(&mut self, step: Option<StagedStep>) {
        if let Some(step) = step {
            debug_assert!(step.resolved);
            if self.foraging_enabled() {
                let pose = self.spatial_resolutions.inputs[&step.request.action].geometry.as_ref()
                    .expect("accepted geometry").path.last().expect("accepted path").clone();
                self.keep_spatial_pose(step.request.actor, step.request.site, pose);
            }
            self.spatial_resolutions.accepted.insert(step.request.action.clone());
            self.spatial_resolutions.order.push(Publication::Step(step.request.action.clone()));
            self.spatial_resolutions.used.insert(step.request.action);
        }
    }
    pub fn spatial_steps(&self) -> &BTreeSet<Key> { &self.spatial_resolutions.accepted }
    pub fn spatial_arrivals(&self) -> &[ArrivalReport] { &self.spatial_resolutions.reports }
    pub fn spatial_order(&self) -> &[Publication] { &self.spatial_resolutions.order }
}
