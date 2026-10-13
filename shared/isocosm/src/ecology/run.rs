// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One ordinary scheduled trajectory, with its genuine clock/geometry history.

use super::*;
use crate::{Command, Result, Session, flows::FlowResult,
    history::Saved, simulation::{Execution, Genesis, Work}};

fn clock_input(command: &Command) -> Result<()> {
    match command {
        Command::AdvanceGeometry { ticks: 1, .. } | Command::AdvanceSpatial { ticks: 1, .. }
            | Command::AdvanceBirthSpatial { ticks: 1, .. } | Command::Patch { .. } => Ok(()),
        _ => Err("joint witness permits only actual one-tick clocks and physical patch inputs".into()),
    }
}

fn declared(genesis: &Genesis, spec: &JointSpec) -> Result<()> {
    spec.validate_genesis(genesis)?;
    if genesis.rules.ecology_witness.as_ref() != Some(spec) {
        return Err("joint witness must be the exact declared native ecology specification".into());
    }
    Ok(())
}

/// The callback is the actual physical driver, or the ordinary native one-tick clock.
/// No actor is forced to perform a role; missing chosen acts remain missing evidence.
pub fn run_joint(genesis: &Genesis, spec: &JointSpec, execution: Execution,
    advance: &mut dyn FnMut(&mut Session) -> Result<FlowResult<Work>>) -> Result<JointWitness> {
    declared(genesis, spec)?;
    let mut session = Session::new(genesis.clone(), execution)?;
    session.sim.begin_ecology_witness(spec)?;
    let result = (|| -> Result<()> {
        for tick in 0..spec.horizon {
            let entries = session.entries.clone();
            let branch = session.branch.clone();
            let result = advance(&mut session)?;
            let next = tick.checked_add(1).ok_or("joint witness clock overflow")?;
            if result.tick != next || session.sim.state().tick != next || session.sim.mode() != execution
                || session.sim.genesis() != genesis || session.branch != branch
                || !session.entries.starts_with(&entries) {
                return Err("joint witness driver changed its native source or one-tick boundary".into());
            }
            for entry in &session.entries[entries.len()..] { clock_input(&entry.command)?; }
        }
        Ok(())
    })();
    // Scratch is withdrawn on either result and cannot become saved native authority.
    let acts = session.sim.end_ecology_witness();
    result?;
    let witness = JointWitness { version: 1, spec: spec.clone(), execution,
        totals: spec.totals(&acts, &genesis.rules)?, native: session.save(), acts };
    witness.validate()?;
    Ok(witness)
}

fn replay_acts(saved: &Saved, spec: &JointSpec, execution: Execution) -> Result<Vec<ActionProof>> {
    let movement = saved.movement.ok_or("joint witness lacks exact native movement semantics")?;
    let mut session = Session::new_with_movement(saved.genesis.clone(), execution, movement)?;
    session.branch = saved.branch.clone();
    session.sim.begin_ecology_witness(spec)?;
    let result = (|| -> Result<()> {
        for entry in &saved.entries {
            clock_input(&entry.command)?;
            if entry.tick < session.sim.state().tick || entry.tick > spec.horizon {
                return Err("joint witness history has an invalid clock order".into());
            }
            while session.sim.state().tick < entry.tick { session.advance(1)?; }
            let outcome = session.command(entry.command.clone())?;
            if outcome != entry.outcome || session.entries.last() != Some(entry) {
                return Err("joint witness exact native entry failed replay".into());
            }
        }
        while session.sim.state().tick < spec.horizon { session.advance(1)?; }
        if session.save() != *saved { return Err("joint witness native preimages failed replay".into()); }
        Ok(())
    })();
    let acts = session.sim.end_ecology_witness();
    result?;
    Ok(acts)
}

impl JointWitness {
    /// Replays this same trajectory and its accepted observations, never separate role trials.
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 || self.native.tick != self.spec.horizon {
            return Err("joint witness version or authored horizon mismatch".into());
        }
        declared(&self.native.genesis, &self.spec)?;
        let acts = replay_acts(&self.native, &self.spec, self.execution)?;
        if acts != self.acts || self.spec.totals(&acts, &self.native.genesis.rules)? != self.totals {
            return Err("joint ecology evidence differs from its actual native trajectory".into());
        }
        Ok(())
    }

    pub fn require_roles(&self) -> Result<()> {
        self.validate()?;
        let missing = self.spec.missing(&self.totals);
        if !missing.is_empty() { return Err(format!("joint ecology witness is incomplete: {}", missing.join("; "))); }
        Ok(())
    }
}
