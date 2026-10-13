// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ordinary explicit start/view choices over existing native founding law.

use super::{HostConfig, Runtime};

pub(super) fn generated(config: &HostConfig, affinity: Option<isocosm::rules::Affinity>) -> Result<Runtime, String> {
    let seed = if config.unchosen_seed {
        // Same clock-sourced seed as native isocosm-bench. This is not OS randomness.
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|why| why.to_string())?.as_nanos() as u64
    } else { config.seed };
    let mut founded = mesocosm_runtime::Founded::generated_with_affinity(seed, config.population, affinity)?;
    if let Some(start) = config.start { founded.start = start; }
    if let Some(mode) = config.mode { founded.mode = mode; }
    Runtime::new(founded, config.ticks_per_second)
}
