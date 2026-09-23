// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The first supported reduction is exact state equivalence for independent
//! unary processes. It is not tau-leaping and makes no approximation claim.

use crate::{Execution, Founding, Result, Simulation, schema::*, simulation::Work};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comparison {
    pub founding: Founding,
    pub ticks: Tick,
    pub checkpoints_checked: u64,
    pub final_hash: Key,
    pub matter: u128,
    pub individuals: Work,
    pub grouped: Work,
    pub final_groups: usize,
}

pub fn compare(founding: &Founding, ticks: Tick) -> Result<Comparison> {
    let genesis = founding.generate()?;
    let mut individuals = Simulation::new(genesis.clone(), Execution::Individuals)?;
    let mut grouped = Simulation::new(genesis, Execution::Grouped)?;
    let mut result = Comparison {
        founding: founding.clone(),
        ticks,
        checkpoints_checked: 0,
        final_hash: String::new(),
        matter: grouped.matter(),
        individuals: Work::default(),
        grouped: Work::default(),
        final_groups: 0,
    };
    for step in 0..ticks {
        // Exercise changing observation without changing physical rules.
        if step % 7 == 0 {
            let id = u64::from(founding.sites)
                + crate::draw(founding.seed, "inspection", &[step]) % founding.population;
            individuals.inspect(id)?;
            grouped.inspect(id)?;
            if step % 14 == 0 {
                individuals.release(id);
                grouped.release(id);
            }
        }
        add(&mut result.individuals, individuals.advance(1)?);
        add(&mut result.grouped, grouped.advance(1)?);
        if individuals.state_hash() != grouped.state_hash() {
            return Err(format!(
                "foreground/background diverged at seed {}, tick {}",
                founding.seed,
                step + 1
            ));
        }
        if individuals.matter() != result.matter || grouped.matter() != result.matter {
            return Err("conserved matter changed".into());
        }
        result.checkpoints_checked += 1;
    }
    result.final_hash = grouped.state_hash();
    result.final_groups = grouped.state().population.groups.len();
    Ok(result)
}

fn add(total: &mut Work, work: Work) {
    total.evaluations += work.evaluations;
    total.represented += work.represented;
    total.accepted += work.accepted;
    total.blocked += work.blocked;
}
