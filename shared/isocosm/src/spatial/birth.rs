// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Physical placement follows an accepted biological birth, never material output (848).

use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BirthProfile {
    pub process: Key, pub places: isometer_space::places::Rules,
    /// Half-open root-local box, including explicitly authored clearance.
    pub clearance: [[i64; 3]; 2], pub medium: BirthMedium, pub support: BirthSupport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BirthMedium { Air, Water, AirOrWater }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// Ground support is an authored child-root-local offset, rotated with its yaw.
pub enum BirthSupport {
    Free, Ground { stance_offset: [i64; 3] },
    /// Ground contact by an occupied part floor in a whole-body world.
    BodyGround,
}

#[derive(Clone, Debug)]
pub(crate) struct BirthMark {
    pub action: Key, pub process: Key, pub parent: Id, pub child: Id,
    pub ordinal: u64, pub parent_alive: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthRequest {
    pub action: Key, pub process: Key, pub parent: Id, pub child: Id, pub ordinal: u64,
    pub tick: Tick, pub revision: Key, pub parent_lineage: Key, pub child_lineage: Key,
    pub parent_alive: bool, pub parent_revision: u64, pub parent_visit: Tick,
    pub site: Option<Id>, pub parent_patch: Option<PlaceId>,
    pub child_revision: u64, pub child_visit: Tick, pub child_journey: Option<Journey>,
    pub terrain_revision: Key,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthGeometry {
    pub profile: Key, pub parent_pose: Pose, pub parent_eye: [i64; 3],
    pub parent_window: Window, pub parent_places: isometer_space::places::Rules,
    /// Relative to the parent's current yaw, not an invented world offset.
    pub offset_from_parent: [i64; 3], pub yaw_from_parent: Yaw,
    /// Relative to the child's resulting yaw.
    pub eye_from_child: [i64; 3], pub window: Window,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthResolution { pub request: BirthRequest, pub geometry: std::result::Result<BirthGeometry, String> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthReport {
    pub request: BirthRequest, pub profile: Option<Key>, pub patch: std::result::Result<PlaceId, String>,
}

#[derive(Clone, Debug)]
pub struct BirthPending { pub request: BirthRequest, pub sim: Box<Simulation> }

pub fn rotate(yaw: Yaw, [x, y, z]: [i64; 3]) -> Result<[i64; 3]> {
    Ok(match yaw {
        Yaw::Zero => [x, y, z], Yaw::Quarter => [z, y, x.checked_neg().ok_or("birth offset overflow")?],
        Yaw::Half => [x.checked_neg().ok_or("birth offset overflow")?, y, z.checked_neg().ok_or("birth offset overflow")?],
        Yaw::ThreeQuarter => [z.checked_neg().ok_or("birth offset overflow")?, y, x],
    })
}

pub fn birth_pose(input: &BirthGeometry) -> Result<(Pose, [i64; 3])> {
    let offset = rotate(input.parent_pose.yaw, input.offset_from_parent)?;
    let mut pose = Pose { at: input.parent_pose.at, yaw: input.parent_pose.yaw.compose(input.yaw_from_parent) };
    for k in 0..3 { pose.at[k] = pose.at[k].checked_add(offset[k]).ok_or("birth pose overflow")?; }
    let offset = rotate(pose.yaw, input.eye_from_child)?;
    let mut eye = pose.at;
    for k in 0..3 { eye[k] = eye[k].checked_add(offset[k]).ok_or("birth eye overflow")?; }
    Ok((pose, eye))
}

impl Simulation {
    fn birth_request(&self, mark: &BirthMark) -> BirthRequest {
        let parent = self.state.population.get(mark.parent).expect("accepted biological parent exists");
        let child = self.state.population.get(mark.child).expect("accepted biological child exists");
        BirthRequest { action: mark.action.clone(), process: mark.process.clone(), parent: mark.parent,
            child: mark.child, ordinal: mark.ordinal, tick: self.state.tick, revision: self.revision.clone(),
            parent_lineage: parent.lineage.clone(), child_lineage: child.lineage.clone(),
            parent_alive: mark.parent_alive, parent_revision: parent.body_revision, parent_visit: parent.arrived,
            site: parent.resident_site(), parent_patch: parent.patch, child_revision: child.body_revision,
            child_visit: child.arrived, child_journey: child.journey.clone(), terrain_revision: self.terrain_revision() }
    }

    pub(crate) fn place_births(&mut self, marks: &[BirthMark]) {
        if !self.spatial_resolutions.birth_enabled { return; }
        for mark in marks {
            let request = self.birth_request(mark);
            let available = request.site.is_some() && request.child_journey.is_none()
                && self.genesis.rules.birth_profiles.values().any(|p| p.process == request.process);
            if available && self.spatial_resolutions.birth_pending.is_none()
                && !self.spatial_resolutions.births.contains_key(&mark.child)
                && let Some(answer) = self.spatial_resolutions.birth_fallback.as_ref()
                    .and_then(|fallback| fallback.answer(&request, &self.spatial_resolutions)) {
                self.spatial_resolutions.auto_births.push(answer.clone());
                self.spatial_resolutions.births.insert(mark.child, answer);
            }
            if let Some(input) = self.spatial_resolutions.births.get(&mark.child).cloned() {
                let profile = input.geometry.as_ref().ok().map(|g| g.profile.clone());
                let patch = if input.request != request { Err("birth answer names another accepted native source".into()) }
                    else { input.geometry.as_ref().map_err(|why| why.clone())
                        .and_then(|g| super::birth_geometry::check(self, &request, g)) };
                if let Ok(patch) = &patch {
                    if self.foraging_enabled() {
                        let (pose, _) = birth_pose(input.geometry.as_ref().expect("accepted birth geometry"))
                            .expect("accepted birth pose");
                        self.keep_spatial_pose(mark.child, request.site.expect("accepted birth site"), pose);
                    }
                    self.state.population.lift(mark.child).expect("accepted child identity exists").patch = Some(*patch);
                    if let Some(targets) = &mut self.targets { targets.touch(&self.state.population, [mark.child]); }
                    if let Some(filed) = &mut self.filed {
                        filed.touch(&self.state.population, [mark.child], self.state.tick, &self.genesis.rules);
                    }
                }
                self.spatial_resolutions.born.insert(mark.child);
                self.spatial_resolutions.birth_reports.push(BirthReport { request, profile, patch });
                self.spatial_resolutions.order.push(Publication::Birth(mark.child));
            } else if !self.genesis.rules.birth_profiles.values().any(|p| p.process == request.process)
                || request.site.is_none() || request.child_journey.is_some()
            {
                self.spatial_resolutions.birth_reports.push(BirthReport { request, profile: None,
                    patch: Err("birth has no resident parent pose/profile; physical binding is unavailable".into()) });
                self.spatial_resolutions.order.push(Publication::Birth(mark.child));
            } else if self.spatial_resolutions.birth_pending.is_none() {
                let mut sim = self.clone();
                sim.clear_owned_capture();
                sim.journal = None;
                sim.geometry_resolutions = Default::default();
                sim.spatial_resolutions.pending = None;
                sim.spatial_resolutions.birth_pending = None;
                self.spatial_resolutions.birth_pending = Some(BirthPending { request, sim: Box::new(sim) });
            }
        }
    }

    pub fn spatial_births(&self) -> &[BirthReport] { &self.spatial_resolutions.birth_reports }
}
