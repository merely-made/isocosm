// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Export a spherical founding and stream successive observations from its save.
//! The viewer reads the same session for orbit, valley and individual views.
use cosmograph::{
    Execution, Founding, Session,
    flows::{Flow, MadeBy},
    map,
    terrain::View,
};
use isometer_space::Atlas;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, BufRead, Write},
    path::Path,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn terrain(session: &Session) -> Result<Value> {
    let g = session.sim.genesis();
    let view = View::of(g)?;
    let Some(map::Layout::Grid(grid)) = g.founding.as_ref().and_then(|f| f.map.as_ref()) else {
        return Err("a mapped founding is required".into());
    };
    if grid.shape != "shape:sphere" {
        return Err("a spherical founding is required".into());
    }
    let mut sites = vec![];
    let samples = 32;
    let side = view.footprint().side;
    for id in view.sites() {
        let l = view.lattice(id)?;
        let mut heights = vec![];
        for y in 0..=samples {
            for x in 0..=samples {
                let h = l.numerator(x * side / samples, y * side / samples) as f64
                    / l.denominator() as f64;
                heights.push((h * 16.).round() as i32);
            }
        }
        let edges: Vec<_> = (0..4)
            .map(|s| view.border(id, s).map(|(to, b)| (to, b.enters)))
            .collect();
        sites.push(json!({"elevation":view.elevation(id)?,"neighbors":edges,"heights":heights}));
    }
    Ok(
        json!({"seed":g.seed,"n":grid.width/3,"side":side,"samples":samples,"water":view.water(0)?,
              "axes":map::sphere::FACES,"sites":sites}),
    )
}

fn anatomy(entity: &cosmograph::schema::Entity, tissue_key: &str) -> Result<Value> {
    let mut blocks = vec![];
    if let Some(body) = &entity.body {
        for (id, tissue) in entity.living() {
            let p = body.part(id).ok_or("absent body part")?;
            let center = body.place(id, p.half_extent).ok_or("unplaced body part")?;
            let half = body
                .world_yaw(id)
                .ok_or("unoriented body part")?
                .rotate(p.half_extent)
                .map(i32::abs);
            let role = if tissue.port.is_some() && tissue.cells.contains_key("function:intake") {
                "mouth"
            } else if tissue.cells.contains_key("function:sense") {
                "sense"
            } else if tissue.cells.contains_key("function:move") {
                "limb"
            } else {
                "tissue"
            };
            let held = tissue.matter.get(tissue_key).copied().unwrap_or(0);
            blocks.push(json!([center, half, role, held]));
        }
    }
    Ok(json!(blocks))
}

fn frame(session: &Session, flows: &[Flow]) -> Result<Value> {
    let state = session.sim.state();
    let mut fed: BTreeMap<u64, u64> = BTreeMap::new();
    let mut sources: BTreeMap<u64, BTreeSet<(&str, u64)>> = BTreeMap::new();
    for f in flows {
        if let MadeBy::Process(key) = &f.made_by
            && key.starts_with("ecology:feed-")
            && !f.internal()
            && let Some(first) = f.to.0.body()
        {
            for id in first..first + f.count {
                *fed.entry(id).or_default() += f.amount;
                if let (Some(source), Some(kind)) = (f.from.0.body(), &f.from_kind) {
                    sources
                        .entry(id)
                        .or_default()
                        .insert((&kind.kingdom, source + (id - first)));
                }
            }
        }
    }
    let mut counts = vec![[0u64; 4]; state.sites.len()];
    let mut animals = vec![];
    let mut bodies = vec![];
    let mut body_indices = BTreeMap::new();
    for (&first, group) in &state.population.groups {
        let e = &group.entity;
        if !e.alive {
            continue;
        }
        let kind = match e.kingdom.as_str() {
            "kingdom:flora" => 0,
            "kingdom:fauna" => 1,
            "kingdom:myco" => 2,
            "kingdom:micro" => 3,
            _ => continue,
        };
        let site = e.resident_site();
        if let Some(s) = site {
            counts[s as usize][kind] += group.count;
        }
        if kind != 1 {
            continue;
        }
        let lineage = e
            .lineage
            .strip_prefix("lineage:")
            .ok_or("unknown founding lineage")?;
        let tissue_key = format!("matter:{lineage}-0");
        let shape = anatomy(e, &tissue_key)?;
        let index = *body_indices.entry(shape.to_string()).or_insert_with(|| {
            bodies.push(shape);
            bodies.len() - 1
        });
        let tissue = cosmograph::anatomy::held(e, &session.sim.genesis().rules, &tissue_key);
        for id in first..first + group.count {
            animals.push(
                json!({"id":id,"site":site,"journey":e.journey,"born":e.born,"lineage":e.lineage,
                "body":index,"tissue":tissue,"sources":sources.get(&id),"fed":fed.get(&id).copied().unwrap_or(0)}),
            );
        }
    }
    Ok(
        json!({"tick":state.tick,"hash":format!("{:016x}",session.sim.state_hash()),"counts":counts,
        "animals":animals,"bodies":bodies}),
    )
}

fn next(s: &mut Session) -> Result<Value> {
    let result = s.advance_tick_with_flows()?;
    frame(s, &result.flows)
}

fn write_json(path: &Path, v: &impl serde::Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec(v)?)?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, config, directory] if mode == "found" => {
            let d = Path::new(directory);
            fs::create_dir_all(d)?;
            let f: Founding = toml::from_str(&fs::read_to_string(config)?)?;
            let mut s = Session::new(f.generate()?, Execution::Grouped)?;
            write_json(&d.join("terrain.json"), &terrain(&s)?)?;
            write_json(&d.join("founding.json"), &s.save())?;
            let mut frames = vec![frame(&s, &[])?];
            for _ in 0..24 {
                frames.push(next(&mut s)?);
            }
            write_json(&d.join("frames.json"), &frames)?;
            write_json(&d.join("resume.json"), &s.save())
        },
        [mode, save] if mode == "watch" => {
            let mut s = Session::load_json(&fs::read(save)?, Execution::Grouped)?;
            let stdout = io::stdout();
            let mut out = io::BufWriter::new(stdout.lock());
            for line in io::stdin().lock().lines() {
                let f = match line?.as_str() {
                    "next" => next(&mut s)?,
                    "now" => frame(&s, &[])?,
                    _ => return Err("expected next or now".into()),
                };
                serde_json::to_writer(&mut out, &f)?;
                writeln!(out)?;
                out.flush()?;
            }
            Ok(())
        },
        _ => Err("usage: planet_atlas found TEMPLATE DIRECTORY | watch SAVE".into()),
    }
}
