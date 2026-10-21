// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual production clocks resolve chosen steps; input never directly moves a body.

use super::*;
use isocosm::spatial::{self as native, SpatialAdvance};
use isometer_space::places::Places;
use std::collections::VecDeque;

impl Game {
    fn resolve_step(&self, pending: &native::Pending, cache: &mut GeometryCache) -> Result<native::Geometry, String> {
        if self.generated_driver() { return self.generated_step(pending, cache); }
        let r = &pending.request;
        let profile = pending.sim.genesis().rules.step_profiles.get(&r.profile).ok_or("native step profile absent")?;
        let input = self.driver.as_ref().ok_or("chosen step has no authored driver")?
            .step_input(r.actor, &r.profile)?;
        if input.target.map(|id| id.0) != r.target { return Err("step goal names another chosen target".into()); }
        let at = self.members.get(&r.actor).ok_or("step actor has no actual saved pose")?.rebound(&pending.sim)?;
        at.check(&pending.sim)?;
        if at.patch != r.patch || at.body_revision != r.body_revision || at.terrain_revision != r.terrain_revision
            || at.places.unwrap_or_default() != profile.places
        { return Err("step profile or actual placement is stale".into()); }
        let volume = pending.sim.volume(r.site, at.window.min, at.window.max, at.window.depth)?;
        let places = Places::derive(&volume, profile.places)?;
        let mut parents = BTreeMap::from([(at.pose.at, None)]);
        let mut queue = VecDeque::from([at.pose.at]);
        while let Some(cell) = queue.pop_front() {
            if cell == input.focus { break; }
            for next in native::neighbours(&volume, &places, profile, cell) {
                if let std::collections::btree_map::Entry::Vacant(slot) = parents.entry(next) {
                    slot.insert(Some(cell)); queue.push_back(next);
                }
            }
        }
        if !parents.contains_key(&input.focus) { return Err("authored focus has no realized walking route".into()); }
        let mut cells = vec![input.focus];
        while let Some(Some(before)) = parents.get(cells.last().ok_or("step route absent")?) { cells.push(*before); }
        cells.reverse();
        let count = usize::try_from(r.cells).unwrap_or(usize::MAX).saturating_add(1);
        cells.truncate(count);
        let mut path: Vec<_> = cells.into_iter().map(|cell| native::Pose { at: cell, yaw: input.yaw }).collect();
        // The first root is the saved source pose, including its exact yaw.
        path.first_mut().ok_or("step route empty")?.yaw = at.pose.yaw;
        let last = path.last().ok_or("step route empty")?.at;
        let mut eye = last;
        for k in 0..3 { eye[k] = eye[k].checked_add(input.eye_offset[k]).ok_or("step eye offset overflow")?; }
        let geometry = native::Geometry { window: at.window.clone(), path, eye };
        native::check_step(&pending.sim, r, &geometry)?;
        Ok(geometry)
    }

    pub(super) fn apply_steps(&mut self, inputs: &[native::Resolution], accepted: &BTreeSet<String>)
        -> Result<Vec<profile::Transition>, String> {
        let mut changes = Vec::new();
        for input in inputs.iter().filter(|r| accepted.contains(&r.request.action)) {
            let geometry = input.geometry.as_ref().map_err(|why| format!("accepted step was refused: {why}"))?;
            let before = self.members.get(&input.request.actor).ok_or("accepted step source pose absent")?.clone();
            let last = geometry.path.last().ok_or("accepted step path empty")?;
            if geometry.path[0].at != before.pose.at || geometry.path[0].yaw != before.pose.yaw {
                return Err("accepted step does not begin at the saved root pose".into());
            }
            if geometry.window != before.window || input.request.site != before.site.0 {
                return Err("accepted step does not use the saved actual site window".into());
            }
            let mut after = before.clone();
            after.pose = Pose { at: last.at, yaw: last.yaw };
            after.eye = geometry.eye;
            after.body_revision = input.request.body_revision;
            after.terrain_revision = input.request.terrain_revision.clone();
            // The final patch is re-derived from this same exact pose on refresh.
            changes.push(profile::Transition { actor: input.request.actor, before: Some(before), after: after.clone(),
                admission: profile::Admission::Step { action: input.request.action.clone() }, sight_range: None });
            self.members.insert(input.request.actor, after);
        }
        Ok(changes)
    }

    pub(super) fn apply_publications(&mut self, sim: &Simulation, steps: &[native::Resolution],
        births: &[native::BirthResolution], reports: &[native::BirthReport], order: &[native::Publication])
        -> Result<Vec<profile::Transition>, String> {
        let mut changes = Vec::new();
        let mut used_steps = BTreeSet::new();
        let mut used_births = BTreeSet::new();
        for publication in order {
            match publication {
                native::Publication::Step(action) => {
                    if !used_steps.insert(action.clone()) { return Err("duplicate native step publication".into()); }
                    let input = steps.iter().find(|r| &r.request.action == action).ok_or("step publication answer absent")?;
                    changes.extend(self.apply_steps(std::slice::from_ref(input), &BTreeSet::from([action.clone()]))?);
                },
                native::Publication::Birth(child) => {
                    if !used_births.insert(*child) { return Err("duplicate biological birth publication".into()); }
                    let report = reports.iter().find(|r| r.request.child == *child).ok_or("birth publication report absent")?;
                    changes.extend(self.apply_births(births, std::slice::from_ref(report), sim)?);
                },
            }
        }
        if used_births.len() != reports.len() { return Err("birth report omitted from native publication order".into()); }
        Ok(changes)
    }

