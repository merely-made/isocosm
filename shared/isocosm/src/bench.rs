// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Execution, Founding, Result, Session,
    aggregate::{self, Comparison},
    history::Command,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Draws {
    pub version: u32,
    pub master_seed: u64,
    pub domain: String,
    pub comparisons: Vec<Comparison>,
    pub saved_replays: u64,
    pub refusal_checks: u64,
}

/// Receipts vary laws and topology, as well as initial populations. The domain
/// is explicit; this does not certify unimplemented social/spatial reductions.
pub fn draws(master_seed: u64, count: u64, ticks: u64) -> Result<Draws> {
    if count == 0 || count > 1024 || ticks > 1000 {
        return Err("bench draw budget exceeded".into());
    }
    let mut report = Draws { version: crate::VERSION, master_seed,
        domain: "alternating unary reservoir networks and shared-resource ecological compartments; 1..8 sites, 1..6 lineages, 16..128 initial members, 2..4 accounts/lineage; changing observed identities; no spatial-contact claim".into(),
        comparisons: vec![], saved_replays: 0, refusal_checks: 0 };
    for i in 0..count {
        let seed = crate::draw(master_seed, "bench-seed", &[i]);
        let founding = Founding {
            seed,
            sites: (1 + seed % 8) as u32,
            lineages: (1 + seed.rotate_left(7) % 6) as u32,
            population: 16 + seed.rotate_left(13) % 113,
            cohort_size: 4 + seed.rotate_left(23) % 29,
            ecology: i % 2 == 1,
            ..Founding::default()
        };
        report.comparisons.push(
            aggregate::compare(&founding, ticks)
                .map_err(|why| format!("draw {i}, seed {seed}: {why}"))?,
        );
        let mut session = Session::new(founding.generate()?, Execution::Grouped)?;
        session.advance(ticks)?;
        let actor = u64::from(founding.sites);
        session.command(Command::Act {
            actor,
            target: None,
            process: "sim:remember".into(),
            cause: None,
        })?;
        let encoded = serde_json::to_vec(&session.save()).map_err(|e| e.to_string())?;
        let saved = serde_json::from_slice(&encoded).map_err(|e| e.to_string())?;
        let replay = Session::load(saved, Execution::Individuals)?;
        if replay.sim.state_hash() != session.sim.state_hash() {
            return Err("saved replay mismatch".into());
        }
        report.saved_replays += 1;
        let before = session.sim.state_hash();
        let refusal = session.sim.execute(actor, None, "unknown:process", None);
        if !matches!(refusal.outcome, crate::simulation::Outcome::Refused(_))
            || before != session.sim.state_hash()
        {
            return Err("refusal was not inert".into());
        }
        report.refusal_checks += 1;
    }
    Ok(report)
}
