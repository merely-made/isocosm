// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{history::{Command, Session}, simulation::{Outcome, Receipt}};

fn refusal(receipt: &Receipt) -> Option<String> {
    match &receipt.outcome { Outcome::Blocked(why) | Outcome::Refused(why) => Some(why.clone()), _ => None }
}

impl Simulation {
    fn supplied_process(&self, actor:Id, target:Option<Id>, process:&str, mode:Mode) -> Context {
        let mut context = Context::new(&self.state,&self.genesis.rules,crate::digest(&(&self.revision,&self.state)),
            DebitOrigin::Process { process:process.into(),actor,target },
            self.owned_action(actor,process,self.state.next_action),mode);
        context.produced = self.produced_at_start();
        context
    }
    pub fn prepare_owned_act(&self, actor:Id, target:Option<Id>, process:&str, cause:Option<Key>) -> Result<DebitPreview> {
        let mut candidate = self.clone();
        candidate.state.projects.debit_capture.set(Some(self.supplied_process(actor,target,process,Mode::Preview)));
        candidate.owned_planning = true;
        let receipt = candidate.execute(actor,target,process,cause);
        let refusal = refusal(&receipt);
        Ok(DebitPreview { requests:candidate.owned_report.requests,receipt:Some(receipt),refusal })
    }
    pub fn owned_act(&mut self, participant:Id, actor:Id, target:Option<Id>, process:&str,
        cause:Option<Key>, selections:&[DebitSelection]) -> Result<String> {
        self.project_control(participant,actor)?;
        if self.state.projects.debit_capture.active() { return Err("owned native action cannot nest".into()); }
        let mut candidate = self.clone();
        candidate.state.projects.debit_capture.set(Some(self.supplied_process(actor,target,process,Mode::Admit(selections.to_vec()))));
        let receipt = candidate.execute(actor,target,process,cause);
        let accepted = receipt.accepted();
        let result = OwnedActed { receipt,debits:candidate.owned_report.requests.clone(),claims:candidate.owned_report.claims.clone() };
        let outcome = serde_json::to_string(&result).map_err(|e| e.to_string())?;
        if accepted { *self = candidate; }
        Ok(outcome)
    }
}

impl Session {
    fn direct_capture(&self, command:&Command, mode:Mode) -> Result<Context> {
        types::direct_operation(command)?;
        let action = format!("intent-stock:{}",crate::digest(&(self.sim.genesis.seed,self.sim.state.tick,
            self.entries.len() as u64,command)));
        let mut context = Context::new(&self.sim.state,&self.sim.genesis.rules,
            crate::digest(&(&self.sim.revision,&self.sim.state)),DebitOrigin::Command(Box::new(command.clone())),action,mode);
        let mut bindings = Vec::new();
        match command {
            Command::Wound { entity, .. } => {
                bindings.push((crate::rules::Binding::Actor,StockSource::Entity(*entity)));
                if let Some(site) = self.sim.state.population.get(*entity).and_then(Entity::resident_site) {
                    bindings.push((crate::rules::Binding::Place,StockSource::Site(site)));
                }
            },
            Command::Edit { by:Some(actor),site, .. } => {
                bindings.push((crate::rules::Binding::Actor,StockSource::Entity(*actor)));
                bindings.push((crate::rules::Binding::Place,StockSource::Site(*site)));
            },
            _ => {},
        }
        context.begin(state_balances(&self.sim.state),bindings,None)?;
        Ok(context)
    }
    pub fn prepare_owned_command(&self, command:&Command) -> Result<DebitPreview> {
        self.direct_preview(command).map(|preview| preview.debits)
    }
    /// Actual route and generated-strain outputs of this exact native candidate.
    pub fn prepare_owned_spread(&self, command:&Command) -> Result<SpreadPreview> {
        if !matches!(command,Command::Spread(_)) { return Err("spread preview requires a real native spread command".into()); }
        self.direct_preview(command)
    }
    fn direct_preview(&self, command:&Command) -> Result<SpreadPreview> {
        let context = self.direct_capture(command,Mode::Preview)?;
        let mut candidate = self.clone();
        candidate.sim.state.projects.debit_capture.set(Some(context));
        let result = crate::history::run(&mut candidate.sim,command);
        let context = candidate.sim.state.projects.debit_capture.take().ok_or("native direct preview lost its capture")?;
        Ok(SpreadPreview { debits:DebitPreview { requests:context.requests,receipt:None,refusal:result.err() },
            outputs:context.output_report })
    }
    pub(crate) fn owned_command(&mut self, command:&Command, selections:&[DebitSelection]) -> Result<String> {
        if self.sim.state.projects.debit_capture.active() { return Err("owned native command cannot nest".into()); }
        let context = self.direct_capture(command,Mode::Admit(selections.to_vec()))?;
        let mut candidate = self.clone();
        candidate.sim.state.projects.debit_capture.set(Some(context));
        let result = crate::history::run(&mut candidate.sim,command);
        let mut context = candidate.sim.state.projects.debit_capture.take().ok_or("native direct admission lost its capture")?;
        let outcome = result?;
        if context.active { context.finish(state_balances(&candidate.sim.state))?; }
        context.complete()?;
        for (key, held) in context.changes() { candidate.sim.state.projects.ownership.insert(key,held); }
        candidate.sim.state.projects.produced.extend(context.produced_changes());
        let result = OwnedCommandResult { outcome,debits:context.requests,claims:context.claims };
        let outcome = serde_json::to_string(&result).map_err(|e| e.to_string())?;
        *self = candidate;
        Ok(outcome)
    }
}
