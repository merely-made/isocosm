// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Observations for a site survey on one running world.
use super::{Branch, Result, observation};
use cosmograph::flows::{Flow, Holder, MadeBy};
use serde_json::{Value, json};
use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn frame(branch: &Branch, flows: &[Flow], site: u64) -> Result<Value> {
    let sim = &branch.session.sim;
    let state = sim.state();
    let mut sites: Vec<_> = state
        .sites
        .values()
        .map(|s| {
            json!({
                "residents":[], "flows":[], "accounts":s.accounts
            })
        })
        .collect();
    let mut bodies = Vec::new();
    let mut indices = BTreeMap::new();
    for (&first, group) in &state.population.groups {
        let e = &group.entity;
        let Some(at) = e.resident_site() else {
            continue;
        };
        if !e.alive
            || !matches!(
                e.kingdom.as_str(),
                "kingdom:flora" | "kingdom:fauna" | "kingdom:myco" | "kingdom:micro"
            )
            || cosmograph::directing::is_participant(e)
        {
            continue;
        }
        let key = format!(
            "matter:{}-0",
            e.lineage.strip_prefix("lineage:").unwrap_or(&e.lineage)
        );
        let shape = observation::anatomy(e, &key)?;
        let index = *indices.entry(shape.to_string()).or_insert_with(|| {
            bodies.push(shape);
            bodies.len() - 1
        });
        let tissue = cosmograph::anatomy::held(e, &sim.genesis().rules, &key);
        sites[at as usize]["residents"].as_array_mut().unwrap().push(json!({
            "first":first,"count":group.count,"kingdom":e.kingdom,"lineage":e.lineage,"body":index,"tissue":tissue
        }));
    }
    let mut totals = BTreeMap::new();
    for flow in flows {
        let MadeBy::Process(process) = &flow.made_by else {
            continue;
        };
        if !process.starts_with("ecology:") || flow.internal() || flow.amount == 0 {
            continue;
        }
        let site_of = |holder| match holder {
            Holder::Site(id) => Some(id),
            _ => None,
        };
        let site_endpoint = site_of(flow.from.0).or_else(|| site_of(flow.to.0));
        for n in 0..flow.count {
            let from = flow.from.0.body().map(|id| id + n);
            let to = flow.to.0.body().map(|id| id + n);
            let place = |id| state.population.get(id).and_then(|e| e.resident_site());
            let at = site_endpoint.or_else(|| {
                let a = place(from?)?;
                (place(to?) == Some(a)).then_some(a)
            });
            if let Some(at) = at {
                *totals
                    .entry((at, process, from, to, &flow.from.1, &flow.to.1))
                    .or_insert(0u64) += flow.amount;
            }
        }
    }
    for ((site, process, from, to, debit, credit), amount) in totals {
        sites[site as usize]["flows"]
            .as_array_mut()
            .unwrap()
            .push(json!({
                "process":process,"from":from,"to":to,"debit":debit,"credit":credit,"amount":amount
            }));
    }
    Ok(
        json!({"fieldSite":site,"tick":state.tick,"hash":format!("{:016x}",sim.state_hash()),
        "branch":branch.session.branch,"sites":sites,"bodies":bodies}),
    )
}

pub(super) fn found(parent: &Path, site: u64, dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)?;
    if fs::read_dir(dir)?.next().is_some() {
        return Err("the field output directory must be empty".into());
    }
    let branch = Branch::found(parent, None, Some(site))?;
    let mut terrain = branch.terrain()?;
    terrain["unitMicrometres"] = json!(branch.session.sim.genesis().world.base_unit_micrometres);
    for (name, value) in [
        ("terrain.json", terrain),
        ("founding.json", branch.save()),
        ("field.json", branch.frame(&[])?),
    ] {
        fs::write(dir.join(name), serde_json::to_vec(&value)?)?;
    }
    Ok(())
}
