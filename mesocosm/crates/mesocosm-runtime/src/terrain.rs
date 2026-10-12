// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Game geometry answers a native declaration, never a player-driving edit.

use isocosm::{Session, operations::{Change, Geometry, Pending, Preparation, Resolution, Window},
    flows::FlowResult, simulation::Work};
use isocosm_overlay::{OrdinaryAccess, Seat, mesocosm::{TerrainGeometry, TerrainShape}};
use isometer_space::{Edit, Op, Shape};
use crate::Refusal;

pub type Resolver = dyn FnMut(&Pending) -> Result<TerrainGeometry, String>;

pub(crate) fn resolved(pending: &Pending, actual: TerrainGeometry) -> Result<Resolution, String> {
    let request = &pending.request;
    if actual.action != request.action || actual.process.0 != request.process
        || actual.tick.0 != request.tick
        || actual.actor.0 != request.actor || actual.site.0 != request.site
        || actual.body_revision != request.body_revision
        || actual.terrain_revision != request.terrain_revision
    { return Err("geometry belongs to another actor, site or revision".into()); }
    let op = match &request.change {
        Change::Carve => Op::Carve,
        Change::Deposit { material } => {
            let id = pending.sim.genesis().world.materials.iter().position(|m| &m.key == material)
                .ok_or("declared deposit material is absent")?;
            Op::Fill(u8::try_from(id).map_err(|_| "deposit material id overflow")?)
        },
    };
    let shape = match actual.shape {
        TerrainShape::Sphere { centre, radius } => Shape::Sphere { centre, radius },
        TerrainShape::Box { min, max } => Shape::Box { min, max },
        TerrainShape::Route { points, radius } => Shape::Route { points, radius },
    };
    let geometry = Geometry {
        window: Window { min: actual.min, max: actual.max, depth: actual.depth },
        eye: actual.eye, focus: actual.focus, edit: Edit { op, shape },
    };
    isocosm::operations::check_geometry(&pending.sim, request, &geometry)?;
    Ok(Resolution { request: request.clone(), geometry })
}

/// One atomic native tick. The active seat cannot turn observation into edits.
/// A native NPC's operation still comes from its own chosen process.
pub fn advance<R>(session: &mut Session, access: OrdinaryAccess, resolver: &mut R)
    -> Result<FlowResult<Work>, Refusal>
where R: FnMut(&Pending) -> Result<TerrainGeometry, String> + ?Sized
{
    if access.seat != Seat::Player || access.subject.is_none()
        || session.sim.plays(access.participant.0) != access.subject.map(|id| id.0)
    { return Err(Refusal::NotPlayed); }
    let mut resolutions: Vec<Resolution> = Vec::new();
    loop {
        match session.prepare_geometry(1, &resolutions).map_err(Refusal::Sim)? {
            Preparation::Ready(prepared) => return prepared.publish(session).map_err(Refusal::Sim),
            Preparation::Pending(pending) => {
                if resolutions.len() >= session.sim.genesis().rules.limits.events_per_advance
                    || resolutions.iter().any(|r| r.request.action == pending.request.action)
                { return Err(Refusal::Sim("native geometry request did not make finite progress".into())); }
                let actual = resolver(&pending).map_err(Refusal::Sim)?;
                resolutions.push(resolved(&pending, actual).map_err(Refusal::Sim)?);
            },
        }
    }
}
