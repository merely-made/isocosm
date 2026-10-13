// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{history::Session, simulation::Simulation};

impl Simulation {
    fn graft_candidate(&self, participant: Id, actor: Id, target: Id, process: &str,
        carry: &Carry) -> Result<Self> {
        if actor == target || self.graft_input.is_some() {
            return Err("graft carrying needs distinct bodies and cannot nest".into());
        }
        if !self.state.population.get(participant).is_some_and(crate::directing::is_participant)
            || self.plays(participant) != Some(actor)
            || !self.state.population.get(actor).is_some_and(|body| body.method == Method::Normative) {
            return Err("controlled graft carrying requires the participant's actual normative body".into());
        }
        let definition = self.genesis.rules.processes.get(process).ok_or("graft process is absent")?;
        process_costs(definition)?;
        validate(definition, &self.genesis.rules)?;
        let mut candidate = self.clone();
        candidate.graft_input = Some(Input { actor, target, process: process.into(), carry: carry.clone() });
        Ok(candidate)
    }

    pub fn graft_preview(&self, actor: Id, target: Id, process: &str, carry: &Carry) -> Result<Preview> {
        if actor == target { return Err("a graft cannot take its own donor branch".into()); }
        let definition = self.genesis.rules.processes.get(process).ok_or("graft process is absent")?;
        let (reserve, soil) = process_costs(definition)?;
        validate(definition, &self.genesis.rules)?;
        prepare(self.state.population.get(actor).ok_or("graft actor is absent")?,
            self.state.population.get(target).ok_or("graft donor is absent")?, carry, reserve, soil,
            &self.genesis.rules, &self.state.lineages)
    }

    pub(crate) fn act_graft(&mut self, participant: Id, actor: Id, target: Id, process: &str,
        cause: Option<Key>, carry: &Carry, selections: &[crate::projects::DebitSelection]) -> Result<String> {
        let mut candidate = self.graft_candidate(participant, actor, target, process, carry)?;
        let result = candidate.owned_act(participant, actor, Some(target), process, cause, selections);
        candidate.graft_input = None;
        let encoded = result?;
        let admitted: crate::projects::OwnedActed = serde_json::from_str(&encoded).map_err(|e| e.to_string())?;
        if !admitted.receipt.accepted() {
            return Err(format!("graft act was not admitted: {:?}", admitted.receipt.outcome));
        }
        // A conditional branch that did not carry anything cannot claim that
        // this exact selected graft input was accepted.
        if !candidate.state.notes.iter().any(|note| note.core.kind == KIND
            && note.core.cause == admitted.receipt.id) {
            return Err("the accepted process did not perform the supplied graft".into());
        }
        *self = candidate;
        Ok(encoded)
    }
}

impl Session {
    pub fn prepare_owned_graft(&self, participant: Id, actor: Id, target: Id, process: &str,
        carry: &Carry, cause: Option<Key>) -> Result<Prepared> {
        let mut candidate = self.sim.graft_candidate(participant, actor, target, process, carry)?;
        let graft = candidate.graft_preview(actor, target, process, carry)?;
        let result = candidate.prepare_owned_act(actor, Some(target), process, cause);
        candidate.graft_input = None;
        Ok(Prepared { graft, debits: result? })
    }
}
