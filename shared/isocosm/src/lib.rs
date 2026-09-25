// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sim owns accepted world transitions. Hosts own input, pacing and views.
//! Exact population grouping is optional; both execution modes use the same
//! admitted processes. Unsupported bulk operations execute individually.

pub mod aggregate;
pub mod bench;
mod ecology;
mod execute;
pub mod generate;
mod genesis;
pub mod history;
pub mod population;
pub mod probe;
mod queries;
pub mod reach;
pub mod rules;
pub mod schema;
pub mod simulation;
mod validation;

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
