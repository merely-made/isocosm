// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn preview(mode: &ModeState, state: &State, actions: &ActionState, sim: &Simulation,
    steps: &[native::Resolution], arrivals: &[native::ArrivalResolution], births: &[native::BirthResolution])
    -> Result<(ModeState, State, ActionState), String> {
    let mut mode = mode.clone(); let mut state = state.clone(); let mut actions = actions.clone();
    arrival::publish(&mut mode, &mut state, arrivals, sim.spatial_arrivals())?;
    publication::ordered(&mut mode, &mut state, &mut actions, sim, steps, births,
        sim.spatial_births(), sim.spatial_order())?;
    Ok((mode, state, actions))
}

/// Exactly one native tick. Native preparation snapshots are borrowed views,
/// not a second game runtime; only its Ready candidate can be published.
pub fn advance(session: &mut Session, mode: &mut ModeState, actions: &mut ActionState) -> Result<String, String> {
    super::super::life::validate_native(session, &mode.life)?;
    let mut state = mode.clock.take().ok_or("Eponym clock has no authored provider")?;
    let result = advance_inner(session, mode, actions, &mut state);
    mode.clock = Some(state); mode.version = mode.required_version();
    result
}

fn advance_inner(session: &mut Session, mode: &mut ModeState, actions: &mut ActionState, state: &mut State)
    -> Result<String, String> {
    state.validate(mode)?; state.validate_native(session)?;
    let (before_entries, _) = publication::refresh_inner(session, mode, state, actions)?;
    let arrivals = arrival::inputs(session, mode, state)?;
    let mut terrain: Vec<native::TerrainResolution> = Vec::new();
    let mut steps: Vec<native::Resolution> = Vec::new();
    let mut births: Vec<native::BirthResolution> = Vec::new();
    let limit = session.sim.genesis().rules.limits.events_per_advance;
    loop {
        match session.prepare_birth_spatial(1, &terrain, &steps, &arrivals, &births)? {
            native::Preparation::Terrain(pending) => {
                finite(&terrain, &steps, &arrivals, limit)?;
                if terrain.iter().any(|r| r.request.action == pending.request.action) {
                    return Err("Eponym terrain geometry made no finite progress".into());
                }
                let (mode, state, _) = preview(mode, state, actions, &pending.sim, &steps, &arrivals, &births)?;
                let geometry = geometry::terrain(&mode, &state, &pending);
                terrain.push(native::TerrainResolution { request: pending.request, geometry });
            },
            native::Preparation::Step(pending) => {
                finite(&terrain, &steps, &arrivals, limit)?;
                if steps.iter().any(|r| r.request.action == pending.request.action) {
                    return Err("Eponym step geometry made no finite progress".into());
                }
                let (mode, state, _) = preview(mode, state, actions, &pending.sim, &steps, &arrivals, &births)?;
                let geometry = geometry::step(&mode, &state, &pending);
                steps.push(native::Resolution { request: pending.request, geometry });
            },
            native::Preparation::Birth(pending) => {
                if births.len() as u128 >= u128::from(session.sim.genesis().rules.limits.entities)
                    || births.iter().any(|r| r.request.child == pending.request.child) {
                    return Err("Eponym birth geometry made no finite progress".into());
                }
                let (mode, state, actions) = preview(mode, state, actions, &pending.sim, &steps, &arrivals, &births)?;
                let geometry = birth::resolve(&mode, &state, &actions, &pending);
                births.push(native::BirthResolution { request: pending.request, geometry });
            },
            native::Preparation::Ready(prepared) => {
                let accepted = prepared.steps.clone(); let reports = prepared.arrivals.clone();
                let born = prepared.births.clone(); let order = prepared.order.clone();
                let out = prepared.publish(session)?;
                let entry = session.entries.last().ok_or("native spatial entry is absent")?.clone();
                let mut changes = arrival::publish(mode, state, &arrivals, &reports)?;
                changes.extend(publication::ordered(mode, state, actions, &session.sim, &steps, &births, &born, &order)?);
                let ordered: BTreeSet<_> = order.iter().filter_map(|p|
                    if let native::Publication::Step(key) = p { Some(key.clone()) } else { None }).collect();
                if ordered != accepted { return Err("native accepted steps differ from publication order".into()); }
                let (refresh_entries, refreshed) = publication::refresh_inner(session, mode, state, actions)?;
                changes.extend(refreshed);
                state.records.push(Record { entry: entry.id, outcome: entry.outcome, before_entries, terrain, steps,
                    arrivals, births, arrival_reports: reports, birth_reports: born, order, changes, refresh_entries });
                state.validate(mode)?; state.validate_native(session)?;
                return Ok(format!("native spatial advanced to {}: {:?}", out.tick, out.result));
            },
        }
    }
}

fn finite(terrain: &[native::TerrainResolution], steps: &[native::Resolution], arrivals: &[native::ArrivalResolution],
    limit: usize) -> Result<(), String> {
    let count = terrain.len().checked_add(steps.len()).and_then(|n| n.checked_add(arrivals.len()))
        .ok_or("Eponym geometry input count overflows")?;
    if count >= limit { return Err("Eponym geometry exceeds the authored native input budget".into()); }
    Ok(())
}
