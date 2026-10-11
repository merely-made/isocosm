// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Probe-owned whole-world checks, including the dev source's issued matter.

use isocosm::{
    Execution, Simulation,
    probe::{ExactRun, ProbeWorld, run_exact_traced},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Matter {
    pub initial: u128,
    pub final_total: u128,
    pub issued: u128,
    pub verified_ticks: u64,
}

impl Matter {
    fn new(sim: &Simulation) -> Self {
        Self {
            initial: sim.matter(),
            final_total: sim.matter(),
            issued: sim.issued(),
            verified_ticks: 0,
        }
    }

    fn observe(&mut self, sim: &Simulation) -> Result<(), String> {
        let total = sim.matter();
        let issued = sim.issued();
        if self.initial.checked_add(issued) != Some(total) {
            return Err(format!(
                "native matter changed at tick {}",
                sim.state().tick
            ));
        }
        if sim.state().tick != self.verified_ticks + 1 {
            return Err("native matter audit skipped a tick".into());
        }
        self.final_total = total;
        self.issued = issued;
        self.verified_ticks += 1;
        Ok(())
    }
}

pub fn run(world: &ProbeWorld, dynamics: u64) -> Result<(ExactRun, Matter), String> {
    let initial = Simulation::new(world.genesis.clone(), Execution::Individuals)?;
    let mut matter = Matter::new(&initial);
    let mut failure = None;
    let mut each = |sim: &Simulation| {
        if failure.is_none() {
            failure = matter.observe(sim).err();
        }
    };
    let run = run_exact_traced(world, dynamics, true, &mut each)?;
    if let Some(why) = failure {
        return Err(why);
    }
    if matter.verified_ticks != world.ticks {
        return Err("native matter audit did not cover the declared run".into());
    }
    Ok((run, matter))
}

#[cfg(test)]
mod tests {
    use super::*;
    use isocosm::probe::{BodyFounding, HarmFounding};

    #[test]
    fn every_declared_harm_tick_has_conserved_native_totals() {
        let world = BodyFounding {
            ticks: 3,
            harm: Some(HarmFounding::default()),
            ..Default::default()
        }
        .generate()
        .unwrap();
        let (run, matter) = super::run(&world, 57).unwrap();
        assert_eq!(matter.verified_ticks, 3);
        assert_eq!(matter.final_total, matter.initial + matter.issued);
        assert_eq!(run.sim.matter(), matter.final_total);
    }

    #[test]
    fn audit_refuses_a_changed_world_total() {
        let founding = BodyFounding {
            sites: [1, 1],
            soil: [100, 100],
            ..Default::default()
        };
        let before =
            Simulation::new(founding.generate().unwrap().genesis, Execution::Individuals).unwrap();
        let mut changed = Simulation::new(
            BodyFounding {
                soil: [101, 101],
                ..founding
            }
            .generate()
            .unwrap()
            .genesis,
            Execution::Individuals,
        )
        .unwrap();
        changed.advance(1).unwrap();
        let mut matter = Matter::new(&before);
        assert!(
            matter
                .observe(&changed)
                .unwrap_err()
                .contains("matter changed")
        );
        assert_eq!(matter.verified_ticks, 0);
    }

    #[test]
    fn issued_matter_is_counted_and_skipped_ticks_are_refused() {
        let world = BodyFounding::default().generate().unwrap();
        let mut sim = Simulation::new(world.genesis, Execution::Individuals).unwrap();
        let mut matter = Matter::new(&sim);
        let site = *sim.state().sites.keys().next().unwrap();
        sim.place(site, "world:soil", 7).unwrap();
        sim.advance(1).unwrap();
        matter.observe(&sim).unwrap();
        assert_eq!(matter.issued, 7);
        assert_eq!(matter.final_total, matter.initial + 7);
        sim.advance(2).unwrap();
        assert!(matter.observe(&sim).unwrap_err().contains("skipped a tick"));
        assert_eq!(matter.verified_ticks, 1);
    }
}
