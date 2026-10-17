// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ordinary explicit start/view choices over existing native founding law.

use super::{HostConfig, Runtime};

fn a1_setup(mut founded: mesocosm_runtime::Founded,
    affinity: Option<isocosm::rules::Affinity>)
    -> Result<mesocosm_runtime::runtime::ecology::Setup, String> {
    if affinity.is_some() {
        return Err("--ecology-rates a1 cannot be combined with the authored pack affinity; remove that affinity before selecting A1".into());
    }
    let mut founding = founded.genesis.founding.clone()
        .ok_or("--ecology-rates a1 requires an ordinary generated founding")?;
    founding.ecology_rates = isocosm::ecology::Rates::A1;
    founded.genesis = founding.generate()?;
    mesocosm_runtime::runtime::ecology::Setup::generated(
        founded, isocosm::Execution::Individuals)
}

pub(super) fn generated(config: &HostConfig, affinity: Option<isocosm::rules::Affinity>) -> Result<Runtime, String> {
    if config.ecology_rates.is_some()
        && (config.bench_world || config.ecology_candidate.is_some()
            || config.watch.is_some()
            || config.spread.is_some() || config.spatial.is_some() || config.graft.is_some()) {
        return Err("--ecology-rates requires an ordinary generated world without a bench, candidate, resume, spread, spatial or graft input".into());
    }
    if config.bench_world {
        let mut founded = mesocosm_runtime::Founded::generated_with_affinity(config.seed, config.population, affinity)?;
        founded.genesis.rules.epoch_ticks = config.bench_epoch;
        if let Some(start) = config.start { founded.start = start; }
        if let Some(mode) = config.mode { founded.mode = mode; }
        return Runtime::bench(founded, config.ticks_per_second);
    }
    if let Some(path) = &config.ecology_candidate {
        let setup = read_setup(path)?;
        return setup.offer(config.ticks_per_second);
    }
    let seed = if config.unchosen_seed {
        // Same clock-sourced seed as native isocosm-bench. This is not OS randomness.
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|why| why.to_string())?.as_nanos() as u64
    } else { config.seed };
    let mut founded = mesocosm_runtime::Founded::generated_with_affinity(seed, config.population, affinity.clone())?;
    if let Some(start) = config.start { founded.start = start; }
    if let Some(mode) = config.mode { founded.mode = mode; }
    if config.ecology_witness.is_none()
        && config.ordinary_ecology_rates() == isocosm::ecology::Rates::A1 {
        eprintln!("Preparing A1 ecology witness: seed {seed}, population {}.", config.population);
        let began = std::time::Instant::now();
        let runtime = a1_setup(founded, affinity)?.offer(config.ticks_per_second)?;
        eprintln!("A1 ecology witness accepted in {:.3}s.", began.elapsed().as_secs_f64());
        return Ok(runtime);
    }
    let path = config.ecology_witness.as_ref()
        .ok_or("new generated world requires --ecology-witness with complete authored witness inputs")?;
    let setup = read_setup(path)?;
    if let Some(rates) = config.ecology_rates {
        let mut founding = founded.genesis.founding.clone()
            .ok_or("--ecology-rates requires a generated founding")?;
        founding.ecology_rates = rates;
        founded.genesis = founding.generate()?;
    }
    // This explicit declaration is installed before the candidate's native Session exists.
    founded.genesis.rules.ecology_witness = Some(setup.specification.clone());
    if setup.founded != founded {
        return Err("ecology witness names another generated candidate, seed, or lifecycle configuration".into());
    }
    setup.offer(config.ticks_per_second)
}

fn read_setup(path: &std::path::Path) -> Result<mesocosm_runtime::runtime::ecology::Setup, String> {
    let bytes = std::fs::read(path).map_err(|why| why.to_string())?;
    serde_json::from_slice(&bytes).map_err(|why| why.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_a1_regenerates_an_unmarked_foundation_before_setup_authors_it() {
        let founded = mesocosm_runtime::Founded::generated(9, 256).unwrap();
        let setup = a1_setup(founded, None).unwrap();
        assert_eq!(setup.founded.genesis.founding.as_ref().unwrap().ecology_rates,
            isocosm::ecology::Rates::A1);
        assert_eq!(setup.specification.horizon, 60);
        assert_eq!(setup.founded.genesis.rules.ecology_witness.as_ref(), Some(&setup.specification));
        assert_eq!(setup.execution, isocosm::Execution::Individuals);
    }

    #[test]
    fn explicit_a1_reports_authored_pack_affinity_instead_of_stripping_it() {
        let founded = mesocosm_runtime::Founded::generated(9, 256).unwrap();
        let error = a1_setup(founded, Some(isocosm::rules::Affinity::default())).unwrap_err();
        assert!(error.contains("authored pack affinity"), "{error}");
    }
}
