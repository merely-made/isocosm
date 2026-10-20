// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Generated inputs to the existing native spatial clock. Actual parent/child
//! identities, chosen acts and publication order come from that clock.

use super::*;
use isocosm::{
    Command,
    spatial::{self as native, embodiment as body},
};
use isometer_space::places::Places;

impl Game {
    /// Current native part boxes, including growth, must clear other admitted
    /// residents. Historical poses remain saved even when this check refuses.
    pub fn placement_unoccupied(&self, sim: &Simulation, actor: Id) -> Result<(), String> {
        if !self.generated_driver() {
            return Ok(());
        }
        let at = self
            .members
            .get(&actor)
            .ok_or("physical actor has no admitted pose")?;
        let member = sim
            .state()
            .population
            .get(actor)
            .ok_or("physical actor absent")?;
        let document = member.body.as_ref().ok_or("physical body absent")?;
        let boxes = body::boxes(
            document,
            &native::Pose {
                at: at.pose.at,
                yaw: at.pose.yaw,
            },
        )?;
        if body::intersects(&boxes, &self.occupied(sim, at.site.0, Some(actor))?) {
            return Err("whole body intersects another current resident".into());
        }
        Ok(())
    }
    pub(super) fn generated_driver(&self) -> bool {
        self.driver
            .as_ref()
            .is_some_and(|driver| driver.profile.generated())
    }

    pub(super) fn found_embodied(
        &mut self,
        session: &mut Session,
        templates: &native::Templates,
    ) -> Result<bool, String> {
        let mut native = session.clone();
        let mut placements = Vec::new();
        for (&site, stands) in &templates.stands {
            if stands.is_empty() {
                continue;
            }
            let w = &templates.window;
            let volume = native.sim.volume(site, w.min, w.max, w.depth)?;
            let places = Places::derive(&volume, templates.places)?;
            for stand in stands {
                let id = stand.actor.ok_or("whole-body stand actor absent")?;
                let patch = places
                    .place_at(stand.at)
                    .ok_or("whole-body stand patch absent")?;
                native.command(Command::Patch {
                    entity: id,
                    patch: Some(patch),
                })?;
                let member = native
                    .sim
                    .state()
                    .population
                    .get(id)
                    .ok_or("whole-body stand actor absent")?;
                placements.push(Placement {
                    actor: EntityHandle(id),
                    site: PlaceHandle(site),
                    patch,
                    body_revision: member.body_revision,
                    terrain_revision: native.sim.terrain_revision(),
                    pose: Pose {
                        at: stand.at,
                        yaw: stand.yaw,
                    },
                    eye: std::array::from_fn(|k| stand.at[k] + templates.eye[k]),
                    window: w.clone(),
                    operations: BTreeMap::new(),
                    places: Some(templates.places),
                    resident_since: Some(member.arrived),
                });
            }
        }
        let mut next = self.clone();
        next.configure(
            &native,
            &Authored {
                placements,
                profile: Profile {
                    version: 5,
                    steps: BTreeMap::new(),
                    arrivals: Vec::new(),
                    births: Vec::new(),
                },
            },
        )?;
        *session = native;
        *self = next;
        Ok(true)
    }

    fn occupied(
        &self,
        sim: &Simulation,
        site: Id,
        exclude: Option<Id>,
    ) -> Result<Vec<body::Box3>, String> {
        let mut blocked = Vec::new();
        for (&id, at) in &self.members {
            if Some(id) == exclude || at.site.0 != site {
                continue;
            }
            let Some(member) = sim.state().population.get(id) else {
                continue;
            };
            if member.resident_site() != Some(site) || at.resident_since != Some(member.arrived) {
                continue;
            }
            if let Some(document) = &member.body {
                blocked.extend(body::boxes(
                    document,
                    &native::Pose {
                        at: at.pose.at,
                        yaw: at.pose.yaw,
                    },
                )?);
            }
        }
        Ok(blocked)
    }

