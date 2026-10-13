// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ordinary explicit start/view choices over existing native founding law.

use super::{HostConfig, Runtime};

pub(super) fn generated(config: &HostConfig, affinity: Option<isocosm::rules::Affinity>) -> Result<Runtime, String> {
    if let Some(path) = &config.ecology_candidate {
        let setup = read_setup(path)?;
        return setup.offer(config.ticks_per_second);
    }
    let seed = if config.unchosen_seed {
        // Same clock-sourced seed as native isocosm-bench. This is not OS randomness.
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|why| why.to_string())?.as_nanos() as u64
    } else { config.seed };
    let mut founded = mesocosm_runtime::Founded::generated_with_affinity(seed, config.population, affinity)?;
    if let Some(start) = config.start { founded.start = start; }
    if let Some(mode) = config.mode { founded.mode = mode; }
    let path = config.ecology_witness.as_ref()
        .ok_or("new generated world requires --ecology-witness with complete authored witness inputs")?;
    let setup = read_setup(path)?;
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