    fn preview(&self, sim: &Simulation, steps: &[native::Resolution], arrivals: &[native::ArrivalResolution],
        births: &[native::BirthResolution], geometry: &mut super::refresh::GeometryCache)
        -> Result<Self, String> {
        let _span = crate::profile::Span::new("clock.preview");
        let mut game = self.clone();
        game.apply_arrivals(arrivals, sim.spatial_arrivals())?;
        game.apply_publications(sim, steps, births, sim.spatial_births(), sim.spatial_order())?;
        for held in game.members.values_mut() {
            if let Ok(next) = held.rebound_with(sim, geometry) { *held = next; }
        }
        Ok(game)
    }

    /// Trusted owner clock. Ordinary played inputs use `advance` and its native access gate.
    pub fn advance_clock(&mut self, session: &mut Session)
        -> Result<isocosm::flows::FlowResult<isocosm::simulation::Work>, String> {
        self.advance_clock_with_geometry(session, &mut super::refresh::GeometryCache::default())
    }

    pub(crate) fn advance_clock_with_geometry(&mut self, session: &mut Session,
        geometry: &mut super::refresh::GeometryCache)
        -> Result<isocosm::flows::FlowResult<isocosm::simulation::Work>, String> {
        let _span = crate::profile::Span::new("clock.total");
        self.version()?;
        let mut native_session = crate::profile::measure("clock.clone_session", || session.clone());
        let mut source = crate::profile::measure("clock.clone_game", || self.clone());
        source.refresh_with(&mut native_session, geometry)?;
        let arrivals = source.arrival_inputs(&native_session)?;
        let fallback = source.generated_driver().then(|| native::BirthFallback {
            known_parents: source.members.keys().copied().collect(),
            refusal: super::generated_body::UNPLACED_PARENT.into(),
        });
        let mut terrain: Vec<native::TerrainResolution> = Vec::new();
        let mut steps: Vec<native::Resolution> = Vec::new();
        let mut births: Vec<native::BirthResolution> = Vec::new();
        let limit = native_session.sim.genesis().rules.limits.events_per_advance;
        loop {
            match crate::profile::measure("clock.prepare", ||
                match &fallback {
                    Some(fallback) => {
                        let (attempt, automatic) = native_session.try_advance_birth_spatial_with_fallback(
                            1, &terrain, &steps, &arrivals, &births, fallback)?;
                        births.extend(automatic);
                        Ok(attempt)
                    },
                    None => native_session.try_advance_birth_spatial_with_flows(1, &terrain, &steps, &arrivals, &births),
                })? {
                SpatialAdvance::Terrain(pending) => {
                    if terrain.len() + steps.len() + arrivals.len() >= limit
                        || terrain.iter().any(|r| r.request.action == pending.request.action)
                    { return Err("spatial terrain request made no finite progress".into()); }
                    let game = source.preview(&pending.sim, &steps, &arrivals, &births, geometry)?;
                    let answer = crate::profile::measure("clock.answer.terrain", || game.resolve(&pending)
                        .and_then(|actual| crate::terrain::resolved(&pending, actual)).map(|r| r.geometry));
                    terrain.push(native::TerrainResolution { request: pending.request, geometry: answer });
                },
                SpatialAdvance::Step(pending) => {
                    if terrain.len() + steps.len() + arrivals.len() >= limit
                        || steps.iter().any(|r| r.request.action == pending.request.action)
                    { return Err("spatial step request made no finite progress".into()); }
                    let game = source.preview(&pending.sim, &steps, &arrivals, &births, geometry)?;
                    let answer = crate::profile::measure("clock.answer.step", || game.resolve_step(&pending, geometry));
                    steps.push(native::Resolution { request: pending.request, geometry: answer });
                },
                SpatialAdvance::Birth(pending) => {
                    if births.len() as u128 >= u128::from(native_session.sim.genesis().rules.limits.entities)
                        || births.iter().any(|r| r.request.child == pending.request.child)
                    { return Err("biological placement request made no finite progress".into()); }
                    let game = source.preview(&pending.sim, &steps, &arrivals, &births, geometry)?;
                    let answer = crate::profile::measure("clock.answer.birth", || game.resolve_birth_with_geometry(&pending, geometry));
                    births.push(native::BirthResolution { request: pending.request, geometry: answer });
                },
                SpatialAdvance::Advanced(out) => {
                    // Fallback refusals are ordinary explicit native receipts.
                    if let Some(isocosm::history::Entry { command: isocosm::Command::AdvanceBirthSpatial { births: recorded, .. }, .. })
                        = native_session.entries.last() { births = recorded.clone(); }
                    let accepted = native_session.sim.spatial_steps().clone();
                    let reports = native_session.sim.spatial_arrivals().to_vec();
                    let born = native_session.sim.spatial_births().to_vec();
                    let order = native_session.sim.spatial_order().to_vec();
                    let mut game = source;
                    let mut changes = game.apply_arrivals(&arrivals, &reports)?;
                    changes.extend(game.apply_publications(&native_session.sim, &steps, &births, &born, &order)?);
                    let ordered: BTreeSet<_> = order.iter().filter_map(|p|
                        if let native::Publication::Step(action) = p { Some(action.clone()) } else { None }).collect();
                    if ordered != accepted { return Err("native accepted steps differ from publication order".into()); }
                    if let Some(driver) = &mut game.driver {
                        let entry = native_session.entries.last().ok_or("spatial native entry absent")?.id.clone();
                        driver.records.push(profile::Record { entry, changes, steps: accepted, arrivals: reports,
                            births: born, order, command_outcome: None });
                    }
                    game.version = game.current_version();
                    game.refresh_with(&mut native_session, geometry)?;
                    crate::profile::measure("clock.assign", || { *session = native_session; *self = game; });
                    return Ok(out);
                },
            }
        }
    }
}