    pub(super) fn generated_arrivals(
        &self,
        session: &Session,
    ) -> Result<Vec<native::ArrivalResolution>, String> {
        let t = session
            .sim
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("arrival templates absent")?;
        let end = session
            .sim
            .state()
            .tick
            .checked_add(1)
            .ok_or("arrival clock overflow")?;
        let mut out = Vec::new();
        let driver = self
            .driver
            .as_ref()
            .ok_or("generated arrival driver absent")?;
        if driver.initial.is_empty() {
            return Ok(out);
        }
        let mut ids: BTreeSet<_> = self
            .members
            .keys()
            .chain(driver.children.keys())
            .copied()
            .collect();
        // An unplaced founding cohort can become controlled at a death hold.
        // Its actual native journey is also a source for later physical landing.
        ids.extend(session.sim.genesis().population.groups.iter()
            .filter(|(_, group)| group.entity.body.is_some())
            .flat_map(|(&first, group)| first..first + group.count));
        let mut destinations: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
        for id in ids {
            let Some(member) = session.sim.state().population.get(id) else { continue; };
            let Some(journey) = &member.journey else { continue; };
            if member.body.is_some() && journey.arrives >= session.sim.state().tick && journey.arrives <= end {
                destinations.entry(journey.to).or_default().push(id);
            }
        }
        // Each destination keeps one terrain and surface lookup for this cut.
        // Actors retain their original id order within each destination.
        for (destination, ids) in destinations {
            let volume = session.sim.volume(destination, t.window.min, t.window.max, t.window.depth)?;
            let mut grounding = body::Grounding::new(&volume);
            let mut blocked = self.occupied(&session.sim, destination, None)?;
            let mut refused = BTreeSet::new();
            for id in ids {
                let member = session.sim.state().population.get(id).ok_or("arrival actor absent")?;
                let journey = member.journey.as_ref().ok_or("arrival journey absent")?;
                let Some(document) = &member.body else {
                    continue;
                };
                let centre = [
                    (t.window.min[0] + t.window.max[0]) / 2,
                    0,
                    (t.window.min[1] + t.window.max[1]) / 2,
                ];
                let yaw = self
                    .members
                    .get(&id)
                    .map_or(Yaw::Zero, |held| held.pose.yaw);
                let Ok(shape) = body::boxes(document, &native::Pose { at: [0; 3], yaw }) else { continue; };
                // The terrain and search stay fixed, and reservations only grow.
                // An identical shape already refused here cannot become feasible.
                if refused.contains(&shape) { continue; }
                let Ok(pose) = grounding.find(
                    document,
                    centre,
                    yaw,
                    body::SEARCH_REACH,
                    body::SEARCH_STRIDE,
                    body::SEARCH_CANDIDATES,
                    &blocked,
                ) else {
                    if refused.len() < 64 { refused.insert(shape); }
                    continue;
                };
                blocked.extend(body::boxes(document, &pose)?);
                out.push(native::ArrivalResolution {
                    request: native::ArrivalRequest {
                        actor: id,
                        from: journey.from,
                        to: journey.to,
                        departed: journey.departed,
                        arrives: journey.arrives,
                        body_revision: member.body_revision,
                        terrain_revision: session.sim.terrain_revision(),
                    },
                    geometry: native::ArrivalGeometry {
                        eye: std::array::from_fn(|k| pose.at[k] + t.eye[k]),
                        pose,
                        window: t.window.clone(),
                        places: t.places,
                    },
                });
            }
        }
        out.sort_by_key(|input| input.request.actor);
        Ok(out)
    }

    pub(super) fn generated_step(
        &self,
        pending: &native::Pending,
        cache: &mut GeometryCache,
    ) -> Result<native::Geometry, String> {
        let r = &pending.request;
        let profile = pending
            .sim
            .genesis()
            .rules
            .step_profiles
            .get(&r.profile)
            .ok_or("generated step profile absent")?;
        let at = self
            .members
            .get(&r.actor)
            .ok_or("walking actor has no admitted pose")?
            .rebound_with(&pending.sim, cache)?;
        if pending.sim.foraging_policy(&r.profile).is_some() {
            let geometry = native::foraging::geometry(&pending.sim, r, &at.window)?;
            native::check_step(&pending.sim, r, &geometry)?;
            return Ok(geometry);
        }
        let actor = pending
            .sim
            .state()
            .population
            .get(r.actor)
            .ok_or("walking actor absent")?;
        let document = actor.body.as_ref().ok_or("walking body absent")?;
        let (volume, places) = cache.get(&pending.sim, r.site, &at.window, profile.places)?;
        let before = native::Pose {
            at: at.pose.at,
            yaw: at.pose.yaw,
        };
        let blocked = self.occupied(&pending.sim, r.site, Some(r.actor))?;
        let available: Vec<_> = body::neighbours(&volume, &places, profile, document, &before)
            .into_iter()
            .filter(|pose| {
                body::boxes(document, pose).is_ok_and(|boxes| !body::intersects(&boxes, &blocked))
            })
            .collect();
        if available.is_empty() {
            return Err("no supported unoccupied whole-body walking neighbour".into());
        }
        let pick = (isocosm::draw(pending.sim.genesis().seed, "body-walk", &[r.actor, r.tick])
            % available.len() as u64) as usize;
        let after = available[pick].clone();
        let offset = pending
            .sim
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("walking templates absent")?
            .eye;
        let eye = std::array::from_fn(|k| after.at[k] + offset[k]);
        let geometry = native::Geometry {
            window: at.window.clone(),
            path: vec![before, after],
            eye,
        };
        native::check_step(&pending.sim, r, &geometry)?;
        Ok(geometry)
    }

