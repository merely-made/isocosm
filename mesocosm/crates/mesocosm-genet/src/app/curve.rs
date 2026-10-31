// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Headless ordinary runtime steps, with resumable native growth observations.
use super::*;
use cosmograph::{anatomy, spatial::embodiment::Balance};
use serde_json::{Value, json};

fn reading(runtime: &Runtime) -> Value {
    let sim = runtime.sim();
    let rules = &sim.genesis().rules;
    let actors:Vec<_>=sim.state().population.groups.iter().filter_map(|(&id,group)| {
        let e=&group.entity;
        if !e.alive {return None;}
        let doc=e.body.as_ref()?;
        let own=anatomy::own(rules,&e.lineage);
        let tissue:u128=e.living().map(|(_,p)| own.tissue.as_ref()
            .and_then(|key|p.matter.get(key)).copied().unwrap_or(0) as u128).sum();
        let capacity:u128=e.living().map(|(id,_)|u128::from(anatomy::ceiling(e.extent(id),rules.body()))).sum();
        let legs:Vec<_>=isometer::core::legs::chains(doc).iter().map(|leg| json!({
            "foot":leg.foot,"upper":e.extent(leg.upper),"lower":e.extent(leg.lower),"pad":e.extent(leg.foot),
            "upper_section":anatomy::measure(e.extent(leg.upper),cosmograph::rules::Measure::CrossSection),
            "lower_section":anatomy::measure(e.extent(leg.lower),cosmograph::rules::Measure::CrossSection)})).collect();
        Some(json!({"actor":id,"count":group.count,"lineage":e.lineage,"born":e.born,
            "age":sim.state().tick.saturating_sub(e.born),"site":e.resident_site(),
            "mass_mg":Balance::of(e,rules).map(|b|b.mass_mg()),"tissue_mg":tissue,
            "current_parts_capacity_mg":capacity,"parts":e.living().count(),"legs":legs}))
    }).collect();
    json!({"tick":sim.state().tick,"hash":format!("{:016x}",sim.state_hash()),"actors":actors})
}

fn persist(config: &HostConfig, runtime: &Runtime, observations: &[Value]) -> Result<(), String> {
    if let Some(path) = &config.receipt {
        crate::played::write_json(
            path,
            &json!({"method":"native runtime and existing keyed demo answers; no stock edits",
            "run":runtime.receipt(),"observations":observations}),
        )?;
    }
    if let Some(path) = &config.trace {
        crate::played::write_world_trace(path, runtime)?;
    }
    Ok(())
}

pub fn run(config: &HostConfig, ticks: u64, every: u64) -> Result<(), String> {
    let authored =
        mesocosm_runtime::Authored::load(&pack_root()).map_err(|why| format!("{why:?}"))?;
    let mut runtime = startup::configured(config, &authored)?;
    let mut observations = vec![reading(&runtime)];
    persist(config, &runtime, &observations)?;
    for step in 0..ticks {
        crate::played::demo_step(&mut runtime, step);
        if step % every.max(1) == 0 || step + 1 == ticks {
            let sample = reading(&runtime);
            let living: u64 = runtime
                .sim()
                .state()
                .population
                .groups
                .values()
                .filter(|g| g.entity.alive)
                .map(|g| g.count)
                .sum();
            println!(
                "curve: soil {}% tick {} living {}",
                config.play_soil,
                runtime.sim().state().tick,
                living
            );
            observations.push(sample);
            persist(config, &runtime, &observations)?;
        }
    }
    Ok(())
}
