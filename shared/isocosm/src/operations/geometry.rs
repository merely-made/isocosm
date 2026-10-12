// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual cells, native placement and the ruled ray bound a returned edit.

use serde::{Deserialize, Serialize};
use isometer_space::{Atlas, Edit, Op, Shape, places::{Places, Rules, ray}};
use crate::{Result, schema::Key, simulation::Simulation};
use super::{Change, Request};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window { pub min: [i64; 2], pub max: [i64; 2], pub depth: i64 }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geometry {
    pub window: Window,
    pub eye: [i64; 3],
    pub focus: [i64; 3],
    pub edit: Edit,
}

fn extent(shape: &Shape) -> Result<[[i64; 3]; 2]> {
    let around = |point: &[i64; 3], radius: u64| -> Result<()> {
        let radius = i64::try_from(radius).map_err(|_| "shape radius overflow")?;
        for c in point {
            c.checked_sub(radius).and_then(|_| c.checked_add(radius)).ok_or("shape extent overflow")?;
        }
        Ok(())
    };
    match shape {
        Shape::Sphere { centre, radius } => around(centre, *radius)?,
        Shape::Route { points, radius } => {
            if points.is_empty() || points.len() > isometer_space::edit::ROUTE_POINTS {
                return Err("invalid geometry route".into());
            }
            for p in points { around(p, *radius)?; }
        },
        Shape::Box { min, max } => {
            for k in 0..3 { max[k].checked_sub(min[k]).ok_or("shape extent overflow")?; }
        },
    }
    let bounds = shape.extent();
    if (0..3).any(|k| bounds[1][k] <= bounds[0][k]) {
        return Err("empty or inverted operation geometry".into());
    }
    Ok(bounds)
}

pub(crate) fn price(sim: &Simulation, request: &Request, geometry: &Geometry)
    -> Result<(Vec<(Key, u64)>, Vec<(Key, u64)>)>
{
    let actor = sim.state().population.get(request.actor).ok_or("operation actor absent")?;
    if !actor.alive || actor.resident_site() != Some(request.site)
        || actor.patch != Some(request.patch) || actor.body_revision != request.body_revision
        || sim.terrain_revision() != request.terrain_revision
        || sim.state().tick != request.tick || sim.genesis().rules.revision() != request.revision
    { return Err("operation placement or revision changed".into()); }
    let atlas = sim.atlas()?;
    let bounds = extent(&geometry.edit.shape)?;
    geometry.edit.check(atlas.footprint())?;
    let expected = match &request.change {
        Change::Carve => Op::Carve,
        Change::Deposit { material } => {
            let id = sim.genesis().world.materials.iter().position(|m| &m.key == material)
                .ok_or("operation material absent")?;
            Op::Fill(u8::try_from(id).map_err(|_| "operation material overflow")?)
        },
    };
    if geometry.edit.op != expected { return Err("edit changes another declared material".into()); }
    if request.reach > i64::MAX as u64 || geometry.window.depth < 0 {
        return Err("operation geometry range overflow".into());
    }
    let v = sim.volume(request.site, geometry.window.min, geometry.window.max, geometry.window.depth)?;
    let inside = |at: [i64; 3]| v.holds(at[0], at[2]) && at[1] >= v.datum && at[1] < v.ceiling();
    if !inside(geometry.eye) || !inside(geometry.focus)
        || !inside(bounds[0]) || !inside(bounds[1].map(|n| n - 1))
    { return Err("operation cells are outside the actual window".into()); }
    let places = Places::derive(&v, Rules::default())?;
    if places.place_at(geometry.eye) != Some(request.patch) {
        return Err("operation eye is outside its admitted native place".into());
    }
    let clear = v.material(geometry.eye).is_some_and(|m| m == v.materials.air || m == v.materials.water);
    if !clear { return Err("operation eye is not in clear volume".into()); }
    for k in 0..3 {
        if geometry.eye[k].abs_diff(geometry.focus[k]) > request.reach
            || geometry.eye[k].abs_diff(bounds[0][k]) > request.reach
            || geometry.eye[k].abs_diff(bounds[1][k] - 1) > request.reach
            || geometry.focus[k] < bounds[0][k] || geometry.focus[k] >= bounds[1][k]
        { return Err("operation shape is outside declared reach or focus".into()); }
    }
    let span = (0..3).map(|k| geometry.eye[k].abs_diff(geometry.focus[k])).max().unwrap_or(0);
    if span.checked_mul(span).and_then(|n| n.checked_mul(2)).is_none_or(|n| n > i64::MAX as u64)
        || !ray(&v, geometry.eye, geometry.focus)
    { return Err("operation focus is outside the ruled sight ray".into()); }
    let mut gained = Vec::new();
    let mut paid = Vec::new();
    let (mut removed, mut placed) = (0u128, 0u128);
    for (id, cells) in sim.moved(request.site, &geometry.edit)? {
        let material = &sim.genesis().world.materials[usize::from(id)];
        let (Some(density), Some(account)) = (material.density, &material.account) else {
            return Err("operation material has no native density/account".into());
        };
        let n = cells.unsigned_abs();
        let amount = n.checked_mul(density).ok_or("operation material mass overflow")?;
        if cells > 0 { removed += u128::from(n); gained.push((account.clone(), amount)); }
        else { placed += u128::from(n); paid.push((account.clone(), amount)); }
    }
    if removed.max(placed) > u128::from(request.cells) {
        return Err("edit exceeds the native operation cell budget".into());
    }
    Ok((gained, paid))
}