    pub(super) fn generated_birth(
        &self,
        pending: &native::BirthPending,
        cache: &mut GeometryCache,
    ) -> Result<native::BirthGeometry, String> {
        let r = &pending.request;
        let parent = self
            .members
            .get(&r.parent)
            .ok_or("biological parent has no physical pose")?
            .rebound_with(&pending.sim, cache)?;
        if !r.parent_alive
            || r.site != Some(parent.site.0)
            || parent.resident_since != Some(r.parent_visit)
        {
            return Err("biological parent pose is unavailable at this birth".into());
        }
        let child = pending
            .sim
            .state()
            .population
            .get(r.child)
            .ok_or("actual newborn absent")?;
        let document = child.body.as_ref().ok_or("actual newborn body absent")?;
        let mut profiles = pending
            .sim
            .genesis()
            .rules
            .birth_profiles
            .iter()
            .filter(|(_, p)| p.process == r.process);
        let (key, _) = profiles
            .next()
            .ok_or("generated native birth profile absent")?;
        if profiles.next().is_some() {
            return Err("generated birth profile is ambiguous".into());
        }
        let (volume, _) = cache.get(
            &pending.sim,
            parent.site.0,
            &parent.window,
            parent.places.unwrap_or_default(),
        )?;
        let blocked = self.occupied(&pending.sim, parent.site.0, None)?;
        let pose = body::Grounding::new(&volume).find(
            document,
            parent.pose.at,
            parent.pose.yaw,
            body::SEARCH_REACH,
            body::SEARCH_STRIDE,
            body::SEARCH_CANDIDATES,
            &blocked,
        )?;
        let mut inverse = Yaw::Zero;
        while parent.pose.yaw.compose(inverse) != Yaw::Zero {
            inverse = inverse.compose(Yaw::Quarter);
        }
        let delta = std::array::from_fn(|k| pose.at[k] - parent.pose.at[k]);
        let offset = native::rotate_offset(inverse, delta)?;
        let eye = pending
            .sim
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("birth templates absent")?
            .eye;
        Ok(native::BirthGeometry {
            profile: key.clone(),
            parent_pose: native::Pose {
                at: parent.pose.at,
                yaw: parent.pose.yaw,
            },
            parent_eye: parent.eye,
            parent_window: parent.window.clone(),
            parent_places: parent.places.unwrap_or_default(),
            offset_from_parent: offset,
            yaw_from_parent: Yaw::Zero,
            eye_from_child: eye,
            window: parent.window.clone(),
        })
    }

    pub(super) fn apply_generated_birth(
        &mut self,
        resolution: &native::BirthResolution,
        report: &native::BirthReport,
        sim: &Simulation,
    ) -> Result<profile::Transition, String> {
        let r = &report.request;
        let patch = *report.patch.as_ref().map_err(|why| why.clone())?;
        let g = resolution.geometry.as_ref().map_err(|why| why.clone())?;
        let parent = self
            .members
            .get(&r.parent)
            .ok_or("accepted biological parent pose absent")?;
        let expected = sim
            .genesis()
            .rules
            .birth_profiles
            .get(&g.profile)
            .ok_or("native birth profile absent")?;
        let eye_offset = sim
            .genesis()
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("native birth templates absent")?
            .eye;
        if resolution.request != *r
            || report.profile.as_ref() != Some(&g.profile)
            || expected.process != r.process
            || g.parent_pose.at != parent.pose.at
            || g.parent_pose.yaw != parent.pose.yaw
            || g.parent_eye != parent.eye
            || g.parent_window != parent.window
            || g.window != parent.window
            || g.eye_from_child != eye_offset
            || g.parent_places != parent.places.unwrap_or_default()
            || r.site != Some(parent.site.0)
            || self.members.contains_key(&r.child)
        {
            return Err(
                "generated birth differs from its admitted parent and native answer".into(),
            );
        }
        let (pose, eye) = native::birth_pose(g)?;
        let after = Placement {
            actor: EntityHandle(r.child),
            site: parent.site,
            patch,
            body_revision: r.child_revision,
            terrain_revision: r.terrain_revision.clone(),
            pose: Pose {
                at: pose.at,
                yaw: pose.yaw,
            },
            eye,
            window: g.window.clone(),
            operations: BTreeMap::new(),
            places: Some(expected.places),
            resident_since: Some(r.child_visit),
        };
        self.members.insert(r.child, after.clone());
        Ok(profile::Transition {
            actor: r.child,
            before: None,
            after,
            admission: profile::Admission::Birth {
                action: r.action.clone(),
                parent: r.parent,
                ordinal: r.ordinal,
            },
            sight_range: None,
        })
    }
}
