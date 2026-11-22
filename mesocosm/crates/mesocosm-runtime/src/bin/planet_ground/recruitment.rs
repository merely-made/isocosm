// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Result;
use cosmograph::{
    Execution, Session,
    ecology::Rates,
    rules::{Climate, Dormancy, Rung, year_ticks},
    schema::Relation,
};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, io::Write, path::Path};

/// Latitude at a site centre, in the atlas viewer's six oriented cube faces.
fn latitude(site: u64, n: u32) -> i64 {
    let (column, row) = (site as u32 % (3 * n), site as u32 / (3 * n));
    let face = (row / n * 3 + column / n) as usize;
    let [normal, right, down] = cosmograph::map::sphere::FACES[face];
    let offset = |index| f64::from(2 * (index % n) + 1) / f64::from(n) - 1.0;
    let v = [0, 1, 2].map(|i| {
        f64::from(normal[i])
            + offset(column) * f64::from(right[i])
            + offset(row) * f64::from(down[i])
    });
    let length = v[0].hypot(v[1]).hypot(v[2]);
    ((v[1] / length).asin().to_degrees() * 100.0).round() as i64
}

/// A sibling founding with the same A1 organisms and stores. A year has 52
/// weekly rounds; propagules can wake once per quarter. Eight units is the
/// original A1 birth provision, so neither treatment buys cheaper daughters.
pub(super) fn seasonal_parent(parent: &Path, destination: &Path) -> Result<()> {
    let source = Session::load_json(&fs::read(parent)?, Execution::Grouped)?;
    let genesis = source.sim.genesis();
    let mut template = genesis
        .founding
        .clone()
        .ok_or("missing founding template")?;
    if source.sim.state().tick != 0
        || !source.entries.is_empty()
        || template.ecology_rates != Rates::A1
        || template.dormancy.is_some()
        || &template.generate()? != genesis
    {
        return Err(
            "seasonal survey requires an unmodified tick-zero generated A1 founding".into(),
        );
    }
    let week = Rung::Week.microseconds();
    let year = year_ticks(week);
    template.dormancy = Some(Dormancy {
        seed: [8, 8],
        season_ticks: Some(year / 4),
        ..Dormancy::flat()
    });
    let mut g = template.generate()?;
    g.world.parent_world = Some(source.save().genesis_digest);
    g.rules.tick_microseconds = Some(week);
    g.rules.epoch_ticks = year;
    let Some(cosmograph::map::Layout::Grid(grid)) = &template.map else {
        return Err("a spherical atlas is required".into());
    };
    if grid.shape != "shape:sphere" {
        return Err("a spherical atlas is required".into());
    }
    // Weekly time steps resolve the annual temperature wave and seed clocks.
    // Suppress weather, drought and shading costs in this thermal comparison.
    g.rules.climate = Some(Climate {
        weather: 0,
        drought: [0, 1],
        dim: [0, 1],
        strain: [1, 60],
        ..Climate::default()
    });
    let key = cosmograph::light::LATITUDE;
    g.rules.conditions.insert(key.into());
    for (&site, patch) in &mut g.sites {
        patch
            .conditions
            .insert(key.into(), latitude(site, grid.width / 3));
    }
    let s = Session::new(g, Execution::Grouped)?;
    let bytes = serde_json::to_vec(&s.save())?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?
        .write_all(&bytes)?;
    Ok(())
}

/// Read kept parentage and biological state without changing focus or pinning anybody.
pub(super) fn family(session: &Session, ancestor: u64) -> Result<Value> {
    let state = session.sim.state();
    if state.population.get(ancestor).is_none() {
        return Err("ancestor missing".into());
    }
    let mut todo = vec![(ancestor, None::<u64>, 0)];
    let mut seen = BTreeSet::new();
    let mut members = vec![];
    while let Some((id, parent, generation)) = todo.pop() {
        if !seen.insert(id) {
            continue;
        }
        let entity = state.population.get(id).ok_or("family member missing")?;
        members.push(json!({"id":id, "parent":parent, "generation":generation,
            "alive":entity.alive,"dormant":session.sim.genesis().rules.dormant(entity),
            "site":entity.resident_site(),"journey":entity.journey}));
        // Native ties are already ordered by subject, kind and object.
        let start = Relation::new(id, "sim:child", 0);
        for tie in state
            .relations
            .range(start..)
            .take_while(|tie| tie.subject == id && tie.kind == "sim:child")
        {
            todo.push((tie.object, Some(id), generation + 1));
        }
    }
    Ok(json!({"tick":state.tick,"ancestor":ancestor,"members":members}))
}
