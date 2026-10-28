// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sim owns accepted world transitions. Hosts own input, pacing and views.
//! Exact population grouping is optional; both execution modes use the same
//! admitted processes. Unsupported bulk operations execute individually.

/// Whether this build runs the society layer's operations (rulings 886 and 893).
pub const SOCIETY: bool = cfg!(feature = "society");

/// Refuses a society operation in a build without the feature; `State` is unchanged either way.
pub(crate) fn society() -> Result<()> {
    if SOCIETY { Ok(()) } else { Err("the society feature is off in this build".into()) }
}

pub mod aggregate;
pub mod anatomy;
pub mod arrival;
pub mod asserted;
pub mod attention;
pub mod band;
pub mod bench;
pub mod bodied;
pub mod datasheet;
mod dev;
pub mod development;
pub mod custody;
pub mod diffusion;
pub mod directing;
pub mod ecology;
pub mod defence;
pub mod collapse;
mod edits;
pub mod effects;
mod execute;
pub mod flows;
pub mod generate;
pub mod goals;
pub mod graft;
pub mod institutions;
mod genesis;
pub mod geometry;
pub mod growth;
pub mod harm;
pub mod history;
mod journal;
pub mod journey;
pub mod kingdom;
pub mod knowing;
pub mod lineage;
pub mod map;
pub mod material;
pub mod spatial;
pub mod mosaic;
pub mod matter;
mod meaning;
pub mod light;
pub mod needs;
pub mod operations;
pub mod population;
pub mod polities;
pub mod preset;
pub mod probe;
pub mod process;
mod profile;
pub mod projects;
pub mod party;
mod queries;
pub mod reach;
pub mod rng;
pub mod rules;
mod schedule;
pub mod schema;
pub mod spread;
pub mod simulation;
pub mod site_authoring;
pub mod social;
mod location_authoring;
mod stage;
mod strain;
pub mod systems;
mod targets;
pub mod terrain;
mod validation;
pub mod watch;

pub use generate::Founding;
pub use history::{Command, Session};
pub use simulation::{Execution, Simulation};

pub const VERSION: u32 = 1;
pub type Result<T> = std::result::Result<T, String>;

/// Ordered maps and integer-only authoritative values give a stable encoding.
pub fn digest(value: &impl serde::Serialize) -> String {
    use sha2::{Digest, Sha256};
    let bytes = serde_json::to_vec(value).expect("sim schema serializes");
    format!("{:x}", Sha256::digest(bytes))
}

/// Domain-separated deterministic draws, independent of visitation order.
pub fn draw(seed: u64, domain: &str, values: &[u64]) -> u64 {
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    hash.update(seed.to_le_bytes());
    hash.update((domain.len() as u64).to_le_bytes());
    hash.update(domain.as_bytes());
    for value in values {
        hash.update(value.to_le_bytes());
    }
    u64::from_le_bytes(hash.finalize()[..8].try_into().unwrap())
}
